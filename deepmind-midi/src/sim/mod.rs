//! A synthesizer to talk to, made of the same protocol.
//!
//! [`device`](crate::device) is the host's end of the conversation. This is the
//! other end: something that answers requests, applies the edits it is sent and
//! says what it heard. It exists because a host cannot be tested against a
//! `DeepMind` that is not plugged in, and hand-assembling reply frames in every
//! test is how that gap gets filled otherwise.
//!
//! ```
//! use deepmind_midi::device::{Device, Event};
//! use deepmind_midi::ids::{DeviceId, ProtocolVersion};
//! use deepmind_midi::program::{Program, ProgramName};
//! use deepmind_midi::sim::Synth;
//!
//! // A unit with a sound in its edit buffer.
//! let mut sound = Program::new(ProtocolVersion::V7);
//! sound.set_name(ProgramName::new("Bass Sweep")?);
//! let mut synth: Synth = Synth::new(DeviceId::Unit(0), sound);
//!
//! // A host that wants to know what it is holding.
//! let mut host: Device = Device::new(DeviceId::Unit(0));
//! host.request_edit_buffer()?;
//!
//! // One turn of the loop, in each direction.
//! host.drain_tx(|bytes| { synth.feed(bytes); Ok::<(), core::convert::Infallible>(()) }).ok();
//! synth.drain_tx(|bytes| { host.feed(bytes); Ok::<(), core::convert::Infallible>(()) }).ok();
//!
//! let Some(Event::EditBuffer(program)) = host.poll_event() else {
//!     panic!("the synthesizer answered");
//! };
//! assert_eq!(program.name().as_str(), "Bass Sweep");
//! # Ok::<(), deepmind_midi::Error>(())
//! ```
//!
//! # What this is not
//!
//! **It is not a `DeepMind`, and it cannot find out that the manual is wrong.**
//! It answers what `spec/messages.toml` says a unit answers, encoded by the same
//! code the library decodes with, so a round trip through it proves only that
//! this library agrees with itself.
//!
//! What it does find is a host that drives the protocol wrongly: one that reads
//! a dump it never asked for, mistakes a stored program for the edit buffer,
//! never drains its queue, or assumes an answer arrives before the next request
//! goes out. Those bugs are reachable without hardware.
//!
//! # The loop
//!
//! The same one [`Device`](crate::device::Device) writes:
//!
//! ```text
//! synth.feed(&bytes_from_the_host);                // inbound, any chunking
//! synth.drain_tx(|bytes| host.feed(bytes))?;       // outbound
//! while let Some(heard) = synth.poll_heard() {}    // what it made of it
//! ```
//!
//! No clock. This unit answers as soon as it is drained, so the test owns the
//! latency: not draining is a synthesizer that has not replied yet, which is how
//! a host's timeout path gets exercised. See [`Synth::drain_tx`].
//!
//! # Stored programs come from a [`Library`]
//!
//! Eight banks are two hundred kilobytes, too much to hold inline on the target
//! this crate is `no_std` for. So a [`Synth`] holds its edit buffer and nothing
//! else, and stored programs come from whatever the caller supplies: [`Empty`]
//! for a unit whose memory is not part of the test, or a [`syx::File`], which
//! makes a preset pack the contents of a simulated unit.
//!
//! A slot the library does not hold is not answered. A real unit always has
//! something in every slot, but an unanswered request is more useful to test
//! than an invented program.
//!
//! # What it answers
//!
//! Every request [`Device`](crate::device::Device) can send, and the control
//! application handshake:
//!
//! | Request | Answer |
//! |---|---|
//! | Device inquiry | Its [`Identity`] |
//! | Control app notify | Its channels, interface and selection |
//! | Edit buffer dump | The program it is holding |
//! | Program dump | The [`Library`]'s program for that slot, if it has one |
//! | Program bank dump | One dump per slot of the run the library holds |
//! | Single program name dump | The name of the library's program for that slot |
//!
//! The globals, the patterns, the chord memories and the calibration data are
//! requests it hears and does not answer, because the manual gives those
//! payloads a length and never says what is in them. A bank of program names is
//! decodable and is not answered either: two kilobytes of reply is more than
//! this is for, and no request for one can be built by
//! [`Device`](crate::device::Device).
//!
//! [`Heard::Request`] reports both the command and whether an answer was queued,
//! so a test can tell "asked and ignored" from "asked and answered".

use crate::error::{Error, Result};
use crate::ids::{Bank, DeviceId, ProgramNumber, Slot};
use crate::nrpn::{Change, Nrpn};
use crate::param::{NrpnEdit, ParamId};
use crate::program::Program;
use crate::queue::Queue;
use crate::sysex::inquiry::{self, Identity, Version};
use crate::sysex::{Command, Frame, Interface, Message, PROGRAM_NAME_LEN, packed};
use crate::syx;
use crate::wire::{Channel, ChannelMessage, Decoder, Event as WireEvent, MAX_SYSEX_LEN};

/// Replies a [`Synth`] holds by default.
///
/// A whole bank run is one of them, so this bounds how many different things a
/// host can ask about before draining rather than how many dumps are owed.
pub const DEFAULT_REPLY_DEPTH: usize = 16;

/// Observations a [`Synth`] holds by default.
pub const DEFAULT_HEARD_DEPTH: usize = 16;

/// Returns the larger of two lengths.
const fn larger(a: usize, b: usize) -> usize {
    if a > b { a } else { b }
}

/// Bytes the longest reply occupies, which is a program dump.
pub const MAX_REPLY_LEN: usize = larger(syx::MAX_PROGRAM_FRAME_LEN, inquiry::REPLY_LEN);

/// Where a [`Synth`] finds the programs it has stored.
///
/// A slot it does not hold is [`None`], and a request for that slot goes
/// unanswered.
pub trait Library {
    /// Returns the program stored in `slot`.
    fn program(&self, slot: Slot) -> Option<Program>;
}

/// A unit with nothing in its memory.
///
/// Every stored-program request goes unanswered, which is what a test that only
/// cares about the edit buffer wants.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Empty;

impl Library for Empty {
    fn program(&self, _slot: Slot) -> Option<Program> {
        None
    }
}

impl<L: Library + ?Sized> Library for &L {
    fn program(&self, slot: Slot) -> Option<Program> {
        (**self).program(slot)
    }
}

/// A preset pack is a unit's memory: the slots the file names are the slots the
/// synthesizer has.
///
/// The file is walked per request rather than indexed, which is the right trade
/// for something answering one request at a time and holding no allocator. A
/// frame that does not parse is skipped, the same as everywhere else in
/// [`syx`](crate::syx).
impl Library for syx::File<'_> {
    fn program(&self, slot: Slot) -> Option<Program> {
        self.programs()
            .filter_map(core::result::Result::ok)
            .find(|entry| entry.slot == Some(slot))
            .map(|entry| entry.program)
    }
}

/// What a [`Synth`] made of what arrived.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Heard {
    /// A device inquiry arrived and was answered.
    Inquiry,
    /// A request arrived, and whether anything was queued in reply.
    ///
    /// Unanswered means one of two things, and which is which is in the
    /// [module documentation](self#what-it-answers): a payload the manual never
    /// describes, or a slot the [`Library`] does not hold.
    Request {
        /// The request.
        command: Command,
        /// Whether a reply was queued for it.
        answered: bool,
    },
    /// A parameter was set, and the edit buffer now holds the new value.
    Parameter {
        /// The parameter named.
        parameter: ParamId,
        /// The value it was set to.
        value: u16,
        /// Whether the message carried the whole value. An NRPN does; a control
        /// change carries seven bits of it.
        exact: bool,
    },
    /// A channel message that was not a parameter edit: notes, bend, program
    /// change, and the controllers that reach something other than a parameter.
    Channel {
        /// Channel it arrived on.
        channel: Channel,
        /// The message itself.
        message: ChannelMessage,
    },
    /// A `DeepMind` message that is not a request.
    ///
    /// A dump response is something a unit sends, not something it is sent. The
    /// manual documents no message that writes a program into a synthesizer, so
    /// one arriving is reported and otherwise ignored.
    Unexpected(Command),
    /// A frame that could not be read, and why.
    Failed(Error),
    /// A `SysEx` frame from another manufacturer or another model.
    Foreign {
        /// The three manufacturer ID bytes the frame carried.
        manufacturer: [u8; 3],
        /// The model ID byte the frame carried.
        model: u8,
    },
    /// Observations were dropped because the queue was full, and how many.
    Lost(u16),
}

/// One reply, held as what to send rather than as the bytes of it.
///
/// A bank run is one item: a whole bank is 128 dumps and 37 kilobytes, and
/// queuing that as bytes would cost more than the unit being simulated has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reply {
    Identity,
    ControlApp,
    EditBuffer,
    Program(Slot),
    ProgramName(Slot),
    /// A run of stored programs, remembering which one is next.
    Run {
        bank: Bank,
        next: u8,
        last: u8,
    },
    Parameter(NrpnEdit),
}

/// A simulated `DeepMind`.
///
/// See the [module documentation](self) for what it answers, what it cannot
/// prove, and the loop a test writes around it.
#[derive(Debug)]
pub struct Synth<
    L = Empty,
    const RX: usize = MAX_SYSEX_LEN,
    const TX: usize = DEFAULT_REPLY_DEPTH,
    const EV: usize = DEFAULT_HEARD_DEPTH,
> {
    id: DeviceId,
    channel: Channel,
    interface: Interface,
    identity: Identity,
    edit_buffer: Program,
    selected: Slot,
    library: L,
    decoder: Decoder<RX>,
    tx: Queue<Reply, TX>,
    heard: Queue<Heard, EV>,
    lost: u16,
    nrpn: Nrpn,
}

impl<const RX: usize, const TX: usize, const EV: usize> Synth<Empty, RX, TX, EV> {
    /// Builds a unit holding `edit_buffer` and nothing else.
    ///
    /// The program decides the comms protocol version the unit speaks, since
    /// that is what decides how long a dump of it is.
    ///
    /// [`DeviceId::Broadcast`] is an address rather than a unit, and a unit
    /// built with it replies with it, which no synthesizer does. Give it the
    /// [`Unit`](DeviceId::Unit) the test is about.
    #[must_use]
    pub fn new(id: DeviceId, edit_buffer: Program) -> Self {
        Self::with_library(id, edit_buffer, Empty)
    }
}

impl<L: Library, const RX: usize, const TX: usize, const EV: usize> Synth<L, RX, TX, EV> {
    /// Builds a unit whose stored programs come from `library`.
    #[must_use]
    pub fn with_library(id: DeviceId, edit_buffer: Program, library: L) -> Self {
        Self {
            id,
            channel: Channel::ONE,
            interface: Interface::Midi,
            identity: Identity {
                device: id,
                firmware: Version { major: 1, minor: 1 },
                voice: Version { major: 1, minor: 0 },
            },
            edit_buffer,
            selected: Slot::FIRST,
            library,
            decoder: Decoder::new(),
            tx: Queue::new(),
            heard: Queue::new(),
            lost: 0,
            nrpn: Nrpn::new(),
        }
    }

    /// Sets the channel the unit listens and sends on. Channel 1 by default.
    #[must_use]
    pub const fn with_channel(mut self, channel: Channel) -> Self {
        self.channel = channel;
        self
    }

    /// Sets what the unit answers a device inquiry with.
    ///
    /// Firmware 1.1 and voice 1.0 by default, which is what the value tables in
    /// [`param`](crate::param) describe unless a host says otherwise.
    #[must_use]
    pub const fn with_identity(mut self, identity: Identity) -> Self {
        self.identity = identity;
        self
    }

    /// Sets the interface the unit says it is reachable over. MIDI by default.
    #[must_use]
    pub const fn with_interface(mut self, interface: Interface) -> Self {
        self.interface = interface;
        self
    }

    /// Sets the slot the unit says is selected. The first by default.
    #[must_use]
    pub const fn with_selected(mut self, slot: Slot) -> Self {
        self.selected = slot;
        self
    }

    /// Returns the unit's device ID.
    #[must_use]
    pub const fn id(&self) -> DeviceId {
        self.id
    }

    /// Returns the channel it listens and sends on.
    #[must_use]
    pub const fn channel(&self) -> Channel {
        self.channel
    }

    /// Returns what it answers a device inquiry with.
    #[must_use]
    pub const fn identity(&self) -> Identity {
        self.identity
    }

    /// Returns the slot it says is selected.
    #[must_use]
    pub const fn selected(&self) -> Slot {
        self.selected
    }

    /// Returns the program it is holding, which is the sound it is making.
    #[must_use]
    pub const fn edit_buffer(&self) -> &Program {
        &self.edit_buffer
    }

    /// Replaces the program it is holding, as loading one at the front panel
    /// would.
    ///
    /// Nothing is sent: a `DeepMind` does not announce a recall, and a host that
    /// wants to know asks.
    pub fn set_edit_buffer(&mut self, program: Program) {
        self.edit_buffer = program;
    }

    /// Returns the library its stored programs come from.
    #[must_use]
    pub const fn library(&self) -> &L {
        &self.library
    }

    /// Returns the library, to change what the unit has stored.
    pub const fn library_mut(&mut self) -> &mut L {
        &mut self.library
    }

    /// Returns how many replies are waiting to be drained.
    #[must_use]
    pub const fn queued(&self) -> usize {
        self.tx.len()
    }

    /// Drops everything queued and everything half-decoded, as a power cycle
    /// would. The edit buffer and the library are left alone.
    pub fn reset(&mut self) {
        self.decoder.reset();
        self.tx.clear();
        self.heard.clear();
        self.lost = 0;
        self.nrpn = Nrpn::new();
    }

    /// Takes the oldest observation.
    pub fn poll_heard(&mut self) -> Option<Heard> {
        if let Some(heard) = self.heard.pop() {
            return Some(heard);
        }
        let lost = core::mem::take(&mut self.lost);
        (lost > 0).then_some(Heard::Lost(lost))
    }

    /// Sets a parameter and queues the NRPN that says so, as turning a knob on
    /// the front panel does.
    ///
    /// This is the inbound direction a host is easy to get wrong: the
    /// synthesizer is not only answering, it is also volunteering. What goes out
    /// is the four control changes of one [`NrpnEdit`], which is what a
    /// `DeepMind` sends when a control application is attached.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ValueOutOfRange`] when the parameter does not accept the
    /// value, and [`Error::QueueFull`] when there is no room to queue the
    /// message. Either way the edit buffer is left as it was.
    pub fn turn(&mut self, parameter: ParamId, value: u16) -> Result<()> {
        let edit = NrpnEdit::new(parameter, value)?;
        let byte = u8::try_from(value).map_err(|_| Error::ValueOutOfRange {
            parameter: parameter.offset(),
            value,
            max: parameter.max(),
        })?;
        if self.tx.remaining() == 0 {
            return Err(Error::QueueFull {
                needed: 1,
                available: 0,
            });
        }
        self.edit_buffer.set(parameter, byte)?;
        self.tx.push(Reply::Parameter(edit));
        Ok(())
    }

    /// Feeds inbound bytes, in whatever chunks they arrived in.
    pub fn feed(&mut self, bytes: &[u8]) {
        let Self {
            id,
            channel,
            decoder,
            heard,
            lost,
            tx,
            edit_buffer,
            selected,
            library,
            nrpn,
            ..
        } = self;
        let mut inbound = Inbound {
            id: *id,
            channel: *channel,
            heard,
            lost,
            tx,
            edit_buffer,
            selected,
            library,
            nrpn,
        };
        decoder.feed(bytes, |event| inbound.handle(event));
    }

    /// Hands every queued reply to `send`, oldest first, and returns how many
    /// went.
    ///
    /// One call per message, as [`Device::drain_tx`](crate::device::Device::drain_tx)
    /// does. A bank run is one queued item and many calls: a reply `send`
    /// refuses stays queued, and a run that was refused halfway resumes where it
    /// stopped rather than starting again.
    ///
    /// Not draining is a unit that has not answered yet, which is the whole of
    /// how a host's timeout path is tested. There is no clock here to do it
    /// with.
    ///
    /// # Errors
    ///
    /// Returns whatever `send` returns, unchanged.
    pub fn drain_tx<E, F>(&mut self, mut send: F) -> core::result::Result<usize, E>
    where
        F: FnMut(&[u8]) -> core::result::Result<(), E>,
    {
        let mut count = 0;
        while let Some(item) = self.tx.peek() {
            // A run is many messages from one item, so it says which slot is
            // next and is popped only once it is spent.
            let (slot, last) = match item {
                Reply::Run { bank, next, last } => (
                    ProgramNumber::new(next)
                        .ok()
                        .map(|number| Slot { bank, number }),
                    Some(last),
                ),
                _ => (None, None),
            };

            let mut bytes = [0; MAX_REPLY_LEN];
            let written = match self.encode(item, slot, &mut bytes) {
                Ok(written) => written,
                // A slot the library does not hold writes nothing, which for a
                // run means moving on to the next one.
                Err(Skip::Empty) => {
                    self.advance(last);
                    continue;
                }
                Err(Skip::Failed(error)) => {
                    enqueue(&mut self.heard, &mut self.lost, Heard::Failed(error));
                    self.advance(last);
                    continue;
                }
            };
            send(bytes.get(..written).unwrap_or_default())?;
            self.advance(last);
            count += 1;
        }
        Ok(count)
    }

    /// Moves past the reply just sent: one step through a run, or off the queue.
    fn advance(&mut self, last: Option<u8>) {
        match last {
            Some(last) => match self.tx.peek_mut() {
                Some(Reply::Run { next, .. }) if *next < last => *next += 1,
                _ => {
                    self.tx.pop();
                }
            },
            None => {
                self.tx.pop();
            }
        }
    }

    /// Writes one reply's bytes.
    fn encode(
        &self,
        item: Reply,
        slot: Option<Slot>,
        out: &mut [u8],
    ) -> core::result::Result<usize, Skip> {
        match item {
            Reply::Identity => inquiry::reply_into(&self.identity, out).map_err(Skip::Failed),
            Reply::ControlApp => self.frame(
                Message::ControlAppNotifyResponse {
                    // The manual prints the receive channel as 0 through 16,
                    // with 0 meaning omni, and the transmit channel as 0
                    // through 15. A simulated unit is on one channel, so it
                    // names it both ways.
                    rx_channel: self.channel.number(),
                    tx_channel: self.channel.index(),
                    interface: self.interface,
                    bank: self.selected.bank,
                    program: self.selected.number,
                },
                out,
            ),
            Reply::EditBuffer => self.dump(&self.edit_buffer, None, out),
            Reply::Program(slot) => {
                let program = self.library.program(slot).ok_or(Skip::Empty)?;
                self.dump(&program, Some(slot), out)
            }
            Reply::Run { .. } => {
                let slot = slot.ok_or(Skip::Empty)?;
                let program = self.library.program(slot).ok_or(Skip::Empty)?;
                self.dump(&program, Some(slot), out)
            }
            Reply::ProgramName(slot) => {
                let program = self.library.program(slot).ok_or(Skip::Empty)?;
                let name = program.name();
                let mut raw = [b' '; PROGRAM_NAME_LEN];
                for (slot, byte) in raw.iter_mut().zip(name.as_str().bytes()) {
                    *slot = byte;
                }
                let mut packed_name = [0; packed::packed_len(PROGRAM_NAME_LEN)];
                let packed_len = packed::pack_into(&raw, &mut packed_name).map_err(Skip::Failed)?;
                self.frame(
                    Message::SingleProgramNameDumpResponse {
                        version: program.version(),
                        bank: slot.bank,
                        packed: packed_name.get(..packed_len).unwrap_or_default(),
                    },
                    out,
                )
            }
            Reply::Parameter(edit) => edit.encode_into(self.channel, out).map_err(Skip::Failed),
        }
    }

    /// Writes a program dump, for a stored slot or for the edit buffer.
    fn dump(
        &self,
        program: &Program,
        slot: Option<Slot>,
        out: &mut [u8],
    ) -> core::result::Result<usize, Skip> {
        let mut packed = [0; Program::PACKED_MAX_LEN];
        let packed_len = program.pack_into(&mut packed).map_err(Skip::Failed)?;
        let packed = packed.get(..packed_len).unwrap_or_default();
        let version = program.version();
        let message = match slot {
            Some(slot) => Message::ProgramDumpResponse {
                version,
                bank: slot.bank,
                program: slot.number,
                packed,
            },
            None => Message::EditBufferDumpResponse { version, packed },
        };
        self.frame(message, out)
    }

    /// Frames a message as this unit.
    fn frame(&self, message: Message<'_>, out: &mut [u8]) -> core::result::Result<usize, Skip> {
        Frame::new(self.id, message)
            .encode_into(out)
            .map_err(Skip::Failed)
    }
}

/// Why a queued reply wrote no bytes.
enum Skip {
    /// The library holds nothing for that slot.
    Empty,
    /// The reply could not be built, which is a bug in this module rather than
    /// in what arrived.
    Failed(Error),
}

/// Queues an observation, counting it as lost when there is no room.
fn enqueue<const EV: usize>(queue: &mut Queue<Heard, EV>, lost: &mut u16, heard: Heard) {
    if !queue.push(heard) {
        *lost = lost.saturating_add(1);
    }
}

/// The borrowed state one inbound byte can reach.
struct Inbound<'a, L, const TX: usize, const EV: usize> {
    id: DeviceId,
    channel: Channel,
    heard: &'a mut Queue<Heard, EV>,
    lost: &'a mut u16,
    tx: &'a mut Queue<Reply, TX>,
    edit_buffer: &'a mut Program,
    selected: &'a mut Slot,
    library: &'a L,
    nrpn: &'a mut Nrpn,
}

impl<L: Library, const TX: usize, const EV: usize> Inbound<'_, L, TX, EV> {
    /// Records an observation.
    fn push(&mut self, heard: Heard) {
        enqueue(self.heard, self.lost, heard);
    }

    /// Queues a reply, reporting whether there was room for it.
    fn reply(&mut self, reply: Reply) -> bool {
        self.tx.push(reply)
    }

    /// Takes one decoded MIDI message.
    fn handle(&mut self, event: core::result::Result<WireEvent<'_>, Error>) {
        match event {
            Ok(WireEvent::SysEx(bytes)) => self.sysex(bytes),
            Ok(WireEvent::Channel { channel, message }) => self.channel(channel, message),
            // A unit answers its clock by playing in time, which is not
            // something this models.
            Ok(WireEvent::Realtime(_) | WireEvent::Common(_)) => {}
            Err(error) => self.push(Heard::Failed(error)),
        }
    }

    /// Takes one whole `SysEx` frame.
    fn sysex(&mut self, bytes: &[u8]) {
        // The device inquiry is a universal message rather than a `DeepMind`
        // one, and at six bytes it is shorter than the header every `DeepMind`
        // frame starts with. So it is tried first: reaching it through a parse
        // failure works for the seventeen-byte reply a host reads and not for
        // the request a unit is sent.
        if let Ok(asked) = inquiry::parse_request(bytes) {
            if asked.addresses(self.id) {
                self.reply(Reply::Identity);
                self.push(Heard::Inquiry);
            }
            return;
        }
        match Frame::parse(bytes) {
            Ok(frame) => {
                if frame.device.addresses(self.id) {
                    self.message(&frame.message);
                }
            }
            Err(Error::Foreign {
                manufacturer,
                model,
            }) => self.push(Heard::Foreign {
                manufacturer,
                model,
            }),
            Err(error) => self.push(Heard::Failed(error)),
        }
    }

    /// Takes one `DeepMind` message addressed to this unit.
    fn message(&mut self, message: &Message<'_>) {
        let command = message.command();
        let answered = match *message {
            Message::ControlAppNotifyRequest { .. } => self.reply(Reply::ControlApp),
            Message::EditBufferDumpRequest => self.reply(Reply::EditBuffer),
            Message::ProgramDumpRequest { bank, program } => {
                let slot = Slot {
                    bank,
                    number: program,
                };
                // A slot the library does not hold is not an answer this unit
                // can give, and saying so here is more useful than queuing a
                // reply that writes nothing.
                self.library.program(slot).is_some() && self.reply(Reply::Program(slot))
            }
            Message::SingleProgramNameDumpRequest { bank, program } => {
                let slot = Slot {
                    bank,
                    number: program,
                };
                self.library.program(slot).is_some() && self.reply(Reply::ProgramName(slot))
            }
            Message::ProgramBankDumpRequest { bank, first, last } => {
                first.get() <= last.get()
                    && self.reply(Reply::Run {
                        bank,
                        next: first.get(),
                        last: last.get(),
                    })
            }
            // Payloads the manual gives a length and no layout, and the one
            // reply too long to be worth building. Heard, not answered.
            Message::GlobalParameterDumpRequest
            | Message::SingleUserPatternDumpRequest { .. }
            | Message::EditBufferPatternDumpRequest
            | Message::BankProgramNamesDumpRequest { .. }
            | Message::CalibrationDataDumpRequest
            | Message::ChordMemoryDumpRequest
            | Message::PolyChordMemoryDumpRequest => false,
            // Everything else is a response, which is not something a unit is
            // sent.
            _ => {
                self.push(Heard::Unexpected(command));
                return;
            }
        };
        self.push(Heard::Request { command, answered });
    }

    /// Takes one channel message.
    fn channel(&mut self, channel: Channel, message: ChannelMessage) {
        if channel != self.channel {
            return;
        }
        if let ChannelMessage::ControlChange { controller, value } = message {
            match self.nrpn.control_change(controller, value) {
                Change::Pending => return,
                Change::Parameter {
                    parameter,
                    value,
                    exact,
                } => {
                    self.parameter(parameter, value, exact);
                    return;
                }
                Change::Ignored => {}
            }
        }
        if let ChannelMessage::ProgramChange { program } = message {
            if let Ok(number) = ProgramNumber::new(program) {
                self.selected.number = number;
                // A recall is what the front panel does with a program change,
                // and the sound that comes up is the stored one.
                if let Some(program) = self.library.program(*self.selected) {
                    *self.edit_buffer = program;
                }
            }
        }
        self.push(Heard::Channel { channel, message });
    }

    /// Applies a parameter edit to the edit buffer.
    fn parameter(&mut self, parameter: ParamId, value: u16, exact: bool) {
        if let Ok(byte) = u8::try_from(value) {
            // A value the parameter does not accept is not applied. The
            // synthesizer is the authority on its own range.
            let _ = self.edit_buffer.set(parameter, byte);
        }
        self.push(Heard::Parameter {
            parameter,
            value,
            exact,
        });
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use super::*;
    use crate::device::{Device, Event};
    use crate::ids::ProtocolVersion;
    use crate::param::{DATA_ENTRY_LSB, DATA_ENTRY_MSB, NRPN_NUMBER_LSB, NRPN_NUMBER_MSB};
    use crate::program::ProgramName;

    /// A unit whose memory is exactly the slots a test names.
    struct Slots<const N: usize>([(Slot, Program); N]);

    impl<const N: usize> Library for Slots<N> {
        fn program(&self, slot: Slot) -> Option<Program> {
            self.0
                .iter()
                .find(|(at, _)| *at == slot)
                .map(|(_, program)| program.clone())
        }
    }

    /// The second bank, which only `new` names.
    fn bank_b() -> Bank {
        Bank::new(1).expect("in range")
    }

    fn named(name: &str) -> Program {
        let mut program = Program::new(ProtocolVersion::V7);
        program.set_name(ProgramName::new(name).expect("a legal name"));
        program
    }

    fn synth() -> Synth {
        Synth::new(DeviceId::Unit(0), named("Edit Buffer"))
    }

    /// Collects everything the unit has to say, one message per entry.
    fn drained<L: Library, const RX: usize, const TX: usize, const EV: usize>(
        synth: &mut Synth<L, RX, TX, EV>,
    ) -> alloc::vec::Vec<alloc::vec::Vec<u8>> {
        let mut sent = alloc::vec::Vec::new();
        synth
            .drain_tx(|bytes| {
                sent.push(bytes.to_vec());
                Ok::<(), core::convert::Infallible>(())
            })
            .expect("the collector never refuses");
        sent
    }

    /// Sends `bytes` to the unit and returns what it says back.
    fn exchange<L: Library, const RX: usize, const TX: usize, const EV: usize>(
        synth: &mut Synth<L, RX, TX, EV>,
        bytes: &[u8],
    ) -> alloc::vec::Vec<alloc::vec::Vec<u8>> {
        synth.feed(bytes);
        drained(synth)
    }

    /// Frames a message for the unit under test.
    fn ask(message: Message<'_>) -> alloc::vec::Vec<u8> {
        let mut bytes = [0; MAX_REPLY_LEN];
        let written = Frame::new(DeviceId::Unit(0), message)
            .encode_into(&mut bytes)
            .expect("the buffer fits");
        bytes[..written].to_vec()
    }

    #[test]
    fn an_inquiry_is_answered_with_the_identity() {
        let mut synth = synth();
        let mut request = [0; inquiry::REQUEST_LEN];
        inquiry::request_into(DeviceId::Broadcast, &mut request).expect("the buffer fits");

        let sent = exchange(&mut synth, &request);

        assert_eq!(sent.len(), 1);
        assert_eq!(
            inquiry::parse_reply(&sent[0]),
            Ok(Identity {
                device: DeviceId::Unit(0),
                firmware: Version { major: 1, minor: 1 },
                voice: Version { major: 1, minor: 0 },
            })
        );
        assert_eq!(synth.poll_heard(), Some(Heard::Inquiry));
    }

    #[test]
    fn the_edit_buffer_is_answered_with_what_it_holds() {
        let mut synth = synth();
        let sent = exchange(&mut synth, &ask(Message::EditBufferDumpRequest));

        assert_eq!(sent.len(), 1);
        let frame = Frame::parse(&sent[0]).expect("a DeepMind frame");
        let program = Program::from_dump(&frame.message).expect("an edit buffer dump");
        assert_eq!(program.name().as_str(), "Edit Buffer");
        assert_eq!(
            synth.poll_heard(),
            Some(Heard::Request {
                command: Command::EditBufferDumpRequest,
                answered: true,
            })
        );
    }

    #[test]
    fn a_frame_for_another_unit_is_not_this_units_business() {
        let mut synth = synth();
        let mut bytes = [0; MAX_REPLY_LEN];
        let written = Frame::new(DeviceId::Unit(3), Message::EditBufferDumpRequest)
            .encode_into(&mut bytes)
            .expect("the buffer fits");

        let sent = exchange(&mut synth, &bytes[..written]);

        assert!(sent.is_empty());
        assert_eq!(synth.poll_heard(), None);
    }

    #[test]
    fn a_broadcast_frame_reaches_every_unit() {
        let mut synth = Synth::<Empty>::new(DeviceId::Unit(7), named("Edit Buffer"));
        let mut bytes = [0; MAX_REPLY_LEN];
        let written = Frame::new(DeviceId::Broadcast, Message::EditBufferDumpRequest)
            .encode_into(&mut bytes)
            .expect("the buffer fits");

        let sent = exchange(&mut synth, &bytes[..written]);

        assert_eq!(sent.len(), 1);
        // It answers as itself, not as the address it was reached at.
        let frame = Frame::parse(&sent[0]).expect("a DeepMind frame");
        assert_eq!(frame.device, DeviceId::Unit(7));
    }

    #[test]
    fn a_stored_program_comes_from_the_library() {
        let stored = named("Stored");
        let slot = Slot::new(Bank::A, ProgramNumber::FIRST);
        let mut synth: Synth<Slots<1>> = Synth::with_library(
            DeviceId::Unit(0),
            named("Edit Buffer"),
            Slots([(slot, stored)]),
        );

        let sent = exchange(
            &mut synth,
            &ask(Message::ProgramDumpRequest {
                bank: slot.bank,
                program: slot.number,
            }),
        );

        assert_eq!(sent.len(), 1);
        let frame = Frame::parse(&sent[0]).expect("a DeepMind frame");
        assert!(matches!(
            frame.message,
            Message::ProgramDumpResponse { bank, program, .. } if bank == slot.bank && program == slot.number
        ));
        assert_eq!(
            Program::from_dump(&frame.message)
                .expect("a program dump")
                .name()
                .as_str(),
            "Stored"
        );
    }

    #[test]
    fn a_slot_the_library_does_not_hold_goes_unanswered() {
        let mut synth = synth();
        let sent = exchange(
            &mut synth,
            &ask(Message::ProgramDumpRequest {
                bank: Bank::A,
                program: ProgramNumber::FIRST,
            }),
        );

        assert!(sent.is_empty());
        assert_eq!(
            synth.poll_heard(),
            Some(Heard::Request {
                command: Command::ProgramDumpRequest,
                answered: false,
            })
        );
    }

    #[test]
    fn a_bank_run_sends_one_dump_per_slot() {
        let slots: [(Slot, Program); 3] = core::array::from_fn(|index| {
            let number = ProgramNumber::new(u8::try_from(index).unwrap_or(0)).expect("in range");
            (Slot::new(bank_b(), number), named("Run"))
        });
        let mut synth: Synth<Slots<3>> =
            Synth::with_library(DeviceId::Unit(0), named("Edit Buffer"), Slots(slots));

        let sent = exchange(
            &mut synth,
            &ask(Message::ProgramBankDumpRequest {
                bank: bank_b(),
                first: ProgramNumber::FIRST,
                last: ProgramNumber::new(2).expect("in range"),
            }),
        );

        assert_eq!(sent.len(), 3);
        for (index, bytes) in sent.iter().enumerate() {
            let frame = Frame::parse(bytes).expect("a DeepMind frame");
            let Message::ProgramDumpResponse { bank, program, .. } = frame.message else {
                panic!("a run is program dumps");
            };
            assert_eq!(bank, bank_b());
            assert_eq!(usize::from(program.get()), index);
        }
        // One request, one item on the queue, however many dumps it became.
        assert_eq!(synth.queued(), 0);
    }

    /// A port that stops taking bytes halfway through a bank has to be able to
    /// come back to where it was, or the host is sent the run twice.
    #[test]
    fn a_run_refused_halfway_resumes_where_it_stopped() {
        let slots: [(Slot, Program); 4] = core::array::from_fn(|index| {
            let number = ProgramNumber::new(u8::try_from(index).unwrap_or(0)).expect("in range");
            (Slot::new(Bank::A, number), named("Run"))
        });
        let mut synth: Synth<Slots<4>> =
            Synth::with_library(DeviceId::Unit(0), named("Edit Buffer"), Slots(slots));

        synth.feed(&ask(Message::ProgramBankDumpRequest {
            bank: Bank::A,
            first: ProgramNumber::FIRST,
            last: ProgramNumber::new(3).expect("in range"),
        }));

        // A port with room for two.
        let mut taken = alloc::vec::Vec::new();
        let refused = synth.drain_tx(|bytes| {
            if taken.len() == 2 {
                return Err("full");
            }
            taken.push(bytes.to_vec());
            Ok(())
        });
        assert_eq!(refused, Err("full"));
        assert_eq!(taken.len(), 2);

        // The rest, once it has room again.
        let rest = drained(&mut synth);
        assert_eq!(rest.len(), 2);

        let numbers: alloc::vec::Vec<u8> = taken
            .iter()
            .chain(rest.iter())
            .map(|bytes| {
                let frame = Frame::parse(bytes).expect("a DeepMind frame");
                let Message::ProgramDumpResponse { program, .. } = frame.message else {
                    panic!("a run is program dumps");
                };
                program.get()
            })
            .collect();
        assert_eq!(numbers, [0, 1, 2, 3]);
    }

    #[test]
    fn a_run_skips_the_slots_the_library_does_not_hold() {
        let slot = Slot::new(Bank::A, ProgramNumber::new(1).expect("in range"));
        let mut synth: Synth<Slots<1>> = Synth::with_library(
            DeviceId::Unit(0),
            named("Edit Buffer"),
            Slots([(slot, named("Only One"))]),
        );

        let sent = exchange(
            &mut synth,
            &ask(Message::ProgramBankDumpRequest {
                bank: Bank::A,
                first: ProgramNumber::FIRST,
                last: ProgramNumber::new(3).expect("in range"),
            }),
        );

        assert_eq!(sent.len(), 1);
        let frame = Frame::parse(&sent[0]).expect("a DeepMind frame");
        let Message::ProgramDumpResponse { program, .. } = frame.message else {
            panic!("a run is program dumps");
        };
        assert_eq!(program.get(), 1);
    }

    #[test]
    fn an_nrpn_moves_the_edit_buffer() {
        let mut synth = synth();
        let edit = NrpnEdit::new(ParamId::Lfo1Rate, 100).expect("a value it accepts");
        let mut bytes = [0; NrpnEdit::LEN];
        let written = edit
            .encode_into(Channel::ONE, &mut bytes)
            .expect("the buffer fits");

        synth.feed(&bytes[..written]);

        assert_eq!(synth.edit_buffer().get(ParamId::Lfo1Rate), 100);
        assert_eq!(
            synth.poll_heard(),
            Some(Heard::Parameter {
                parameter: ParamId::Lfo1Rate,
                value: 100,
                exact: true,
            })
        );
    }

    #[test]
    fn an_edit_on_another_channel_is_not_this_units() {
        let mut synth = synth();
        let edit = NrpnEdit::new(ParamId::Lfo1Rate, 100).expect("a value it accepts");
        let mut bytes = [0; NrpnEdit::LEN];
        let channel = Channel::new(5).expect("a legal channel");
        let written = edit
            .encode_into(channel, &mut bytes)
            .expect("the buffer fits");

        synth.feed(&bytes[..written]);

        assert_eq!(synth.edit_buffer().get(ParamId::Lfo1Rate), 0);
        assert_eq!(synth.poll_heard(), None);
    }

    #[test]
    fn a_controller_carries_seven_bits_of_its_parameter() {
        let mut synth = synth();
        let Some(controller) = ParamId::Lfo1Rate.controller() else {
            panic!("LFO 1 rate has a controller");
        };
        let mut bytes = [0; 3];
        let written = ChannelMessage::ControlChange {
            controller: controller.cc,
            value: 127,
        }
        .encode_into(Channel::ONE, &mut bytes)
        .expect("the buffer fits");

        synth.feed(&bytes[..written]);

        assert_eq!(
            synth.poll_heard(),
            Some(Heard::Parameter {
                parameter: ParamId::Lfo1Rate,
                value: ParamId::Lfo1Rate.from_cc_value(127),
                exact: false,
            })
        );
    }

    #[test]
    fn a_program_change_recalls_the_stored_sound() {
        let slot = Slot::new(Bank::A, ProgramNumber::new(4).expect("in range"));
        let mut synth: Synth<Slots<1>> = Synth::with_library(
            DeviceId::Unit(0),
            named("Edit Buffer"),
            Slots([(slot, named("Recalled"))]),
        );

        let mut bytes = [0; 2];
        let written = ChannelMessage::ProgramChange { program: 4 }
            .encode_into(Channel::ONE, &mut bytes)
            .expect("the buffer fits");
        synth.feed(&bytes[..written]);

        assert_eq!(synth.edit_buffer().name().as_str(), "Recalled");
        assert_eq!(synth.selected(), slot);
    }

    #[test]
    fn turning_a_knob_sends_what_it_changed() {
        let mut synth = synth();
        synth
            .turn(ParamId::Lfo1Rate, 64)
            .expect("a value it accepts, with room to queue it");

        assert_eq!(synth.edit_buffer().get(ParamId::Lfo1Rate), 64);

        let sent = drained(&mut synth);
        assert_eq!(sent.len(), 1);
        assert_eq!(
            sent[0],
            [
                0xB0,
                NRPN_NUMBER_MSB,
                0,
                0xB0,
                NRPN_NUMBER_LSB,
                ParamId::Lfo1Rate.offset(),
                0xB0,
                DATA_ENTRY_MSB,
                0,
                0xB0,
                DATA_ENTRY_LSB,
                64,
            ]
        );
    }

    #[test]
    fn a_knob_turned_past_the_range_moves_nothing() {
        let mut synth = synth();
        let before = synth.edit_buffer().clone();

        assert!(synth.turn(ParamId::Lfo1Rate, 5_000).is_err());

        assert_eq!(synth.edit_buffer(), &before);
        assert_eq!(synth.queued(), 0);
    }

    #[test]
    fn a_payload_the_manual_never_describes_is_heard_and_not_answered() {
        let mut synth = synth();
        let sent = exchange(&mut synth, &ask(Message::GlobalParameterDumpRequest));

        assert!(sent.is_empty());
        assert_eq!(
            synth.poll_heard(),
            Some(Heard::Request {
                command: Command::GlobalParameterDumpRequest,
                answered: false,
            })
        );
    }

    /// Nothing documented writes a program into a synthesizer, so a dump
    /// arriving is reported rather than applied.
    #[test]
    fn a_dump_sent_to_a_unit_is_unexpected() {
        let mut synth = synth();
        let program = named("Not Mine");
        let mut packed = [0; Program::PACKED_MAX_LEN];
        let packed_len = program.pack_into(&mut packed).expect("the buffer fits");

        let sent = exchange(
            &mut synth,
            &ask(Message::EditBufferDumpResponse {
                version: program.version(),
                packed: &packed[..packed_len],
            }),
        );

        assert!(sent.is_empty());
        assert_eq!(
            synth.poll_heard(),
            Some(Heard::Unexpected(Command::EditBufferDumpResponse))
        );
        assert_eq!(synth.edit_buffer().name().as_str(), "Edit Buffer");
    }

    #[test]
    fn a_control_app_notify_is_answered_with_the_selection() {
        let slot = Slot::new(
            Bank::new(2).expect("in range"),
            ProgramNumber::new(9).expect("in range"),
        );
        let mut synth = Synth::<Empty>::new(DeviceId::Unit(0), named("Edit Buffer"))
            .with_channel(Channel::new(3).expect("a legal channel"))
            .with_interface(Interface::Usb)
            .with_selected(slot);

        let sent = exchange(
            &mut synth,
            &ask(Message::ControlAppNotifyRequest { reserved: 0 }),
        );

        assert_eq!(sent.len(), 1);
        let frame = Frame::parse(&sent[0]).expect("a DeepMind frame");
        assert_eq!(
            frame.message,
            Message::ControlAppNotifyResponse {
                rx_channel: 4,
                tx_channel: 3,
                interface: Interface::Usb,
                bank: slot.bank,
                program: slot.number,
            }
        );
    }

    #[test]
    fn a_name_request_is_answered_with_the_name() {
        let slot = Slot::new(Bank::A, ProgramNumber::FIRST);
        let mut synth: Synth<Slots<1>> = Synth::with_library(
            DeviceId::Unit(0),
            named("Edit Buffer"),
            Slots([(slot, named("Bass Sweep"))]),
        );

        let sent = exchange(
            &mut synth,
            &ask(Message::SingleProgramNameDumpRequest {
                bank: slot.bank,
                program: slot.number,
            }),
        );

        assert_eq!(sent.len(), 1);
        let frame = Frame::parse(&sent[0]).expect("a DeepMind frame");
        let Message::SingleProgramNameDumpResponse { packed, .. } = frame.message else {
            panic!("a name request is answered with a name");
        };
        // Packing pads the last group, so sixteen bytes come back as twenty-one.
        let mut raw = [0; 21];
        packed::unpack_into(packed, &mut raw).expect("the buffer fits");
        assert_eq!(
            core::str::from_utf8(&raw[..PROGRAM_NAME_LEN]),
            Ok("Bass Sweep      ")
        );
    }

    #[test]
    fn a_foreign_frame_is_reported_rather_than_answered() {
        let mut synth = synth();
        // Roland's manufacturer ID, which is not this library's business.
        let sent = exchange(
            &mut synth,
            &[0xF0, 0x41, 0x10, 0x00, 0x0C, 0x12, 0x00, 0x00, 0xF7],
        );

        assert!(sent.is_empty());
        assert!(matches!(synth.poll_heard(), Some(Heard::Foreign { .. })));
    }

    #[test]
    fn observations_dropped_when_the_queue_is_full_are_counted() {
        let mut synth: Synth<Empty, MAX_SYSEX_LEN, DEFAULT_REPLY_DEPTH, 2> =
            Synth::with_library(DeviceId::Unit(0), named("Edit Buffer"), Empty);

        for _ in 0..4 {
            synth.feed(&ask(Message::GlobalParameterDumpRequest));
        }

        assert!(matches!(synth.poll_heard(), Some(Heard::Request { .. })));
        assert!(matches!(synth.poll_heard(), Some(Heard::Request { .. })));
        assert_eq!(synth.poll_heard(), Some(Heard::Lost(2)));
        assert_eq!(synth.poll_heard(), None);
    }

    #[test]
    fn resetting_drops_what_was_waiting() {
        let mut synth = synth();
        synth.feed(&ask(Message::EditBufferDumpRequest));
        assert_eq!(synth.queued(), 1);

        synth.reset();

        assert_eq!(synth.queued(), 0);
        assert_eq!(synth.poll_heard(), None);
        assert!(drained(&mut synth).is_empty());
    }

    /// The reason this module exists: a host and a unit, talking.
    #[test]
    fn a_host_reads_what_the_unit_holds_and_the_unit_hears_the_edit() {
        let mut synth = synth();
        let mut host: Device = Device::new(DeviceId::Unit(0));

        host.request_edit_buffer().expect("room to queue it");
        host.drain_tx(|bytes| {
            synth.feed(bytes);
            Ok::<(), core::convert::Infallible>(())
        })
        .expect("the collector never refuses");
        synth
            .drain_tx(|bytes| {
                host.feed(bytes);
                Ok::<(), core::convert::Infallible>(())
            })
            .expect("the collector never refuses");

        assert!(matches!(host.poll_event(), Some(Event::EditBuffer(_))));
        assert!(host.program().is_confirmed());

        host.edit(|program| program.set_lfo1_rate(64))
            .expect("a program is known and the value fits");
        host.drain_tx(|bytes| {
            synth.feed(bytes);
            Ok::<(), core::convert::Infallible>(())
        })
        .expect("the collector never refuses");

        assert_eq!(synth.edit_buffer().get(ParamId::Lfo1Rate), 64);
    }
}
