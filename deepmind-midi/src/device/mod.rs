//! The state machine a host drives: what the synthesizer holds, and what to send.
//!
//! This is where the layers below meet. Bytes off a port go in, a dump becomes a
//! confirmed [`Program`], an edit becomes the NRPN messages that carry it, a
//! request that goes unanswered becomes a [`Event::Timeout`], and bytes for the
//! port come out. It is also the first layer with a clock, in the sense that the
//! host supplies one.
//!
//! ```
//! use deepmind_midi::device::{Device, Event};
//! use deepmind_midi::ids::{DeviceId, ProtocolVersion};
//! use deepmind_midi::program::{Program, ProgramName};
//! use deepmind_midi::sysex::{Frame, Message};
//!
//! let mut device: Device = Device::new(DeviceId::Unit(0));
//!
//! // Ask for the edit buffer. The bytes wait until the host takes them.
//! device.request_edit_buffer()?;
//! let mut asked = [0; 8];
//! device.drain_tx(|bytes| {
//!     asked.get_mut(..bytes.len()).map(|slot| slot.copy_from_slice(bytes));
//!     Ok::<(), core::convert::Infallible>(())
//! }).ok();
//! assert_eq!(asked, [0xF0, 0x00, 0x20, 0x32, 0x20, 0x00, 0x03, 0xF7]);
//!
//! // What comes back, in whatever chunks the port delivers it.
//! let mut program = Program::new(ProtocolVersion::V6)?;
//! program.set_name(ProgramName::new("Bass Sweep")?);
//! let mut packed = [0; Program::PACKED_MAX_LEN];
//! let packed_len = program.pack_into(&mut packed)?;
//! let mut reply = [0; 320];
//! let reply_len = Frame::new(
//!     DeviceId::Unit(0),
//!     Message::EditBufferDumpResponse {
//!         version: program.version(),
//!         packed: packed.get(..packed_len).unwrap_or_default(),
//!     },
//! )
//! .encode_into(&mut reply)?;
//! device.feed(reply.get(..reply_len).unwrap_or_default());
//!
//! assert!(matches!(device.poll_event(), Some(Event::EditBuffer(_))));
//! assert!(device.program().is_confirmed());
//!
//! // Change one thing. The library works out which message that implies.
//! assert_eq!(device.edit(|program| program.set_lfo1_rate(64))?, 1);
//! assert!(device.program().is_assumed());
//! # Ok::<(), deepmind_midi::Error>(())
//! ```
//!
//! # The loop a host writes
//!
//! Four calls, in whatever order suits the host:
//!
//! ```text
//! device.feed(&bytes_from_port);                 // inbound, any chunking
//! device.tick(now_ms);                           // the host owns the clock
//! device.drain_tx(|bytes| port.send(bytes))?;    // outbound
//! while let Some(event) = device.poll_event() {} // results
//! ```
//!
//! No IO, no threads, no blocking and no clock of its own. A fake clock and a
//! byte vector reproduce any timing bug exactly, which is the point: the
//! hardware-dependent tests are the ones that rot.
//!
//! # Nothing is sent until it is drained
//!
//! Queuing is not sending. [`request_edit_buffer`](Device::request_edit_buffer)
//! and its neighbours put an item in the outbound queue and return; the bytes
//! exist when [`drain_tx`](Device::drain_tx) hands them to the host, and the
//! timeout on a request starts then rather than when it was queued. A host that
//! never drains never times out, which is the truthful answer for a request
//! that never went anywhere.
//!
//! The queue holds items, not bytes: a request is a few bytes and a whole-program
//! edit is 242 of them, so the cost is what is waiting rather than the worst case
//! of what might be.
//!
//! # State is a set of claims
//!
//! [`Known`] is what the synthesizer is believed to hold and how that belief was
//! come by. The synthesizer answers no per-parameter reads, so there are exactly
//! two ways to believe something about it: it said so, or the host told it so
//! and nothing has contradicted that. [`Device::program`] says which.
//!
//! An edit makes the tracked program [`Known::Assumed`]. After sending edits, an
//! edit buffer dump is the only thing that turns it back into
//! [`Known::Confirmed`], and this layer never asks for one on its own: whether a
//! resync is worth its latency is the host's call, not the library's.
//!
//! # What arrives from the synthesizer
//!
//! Turning a knob on the front panel sends the parameter out, so inbound traffic
//! changes the tracked program as well as outbound traffic does. An NRPN carries
//! the parameter's whole value and the tracked program stays confirmed; a
//! control change carries seven bits of it, which for most parameters is less
//! than the value has, so applying one leaves the program assumed. Either way
//! [`Event::Parameter`] reports it.
//!
//! Real-time bytes are dropped rather than queued. A running MIDI clock is
//! twenty-four messages a beat, none of which says anything about what the
//! synthesizer holds, and queuing them would starve the queue of the events that
//! do. A host that wants everything on the port drives [`Decoder`] itself, which
//! is what it is for.
//!
//! # Sizing
//!
//! Three capacities, all const parameters with defaults, because the right
//! numbers differ by two orders of magnitude between a desktop host and a
//! microcontroller:
//!
//! | Parameter | Default | What it bounds |
//! |---|---|---|
//! | `RX` | [`MAX_SYSEX_LEN`] | The longest `SysEx` frame that can be reassembled |
//! | `TX` | [`DEFAULT_TX_DEPTH`] | Items waiting to be sent; a whole-program edit is 242 |
//! | `EV` | [`DEFAULT_EVENT_DEPTH`] | Events waiting to be polled |
//!
//! ```
//! use deepmind_midi::device::Device;
//! use deepmind_midi::ids::DeviceId;
//!
//! // A host that sends single edits and reads the edit buffer needs none of the
//! // room a bank transfer wants.
//! let device: Device<300, 8, 2> = Device::new(DeviceId::Unit(0));
//! ```
//!
//! An event that does not fit is dropped and counted, and the count arrives as
//! [`Event::Lost`] once the queue has room again. Nothing is dropped quietly.

mod event;
mod known;
mod request;

pub use event::Event;
pub use known::Known;
pub use request::{MAX_OUTBOUND_LEN, Request};

use request::Outbound;

use crate::error::{Error, Result};
use crate::ids::{Bank, DeviceId, ProgramNumber, Slot};
use crate::param::{
    Controller, DATA_ENTRY_LSB, DATA_ENTRY_MSB, DEFAULT_FIRMWARE, NRPN_NUMBER_LSB, NRPN_NUMBER_MSB,
    ParamId,
};
use crate::program::Program;
use crate::queue::Queue;
use crate::sysex::inquiry::{self, Version};
use crate::sysex::{Frame, Identity, Message};
use crate::wire::{Channel, ChannelMessage, Decoder, Event as WireEvent, MAX_SYSEX_LEN};

/// Outbound items a [`Device`] holds by default.
///
/// Enough for a whole-program edit, which is 242 of them, and a few requests
/// beside it.
pub const DEFAULT_TX_DEPTH: usize = 256;

/// Events a [`Device`] holds by default.
pub const DEFAULT_EVENT_DEPTH: usize = 8;

/// Milliseconds a request waits for its answer before timing out.
///
/// A dump crosses a MIDI port in a few tens of milliseconds; this is generous
/// rather than tight, because a timeout that fires on a slow port is worse than
/// one that fires late. [`Device::with_timeout`] sets another.
pub const DEFAULT_TIMEOUT_MS: u64 = 2_000;

/// Requests that can be outstanding at once and still be timed.
///
/// A host waiting on more than this many different answers at the same time is
/// not really waiting on any of them. Beyond it, requests are still sent; they
/// are simply not timed.
pub const MAX_PENDING: usize = 8;

/// A request that has gone out and is waiting for its answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Pending {
    request: Request,
    sent_at: u64,
}

/// The inbound NRPN registers, which the synthesizer keeps too.
///
/// A parameter is selected by a pair of controllers and then written by another
/// pair, and the selection stays in force, so a knob sweep is one selection and
/// a run of values.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Nrpn {
    number_msb: Option<u8>,
    number_lsb: Option<u8>,
    data_msb: Option<u8>,
}

impl Nrpn {
    /// Returns the parameter currently selected, where one is and it names one.
    fn parameter(self) -> Option<ParamId> {
        let (msb, lsb) = (self.number_msb?, self.number_lsb?);
        let number = (u16::from(msb) << 7) | u16::from(lsb);
        ParamId::from_offset(u8::try_from(number).ok()?).ok()
    }
}

/// One synthesizer, tracked.
///
/// See the [module documentation](self) for the loop a host writes around it and
/// for what the three capacities bound.
#[derive(Debug)]
pub struct Device<
    const RX: usize = MAX_SYSEX_LEN,
    const TX: usize = DEFAULT_TX_DEPTH,
    const EV: usize = DEFAULT_EVENT_DEPTH,
> {
    id: DeviceId,
    channel: Channel,
    now: u64,
    timeout: u64,
    decoder: Decoder<RX>,
    tx: Queue<Outbound, TX>,
    events: Queue<Event, EV>,
    lost: u16,
    pending: [Option<Pending>; MAX_PENDING],
    program: Known<Program>,
    identity: Known<Identity>,
    nrpn: Nrpn,
}

impl<const RX: usize, const TX: usize, const EV: usize> Device<RX, TX, EV> {
    /// Builds a device that talks to `id` on MIDI channel 1.
    ///
    /// [`DeviceId::Broadcast`] reaches every unit on the port and accepts what
    /// any of them answers, which is how a host finds a synthesizer whose device
    /// ID it does not know yet.
    #[must_use]
    pub const fn new(id: DeviceId) -> Self {
        Self {
            id,
            channel: Channel::ONE,
            now: 0,
            timeout: DEFAULT_TIMEOUT_MS,
            decoder: Decoder::new(),
            tx: Queue::new(),
            events: Queue::new(),
            lost: 0,
            pending: [const { None }; MAX_PENDING],
            program: Known::Unknown,
            identity: Known::Unknown,
            nrpn: Nrpn {
                number_msb: None,
                number_lsb: None,
                data_msb: None,
            },
        }
    }

    /// Sets the MIDI channel edits are sent on and parameter traffic is read on.
    #[must_use]
    pub const fn with_channel(mut self, channel: Channel) -> Self {
        self.channel = channel;
        self
    }

    /// Sets how long a request waits for its answer, in milliseconds.
    ///
    /// Zero times a request out at the first [`tick`](Device::tick) after it is
    /// sent, which is a way of turning the mechanism off rather than a way of
    /// making it strict.
    #[must_use]
    pub const fn with_timeout(mut self, milliseconds: u64) -> Self {
        self.timeout = milliseconds;
        self
    }

    /// Returns the unit this device talks to.
    #[must_use]
    pub const fn id(&self) -> DeviceId {
        self.id
    }

    /// Returns the MIDI channel it talks on.
    #[must_use]
    pub const fn channel(&self) -> Channel {
        self.channel
    }

    /// Returns the host clock reading of the last [`tick`](Device::tick).
    #[must_use]
    pub const fn now(&self) -> u64 {
        self.now
    }

    /// Returns the edit buffer as the library believes it to be.
    ///
    /// [`Known::Confirmed`] means the synthesizer sent it and nothing lossy has
    /// been applied since. [`Known::Assumed`] means the host has edited it, or a
    /// control change has moved a parameter by seven bits of resolution.
    #[must_use]
    pub const fn program(&self) -> &Known<Program> {
        &self.program
    }

    /// Returns what a device inquiry answered, if one has.
    #[must_use]
    pub const fn identity(&self) -> &Known<Identity> {
        &self.identity
    }

    /// Returns the firmware version to read value tables against.
    ///
    /// What the synthesizer reported, or [`DEFAULT_FIRMWARE`] where it has not
    /// been asked. Three value tables were renumbered by firmware 1.1, so this
    /// is the difference between naming a modulation source and misnaming it.
    #[must_use]
    pub fn firmware(&self) -> Version {
        self.identity
            .value()
            .map_or(DEFAULT_FIRMWARE, |identity| identity.firmware)
    }

    /// Returns how long a request waits for its answer.
    ///
    /// [`DEFAULT_TIMEOUT_MS`] unless [`with_timeout`](Device::with_timeout) said
    /// otherwise. A driver loop around this reads it to know how long waiting
    /// can reasonably last.
    #[must_use]
    pub const fn timeout(&self) -> u64 {
        self.timeout
    }

    /// Returns the requests that have been sent and not yet answered.
    pub fn outstanding(&self) -> impl Iterator<Item = Request> + '_ {
        self.pending
            .iter()
            .filter_map(|slot| slot.map(|pending| pending.request))
    }

    /// Returns how many items are waiting to be sent.
    #[must_use]
    pub const fn queued(&self) -> usize {
        self.tx.len()
    }

    /// Returns how many more items the outbound queue holds.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.tx.remaining()
    }

    /// Forgets everything: the part-decoded frame, both queues, what was asked
    /// for and what was believed.
    ///
    /// What a host does when it reopens the port, since none of the above
    /// survives that.
    pub fn reset(&mut self) {
        self.decoder.reset();
        self.tx.clear();
        self.events.clear();
        self.lost = 0;
        self.pending = [const { None }; MAX_PENDING];
        self.program = Known::Unknown;
        self.identity = Known::Unknown;
        self.nrpn = Nrpn::default();
    }

    /// Seeds the tracked program with one the host already has.
    ///
    /// Assumed, not confirmed: a program read from a file says what a host wants
    /// the synthesizer to hold, which is exactly the claim
    /// [`Known::Assumed`] makes. It sends nothing; a following
    /// [`edit`](Device::edit) diffs against it.
    pub fn assume_program(&mut self, program: Program) {
        self.program = Known::Assumed {
            value: program,
            sent_at: self.now,
        };
    }

    /// Advances the clock and raises a timeout for anything now overdue.
    ///
    /// The host owns the clock. Milliseconds, monotonic; a reading behind the
    /// last one is ignored rather than treated as time passing backwards.
    pub fn tick(&mut self, now_ms: u64) {
        if now_ms < self.now {
            return;
        }
        self.now = now_ms;
        let Self {
            timeout,
            pending,
            events,
            lost,
            ..
        } = self;
        for slot in pending.iter_mut() {
            let overdue =
                slot.is_some_and(|entry| now_ms.saturating_sub(entry.sent_at) >= *timeout);
            if !overdue {
                continue;
            }
            if let Some(entry) = slot.take() {
                enqueue(events, lost, Event::Timeout(entry.request));
            }
        }
    }

    /// Takes the oldest event, if there is one.
    pub fn poll_event(&mut self) -> Option<Event> {
        if let Some(event) = self.events.pop() {
            return Some(event);
        }
        let lost = core::mem::take(&mut self.lost);
        (lost > 0).then_some(Event::Lost(lost))
    }

    /// Feeds inbound bytes, in whatever chunks they arrived in.
    ///
    /// Decodes them, updates what is known, and queues what happened.
    pub fn feed(&mut self, bytes: &[u8]) {
        let Self {
            id,
            channel,
            now,
            decoder,
            events,
            lost,
            pending,
            program,
            identity,
            nrpn,
            ..
        } = self;
        let mut inbound = Inbound {
            id: *id,
            channel: *channel,
            now: *now,
            events,
            lost,
            pending,
            program,
            identity,
            nrpn,
        };
        decoder.feed(bytes, |event| inbound.handle(event));
    }

    /// Hands every queued item to `send`, oldest first, and returns how many
    /// went.
    ///
    /// One call per message, which is the granularity a MIDI port wants. A host
    /// that would rather make fewer, larger writes buffers inside the closure.
    ///
    /// A request becomes outstanding as its bytes go out, which is when its
    /// timeout starts. An item `send` refuses stays queued, along with
    /// everything behind it, so a port that blocks costs nothing but latency.
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
            let mut bytes = [0; MAX_OUTBOUND_LEN];
            let written = match item.encode_into(self.id, self.channel, &mut bytes) {
                Ok(written) => written,
                Err(error) => {
                    self.tx.pop();
                    enqueue(&mut self.events, &mut self.lost, Event::Failed(error));
                    continue;
                }
            };
            send(bytes.get(..written).unwrap_or_default())?;
            self.tx.pop();
            if let Some(request) = item.request() {
                self.start(request);
            }
            count += 1;
        }
        Ok(count)
    }

    /// Queues a device inquiry, which is the only thing that reports firmware.
    ///
    /// # Errors
    ///
    /// Returns [`Error::QueueFull`] when the outbound queue has no room.
    pub fn request_identity(&mut self) -> Result<()> {
        self.ask(Request::Identity)
    }

    /// Queues a request for the edit buffer, the program as it currently sounds.
    ///
    /// # Errors
    ///
    /// Returns [`Error::QueueFull`] when the outbound queue has no room.
    pub fn request_edit_buffer(&mut self) -> Result<()> {
        self.ask(Request::EditBuffer)
    }

    /// Queues a request for one stored program.
    ///
    /// # Errors
    ///
    /// Returns [`Error::QueueFull`] when the outbound queue has no room.
    pub fn request_program(&mut self, slot: Slot) -> Result<()> {
        self.ask(Request::Program(slot))
    }

    /// Queues a request for a run of stored programs, answered one dump at a
    /// time.
    ///
    /// Every dump arrives as its own [`Event::Program`], so a host reading a
    /// whole bank drains events as they come rather than waiting for all 128.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ProgramOutOfRange`] when `last` is before `first`, and
    /// [`Error::QueueFull`] when the outbound queue has no room.
    pub fn request_bank(
        &mut self,
        bank: Bank,
        first: ProgramNumber,
        last: ProgramNumber,
    ) -> Result<()> {
        if last.get() < first.get() {
            return Err(Error::ProgramOutOfRange(last.get()));
        }
        self.ask(Request::Bank { bank, first, last })
    }

    /// Edits the tracked program and queues the messages that carry the change.
    ///
    /// The closure is handed the program as the synthesizer is believed to hold
    /// it. What it changes is what gets sent: the diff is
    /// [`Program::changes`], so two writes to one parameter inside one closure
    /// are one message, and a write that puts a parameter back where it was is
    /// none. Returns how many parameters changed.
    ///
    /// Afterwards the tracked program is [`Known::Assumed`]. Nothing confirms it
    /// but a dump, and asking for one is the host's call.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ProgramNotKnown`] when no program is tracked yet, since
    /// there is then nothing to diff against - ask for the edit buffer, or seed
    /// one with [`assume_program`](Device::assume_program).
    ///
    /// Returns [`Error::ValueOutOfRange`] when the closure left a parameter
    /// holding something it does not accept, and [`Error::QueueFull`] when the
    /// change needs more room than the outbound queue has. Either way nothing is
    /// queued and the tracked program is left as it was: a half-sent edit is a
    /// sound nobody asked for.
    pub fn edit<F>(&mut self, edit: F) -> Result<usize>
    where
        F: FnOnce(&mut Program),
    {
        let current = self.program.value().ok_or(Error::ProgramNotKnown)?.clone();
        let mut target = current.clone();
        edit(&mut target);

        let mut needed = 0;
        for (parameter, value) in current.changes(&target) {
            parameter.edit(u16::from(value))?;
            needed += 1;
        }
        if needed == 0 {
            return Ok(0);
        }
        let available = self.tx.remaining();
        if needed > available {
            return Err(Error::QueueFull { needed, available });
        }

        for (parameter, value) in current.changes(&target) {
            if let Ok(message) = parameter.edit(u16::from(value)) {
                self.tx.push(Outbound::Edit(message));
            }
        }
        self.program = Known::Assumed {
            value: target,
            sent_at: self.now,
        };
        Ok(needed)
    }

    /// Queues one request.
    fn ask(&mut self, request: Request) -> Result<()> {
        if self.tx.push(Outbound::Ask(request)) {
            Ok(())
        } else {
            Err(Error::QueueFull {
                needed: 1,
                available: 0,
            })
        }
    }

    /// Marks a request outstanding, as its bytes go out.
    fn start(&mut self, request: Request) {
        let now = self.now;
        let existing = self
            .pending
            .iter_mut()
            .find(|slot| slot.is_some_and(|entry| entry.request == request));
        if let Some(slot) = existing {
            *slot = Some(Pending {
                request,
                sent_at: now,
            });
            return;
        }
        if let Some(slot) = self.pending.iter_mut().find(|slot| slot.is_none()) {
            *slot = Some(Pending {
                request,
                sent_at: now,
            });
        }
    }
}

/// Queues an event, counting it as lost when there is no room.
fn enqueue<const EV: usize>(events: &mut Queue<Event, EV>, lost: &mut u16, event: Event) {
    if !events.push(event) {
        *lost = lost.saturating_add(1);
    }
}

/// The parts of a device that inbound bytes touch.
///
/// [`Device::feed`] borrows the decoder for the length of the call, so what the
/// decoder's closure needs has to be reachable without it. Naming those fields
/// here is what makes the two halves disjoint.
struct Inbound<'a, const EV: usize> {
    id: DeviceId,
    channel: Channel,
    now: u64,
    events: &'a mut Queue<Event, EV>,
    lost: &'a mut u16,
    pending: &'a mut [Option<Pending>; MAX_PENDING],
    program: &'a mut Known<Program>,
    identity: &'a mut Known<Identity>,
    nrpn: &'a mut Nrpn,
}

impl<const EV: usize> Inbound<'_, EV> {
    /// Queues an event.
    fn push(&mut self, event: Event) {
        enqueue(self.events, self.lost, event);
    }

    /// Takes one decoded MIDI message.
    fn handle(&mut self, event: core::result::Result<WireEvent<'_>, Error>) {
        match event {
            Ok(WireEvent::SysEx(bytes)) => self.sysex(bytes),
            Ok(WireEvent::Channel { channel, message }) => self.channel(channel, message),
            // Real-time and system common carry nothing about what the
            // synthesizer holds, and a running clock would drown the queue.
            Ok(WireEvent::Realtime(_) | WireEvent::Common(_)) => {}
            Err(error) => self.push(Event::Failed(error)),
        }
    }

    /// Takes one whole `SysEx` frame.
    fn sysex(&mut self, bytes: &[u8]) {
        match Frame::parse(bytes) {
            Ok(frame) => {
                // A frame addressed to another unit on the same port is not this
                // device's business.
                if self.id == DeviceId::Broadcast || frame.device.addresses(self.id) {
                    self.message(&frame.message);
                }
            }
            // A device inquiry reply is a universal frame, so it reaches here as
            // another manufacturer's: 0x7E is the universal non-real-time ID.
            Err(Error::Foreign {
                manufacturer,
                model,
            }) => match inquiry::parse_reply(bytes) {
                Ok(identity) => self.identity(identity),
                Err(_) => self.push(Event::Foreign {
                    manufacturer,
                    model,
                }),
            },
            Err(error) => self.push(Event::Failed(error)),
        }
    }

    /// Takes one `DeepMind` message addressed to this device.
    fn message(&mut self, message: &Message<'_>) {
        match *message {
            Message::EditBufferDumpResponse { version, packed } => {
                match Program::from_packed(version, packed) {
                    Ok(program) => {
                        *self.program = Known::Confirmed {
                            value: program.clone(),
                            at: self.now,
                        };
                        self.resolve(|request| request == Request::EditBuffer);
                        self.push(Event::EditBuffer(program));
                    }
                    Err(error) => self.push(Event::Failed(error)),
                }
            }
            Message::ProgramDumpResponse {
                version,
                bank,
                program,
                packed,
            } => {
                let slot = Slot::new(bank, program);
                match Program::from_packed(version, packed) {
                    Ok(program) => {
                        self.arrived(slot);
                        self.push(Event::Program { slot, program });
                    }
                    Err(error) => self.push(Event::Failed(error)),
                }
            }
            _ => self.push(Event::Unhandled(message.command())),
        }
    }

    /// Takes a device inquiry reply.
    ///
    /// A broadcast inquiry is answered by every unit on the port, so a reply
    /// from one this device does not talk to is another unit's answer.
    fn identity(&mut self, identity: Identity) {
        if self.id != DeviceId::Broadcast && !identity.device.addresses(self.id) {
            return;
        }
        *self.identity = Known::Confirmed {
            value: identity,
            at: self.now,
        };
        self.resolve(|request| request == Request::Identity);
        self.push(Event::Identity(identity));
    }

    /// Takes one channel message.
    fn channel(&mut self, channel: Channel, message: ChannelMessage) {
        if !self.consumed(channel, message) {
            self.push(Event::Channel { channel, message });
        }
    }

    /// Returns whether the message was a parameter edit on this device's channel.
    fn consumed(&mut self, channel: Channel, message: ChannelMessage) -> bool {
        if channel != self.channel {
            return false;
        }
        match message {
            ChannelMessage::ControlChange { controller, value } => {
                self.control_change(controller, value)
            }
            _ => false,
        }
    }

    /// Takes one control change, returning whether it was a parameter edit.
    fn control_change(&mut self, controller: u8, value: u8) -> bool {
        match controller {
            NRPN_NUMBER_MSB => {
                self.nrpn.number_msb = Some(value);
                self.nrpn.data_msb = None;
                true
            }
            NRPN_NUMBER_LSB => {
                self.nrpn.number_lsb = Some(value);
                self.nrpn.data_msb = None;
                true
            }
            // Held until its other half arrives: the value is fourteen bits and
            // this is the top seven of them.
            DATA_ENTRY_MSB => {
                self.nrpn.data_msb = Some(value);
                true
            }
            DATA_ENTRY_LSB => {
                self.data_entry(value);
                true
            }
            _ => match Controller::for_cc(controller).and_then(|control| control.parameter) {
                Some(parameter) => {
                    // Seven bits of a value that usually has eight, so what it
                    // says is close rather than exact.
                    self.parameter(parameter, parameter.from_cc_value(value), false);
                    true
                }
                None => false,
            },
        }
    }

    /// Applies a data entry LSB against the selected parameter.
    fn data_entry(&mut self, lsb: u8) {
        let Some(parameter) = self.nrpn.parameter() else {
            return;
        };
        let msb = self.nrpn.data_msb.take().unwrap_or(0);
        let value = (u16::from(msb) << 7) | u16::from(lsb);
        // The selection stays in force, so a knob sweep is one selection and a
        // run of values.
        self.parameter(parameter, value, true);
    }

    /// Records that a parameter now holds `value`.
    fn parameter(&mut self, parameter: ParamId, value: u16, exact: bool) {
        self.push(Event::Parameter { parameter, value });

        let Ok(byte) = u8::try_from(value) else {
            return;
        };
        let Some(current) = self.program.value() else {
            return;
        };
        let mut updated = current.clone();
        if updated.set(parameter, byte).is_err() {
            return;
        }
        *self.program = if exact && self.program.is_confirmed() {
            Known::Confirmed {
                value: updated,
                at: self.now,
            }
        } else {
            Known::Assumed {
                value: updated,
                sent_at: self.now,
            }
        };
    }

    /// Drops the first outstanding request `matches` accepts.
    fn resolve<F: Fn(Request) -> bool>(&mut self, matches: F) {
        if let Some(slot) = self
            .pending
            .iter_mut()
            .find(|slot| slot.is_some_and(|entry| matches(entry.request)))
        {
            *slot = None;
        }
    }

    /// Notes that the dump for `slot` arrived.
    ///
    /// A bank request is answered one dump at a time, so every dump inside its
    /// range is progress and only the last one ends it.
    fn arrived(&mut self, slot: Slot) {
        let now = self.now;
        for entry in self.pending.iter_mut() {
            match *entry {
                Some(Pending {
                    request: Request::Program(asked),
                    ..
                }) if asked == slot => {
                    *entry = None;
                    return;
                }
                Some(Pending {
                    request: Request::Bank { bank, first, last },
                    ..
                }) if bank == slot.bank
                    && (first.get()..=last.get()).contains(&slot.number.get()) =>
                {
                    if slot.number.get() == last.get() {
                        *entry = None;
                    } else if let Some(pending) = entry.as_mut() {
                        pending.sent_at = now;
                    }
                    return;
                }
                _ => {}
            }
        }
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
    use crate::ids::ProtocolVersion;
    use crate::program::ProgramName;
    use crate::sysex::{Command, inquiry};

    /// Room for the longest frame these tests build, which is a program dump.
    const FRAME: usize = 320;

    /// A port that remembers what it was handed.
    struct Port {
        bytes: [u8; 4096],
        len: usize,
        writes: usize,
    }

    impl Port {
        const fn new() -> Self {
            Self {
                bytes: [0; 4096],
                len: 0,
                writes: 0,
            }
        }

        fn send(&mut self, chunk: &[u8]) -> core::result::Result<(), ()> {
            let end = self.len + chunk.len();
            if end > self.bytes.len() {
                return Err(());
            }
            self.bytes[self.len..end].copy_from_slice(chunk);
            self.len = end;
            self.writes += 1;
            Ok(())
        }

        fn sent(&self) -> &[u8] {
            &self.bytes[..self.len]
        }
    }

    fn program(name: &str) -> Program {
        let mut program = Program::new(ProtocolVersion::V6).expect("a version the build knows");
        program.set_name(ProgramName::new(name).expect("a name the display can write"));
        program
    }

    fn slot(bank: Bank, number: u8) -> Slot {
        Slot::new(
            bank,
            ProgramNumber::new(number).expect("a program in range"),
        )
    }

    /// Encodes a frame from unit 0 into `out`.
    fn frame<'a>(out: &'a mut [u8; FRAME], message: Message<'_>) -> &'a [u8] {
        let len = Frame::new(DeviceId::Unit(0), message)
            .encode_into(out)
            .expect("the buffer fits");
        &out[..len]
    }

    /// Feeds the dump that answers an edit buffer request.
    fn feed_edit_buffer<const RX: usize, const TX: usize, const EV: usize>(
        device: &mut Device<RX, TX, EV>,
        program: &Program,
    ) {
        let mut packed = [0; Program::PACKED_MAX_LEN];
        let len = program.pack_into(&mut packed).expect("the buffer fits");
        let mut out = [0; FRAME];
        device.feed(frame(
            &mut out,
            Message::EditBufferDumpResponse {
                version: program.version(),
                packed: &packed[..len],
            },
        ));
    }

    /// Feeds the dump that answers a stored program request.
    fn feed_program<const RX: usize, const TX: usize, const EV: usize>(
        device: &mut Device<RX, TX, EV>,
        slot: Slot,
        program: &Program,
    ) {
        let mut packed = [0; Program::PACKED_MAX_LEN];
        let len = program.pack_into(&mut packed).expect("the buffer fits");
        let mut out = [0; FRAME];
        device.feed(frame(
            &mut out,
            Message::ProgramDumpResponse {
                version: program.version(),
                bank: slot.bank,
                program: slot.number,
                packed: &packed[..len],
            },
        ));
    }

    /// Feeds one control change on channel 1.
    fn feed_cc<const RX: usize, const TX: usize, const EV: usize>(
        device: &mut Device<RX, TX, EV>,
        controller: u8,
        value: u8,
    ) {
        device.feed(&[0xB0, controller, value]);
    }

    fn drain(device: &mut Device) -> Port {
        let mut port = Port::new();
        device.drain_tx(|bytes| port.send(bytes)).unwrap_or(0);
        port
    }

    fn events(device: &mut Device) -> impl Iterator<Item = Event> + '_ {
        core::iter::from_fn(move || device.poll_event())
    }

    #[test]
    fn a_request_is_bytes_only_once_it_is_drained() {
        let mut device: Device = Device::new(DeviceId::Unit(0));
        device.request_edit_buffer().expect("room in the queue");

        assert_eq!(device.queued(), 1);
        assert_eq!(device.outstanding().count(), 0, "queued is not sent");

        let port = drain(&mut device);
        assert_eq!(
            port.sent(),
            [0xF0, 0x00, 0x20, 0x32, 0x20, 0x00, 0x03, 0xF7]
        );
        assert_eq!(device.queued(), 0);
        assert_eq!(
            device.outstanding().next(),
            Some(Request::EditBuffer),
            "sent is outstanding"
        );
    }

    #[test]
    fn an_edit_buffer_dump_confirms_the_program_and_answers_the_request() {
        let mut device: Device = Device::new(DeviceId::Unit(0));
        device.request_edit_buffer().expect("room in the queue");
        drain(&mut device);
        assert_eq!(device.outstanding().count(), 1);

        let sound = program("Bass Sweep");
        feed_edit_buffer(&mut device, &sound);

        assert!(matches!(device.poll_event(), Some(Event::EditBuffer(_))));
        assert_eq!(device.poll_event(), None);
        assert!(device.program().is_confirmed());
        assert_eq!(
            device.program().value().map(Program::name),
            Some(sound.name())
        );
        assert_eq!(device.outstanding().count(), 0);
    }

    #[test]
    fn the_same_bytes_arrive_the_same_way_one_at_a_time() {
        let sound = program("Bass Sweep");
        let mut packed = [0; Program::PACKED_MAX_LEN];
        let len = sound.pack_into(&mut packed).expect("the buffer fits");
        let mut out = [0; FRAME];
        let bytes = frame(
            &mut out,
            Message::EditBufferDumpResponse {
                version: sound.version(),
                packed: &packed[..len],
            },
        );

        let mut whole: Device = Device::new(DeviceId::Unit(0));
        whole.feed(bytes);

        let mut piecemeal: Device = Device::new(DeviceId::Unit(0));
        for byte in bytes {
            piecemeal.feed(&[*byte]);
        }

        assert_eq!(whole.program(), piecemeal.program());
        assert!(whole.program().is_confirmed());
    }

    #[test]
    fn a_request_that_goes_unanswered_times_out_once() {
        let mut device: Device = Device::new(DeviceId::Unit(0)).with_timeout(100);
        device.request_edit_buffer().expect("room in the queue");
        drain(&mut device);

        device.tick(99);
        assert_eq!(device.poll_event(), None, "not overdue yet");

        device.tick(100);
        assert_eq!(
            device.poll_event(),
            Some(Event::Timeout(Request::EditBuffer))
        );
        device.tick(10_000);
        assert_eq!(device.poll_event(), None, "raised once, then forgotten");
    }

    #[test]
    fn a_request_never_drained_never_times_out() {
        let mut device: Device = Device::new(DeviceId::Unit(0)).with_timeout(100);
        device.request_edit_buffer().expect("room in the queue");

        device.tick(10_000);
        assert_eq!(device.poll_event(), None);
        assert_eq!(device.queued(), 1, "still waiting to be sent");
    }

    #[test]
    fn a_clock_that_goes_backwards_is_ignored() {
        let mut device: Device = Device::new(DeviceId::Unit(0)).with_timeout(100);
        device.tick(1_000);
        device.tick(10);
        assert_eq!(device.now(), 1_000);
    }

    #[test]
    fn editing_before_anything_is_known_says_so() {
        let mut device: Device = Device::new(DeviceId::Unit(0));
        assert_eq!(
            device.edit(|program| program.set_lfo1_rate(64)),
            Err(Error::ProgramNotKnown)
        );
    }

    #[test]
    fn an_edit_sends_one_message_for_each_parameter_that_changed() {
        let mut device: Device = Device::new(DeviceId::Unit(0));
        feed_edit_buffer(&mut device, &program("Bass Sweep"));
        while device.poll_event().is_some() {}

        let changed = device
            .edit(|program| {
                program.set_lfo1_rate(64);
                program.set_lfo1_rate(65);
                let was = program.vcf_frequency();
                program.set_vcf_frequency(was);
            })
            .expect("a program to diff against");

        assert_eq!(changed, 1, "two writes to one parameter, one message");
        let port = drain(&mut device);
        assert_eq!(
            port.sent(),
            [0xB0, 99, 0, 0xB0, 98, 0, 0xB0, 6, 0, 0xB0, 38, 65]
        );
        assert_eq!(port.writes, 1, "one call per message");
        assert!(device.program().is_assumed(), "sent, not confirmed");
    }

    #[test]
    fn an_edit_that_changes_nothing_sends_nothing() {
        let mut device: Device = Device::new(DeviceId::Unit(0));
        feed_edit_buffer(&mut device, &program("Bass Sweep"));
        while device.poll_event().is_some() {}

        assert_eq!(device.edit(|_| {}), Ok(0));
        assert_eq!(device.queued(), 0);
        assert!(device.program().is_confirmed(), "still nothing sent");
    }

    #[test]
    fn an_edit_too_big_for_the_queue_queues_none_of_it() {
        let mut device: Device<MAX_SYSEX_LEN, 1, 4> = Device::new(DeviceId::Unit(0));
        feed_edit_buffer(&mut device, &program("Bass Sweep"));
        while device.poll_event().is_some() {}

        let outcome = device.edit(|program| {
            program.set_lfo1_rate(64);
            program.set_lfo2_rate(64);
        });

        assert_eq!(
            outcome,
            Err(Error::QueueFull {
                needed: 2,
                available: 1
            })
        );
        assert_eq!(device.queued(), 0, "nothing half-sent");
        assert!(device.program().is_confirmed(), "and nothing assumed");
    }

    #[test]
    fn an_nrpn_from_the_synthesizer_keeps_the_program_confirmed() {
        let mut device: Device = Device::new(DeviceId::Unit(0));
        feed_edit_buffer(&mut device, &program("Bass Sweep"));
        while device.poll_event().is_some() {}

        feed_cc(&mut device, NRPN_NUMBER_MSB, 0);
        feed_cc(&mut device, NRPN_NUMBER_LSB, ParamId::Lfo1Rate.offset());
        feed_cc(&mut device, DATA_ENTRY_MSB, 0);
        feed_cc(&mut device, DATA_ENTRY_LSB, 99);

        assert_eq!(
            device.poll_event(),
            Some(Event::Parameter {
                parameter: ParamId::Lfo1Rate,
                value: 99
            })
        );
        assert_eq!(device.poll_event(), None, "the selection is not an event");
        assert!(device.program().is_confirmed(), "the whole value arrived");
        assert_eq!(device.program().value().map(Program::lfo1_rate), Some(99),);
    }

    #[test]
    fn a_selected_parameter_stays_selected_across_a_sweep() {
        let mut device: Device = Device::new(DeviceId::Unit(0));
        feed_edit_buffer(&mut device, &program("Bass Sweep"));
        while device.poll_event().is_some() {}

        feed_cc(&mut device, NRPN_NUMBER_MSB, 0);
        feed_cc(&mut device, NRPN_NUMBER_LSB, ParamId::Lfo1Rate.offset());
        for value in [10, 11, 12] {
            feed_cc(&mut device, DATA_ENTRY_MSB, 0);
            feed_cc(&mut device, DATA_ENTRY_LSB, value);
        }

        let seen: usize = events(&mut device)
            .filter(|event| matches!(event, Event::Parameter { .. }))
            .count();
        assert_eq!(seen, 3);
        assert_eq!(device.program().value().map(Program::lfo1_rate), Some(12));
    }

    #[test]
    fn a_controller_carries_seven_bits_so_the_program_becomes_assumed() {
        let mut device: Device = Device::new(DeviceId::Unit(0));
        feed_edit_buffer(&mut device, &program("Bass Sweep"));
        while device.poll_event().is_some() {}

        feed_cc(&mut device, 16, 64);

        let expected = ParamId::Lfo1Rate.from_cc_value(64);
        assert_eq!(
            device.poll_event(),
            Some(Event::Parameter {
                parameter: ParamId::Lfo1Rate,
                value: expected
            })
        );
        assert!(
            device.program().is_assumed(),
            "a controller does not confirm a value it cannot carry"
        );
    }

    #[test]
    fn a_message_on_another_channel_is_reported_and_not_applied() {
        let mut device: Device = Device::new(DeviceId::Unit(0));
        feed_edit_buffer(&mut device, &program("Bass Sweep"));
        while device.poll_event().is_some() {}
        let before = device.program().clone();

        device.feed(&[0xB1, 16, 64]);

        assert!(matches!(device.poll_event(), Some(Event::Channel { .. })));
        assert_eq!(device.program(), &before);
    }

    #[test]
    fn a_note_is_reported_rather_than_swallowed() {
        let mut device: Device = Device::new(DeviceId::Unit(0));
        device.feed(&[0x90, 60, 100]);
        assert!(matches!(device.poll_event(), Some(Event::Channel { .. })));
    }

    #[test]
    fn a_running_clock_is_not_an_event() {
        let mut device: Device = Device::new(DeviceId::Unit(0));
        device.feed(&[0xF8; 64]);
        assert_eq!(device.poll_event(), None);
    }

    #[test]
    fn a_stored_program_is_reported_and_not_tracked() {
        let mut device: Device = Device::new(DeviceId::Unit(0));
        let target = slot(Bank::from_letter('B').expect("a bank letter"), 4);
        device.request_program(target).expect("room in the queue");
        drain(&mut device);

        feed_program(&mut device, target, &program("Lead"));

        match device.poll_event() {
            Some(Event::Program { slot, program }) => {
                assert_eq!(slot, target);
                assert_eq!(program.name().as_str(), "Lead");
            }
            other => panic!("expected a program dump, got {other:?}"),
        }
        assert!(
            device.program().is_unknown(),
            "a stored program is not the sound it is making"
        );
        assert_eq!(device.outstanding().count(), 0);
    }

    #[test]
    fn a_bank_request_ends_on_its_last_program() {
        let mut device: Device = Device::new(DeviceId::Unit(0)).with_timeout(100);
        let first = ProgramNumber::new(0).expect("a program in range");
        let last = ProgramNumber::new(2).expect("a program in range");
        device
            .request_bank(Bank::A, first, last)
            .expect("room in the queue");
        drain(&mut device);

        let sound = program("Lead");
        for number in 0..=1 {
            device.tick(u64::from(number) * 90);
            feed_program(&mut device, slot(Bank::A, number), &sound);
            assert_eq!(
                device.outstanding().count(),
                1,
                "a dump inside the run is progress, not an answer"
            );
        }
        device.tick(180);
        assert_eq!(device.outstanding().count(), 1, "and it stays progress");

        feed_program(&mut device, slot(Bank::A, 2), &sound);
        assert_eq!(device.outstanding().count(), 0);
    }

    #[test]
    fn a_bank_request_before_a_late_program_is_rejected() {
        let mut device: Device = Device::new(DeviceId::Unit(0));
        let first = ProgramNumber::new(4).expect("a program in range");
        let last = ProgramNumber::new(2).expect("a program in range");
        assert_eq!(
            device.request_bank(Bank::A, first, last),
            Err(Error::ProgramOutOfRange(2))
        );
    }

    #[test]
    fn a_device_inquiry_reply_reports_firmware() {
        let mut device: Device = Device::new(DeviceId::Unit(0));
        device.request_identity().expect("room in the queue");
        drain(&mut device);

        let identity = Identity {
            device: DeviceId::Unit(0),
            firmware: inquiry::Version { major: 1, minor: 1 },
            voice: inquiry::Version { major: 1, minor: 0 },
        };
        let mut reply = [0; inquiry::REPLY_LEN];
        inquiry::reply_into(&identity, &mut reply).expect("the buffer fits");
        device.feed(&reply);

        assert_eq!(device.poll_event(), Some(Event::Identity(identity)));
        assert!(device.identity().is_confirmed());
        assert_eq!(device.firmware(), identity.firmware);
        assert_eq!(device.outstanding().count(), 0);
    }

    #[test]
    fn another_units_inquiry_reply_is_that_units_answer() {
        let mut device: Device = Device::new(DeviceId::Unit(0));
        let identity = Identity {
            device: DeviceId::Unit(1),
            firmware: inquiry::Version { major: 1, minor: 1 },
            voice: inquiry::Version { major: 1, minor: 0 },
        };
        let mut reply = [0; inquiry::REPLY_LEN];
        inquiry::reply_into(&identity, &mut reply).expect("the buffer fits");
        device.feed(&reply);

        assert_eq!(device.poll_event(), None);
        assert!(device.identity().is_unknown());
    }

    #[test]
    fn the_firmware_is_the_current_one_until_the_synthesizer_says_otherwise() {
        let device: Device = Device::new(DeviceId::Unit(0));
        assert_eq!(device.firmware(), DEFAULT_FIRMWARE);
    }

    #[test]
    fn another_manufacturers_frame_is_reported_as_theirs() {
        let mut device: Device = Device::new(DeviceId::Unit(0));
        device.feed(&[0xF0, 0x43, 0x00, 0x00, 0x01, 0x00, 0x00, 0xF7]);
        assert_eq!(
            device.poll_event(),
            Some(Event::Foreign {
                manufacturer: [0x43, 0x00, 0x00],
                model: 0x01
            })
        );
    }

    #[test]
    fn a_frame_for_another_unit_is_not_this_devices_business() {
        let mut device: Device = Device::new(DeviceId::Unit(1));
        feed_edit_buffer(&mut device, &program("Bass Sweep"));
        assert_eq!(device.poll_event(), None);
        assert!(device.program().is_unknown());
    }

    #[test]
    fn a_broadcast_host_listens_to_every_unit() {
        let mut device: Device = Device::new(DeviceId::Broadcast);
        feed_edit_buffer(&mut device, &program("Bass Sweep"));
        assert!(matches!(device.poll_event(), Some(Event::EditBuffer(_))));
    }

    #[test]
    fn a_message_this_layer_does_not_track_is_still_reported() {
        let mut device: Device = Device::new(DeviceId::Unit(0));
        let mut out = [0; FRAME];
        device.feed(frame(&mut out, Message::ChordMemoryDumpRequest));
        assert_eq!(
            device.poll_event(),
            Some(Event::Unhandled(Command::ChordMemoryDumpRequest))
        );
    }

    #[test]
    fn an_unreadable_frame_is_reported_with_its_reason() {
        let mut device: Device = Device::new(DeviceId::Unit(0));
        // A DeepMind header with a command byte no message uses.
        device.feed(&[0xF0, 0x00, 0x20, 0x32, 0x20, 0x00, 0x7E, 0xF7]);
        assert_eq!(
            device.poll_event(),
            Some(Event::Failed(Error::UnknownCommand(0x7E)))
        );
    }

    #[test]
    fn events_that_do_not_fit_are_counted_and_reported() {
        let mut device: Device<MAX_SYSEX_LEN, 8, 2> = Device::new(DeviceId::Unit(0));
        for _ in 0..5 {
            device.feed(&[0x90, 60, 100]);
        }

        assert!(matches!(device.poll_event(), Some(Event::Channel { .. })));
        assert!(matches!(device.poll_event(), Some(Event::Channel { .. })));
        assert_eq!(device.poll_event(), Some(Event::Lost(3)));
        assert_eq!(device.poll_event(), None);
    }

    #[test]
    fn a_port_that_refuses_keeps_everything_queued() {
        let mut device: Device = Device::new(DeviceId::Unit(0));
        device.request_identity().expect("room in the queue");
        device.request_edit_buffer().expect("room in the queue");

        let outcome: core::result::Result<usize, ()> = device.drain_tx(|_| Err(()));

        assert_eq!(outcome, Err(()));
        assert_eq!(device.queued(), 2, "the refused item stays first");
        assert_eq!(device.outstanding().count(), 0, "nothing went out");
    }

    #[test]
    fn a_full_outbound_queue_says_so() {
        let mut device: Device<MAX_SYSEX_LEN, 1, 2> = Device::new(DeviceId::Unit(0));
        device.request_identity().expect("room in the queue");
        assert_eq!(
            device.request_edit_buffer(),
            Err(Error::QueueFull {
                needed: 1,
                available: 0
            })
        );
    }

    #[test]
    fn a_seeded_program_is_assumed_and_sends_nothing() {
        let mut device: Device = Device::new(DeviceId::Unit(0));
        device.assume_program(program("From A File"));

        assert!(device.program().is_assumed());
        assert_eq!(device.queued(), 0);
        assert_eq!(device.edit(|program| program.set_lfo1_rate(64)), Ok(1));
    }

    #[test]
    fn a_reset_forgets_everything() {
        let mut device: Device = Device::new(DeviceId::Unit(0)).with_timeout(100);
        device.request_edit_buffer().expect("room in the queue");
        drain(&mut device);
        feed_edit_buffer(&mut device, &program("Bass Sweep"));
        device.feed(&[0x90, 60]);

        device.reset();

        assert!(device.program().is_unknown());
        assert!(device.identity().is_unknown());
        assert_eq!(device.queued(), 0);
        assert_eq!(device.poll_event(), None);
        assert_eq!(device.outstanding().count(), 0);
        device.tick(10_000);
        assert_eq!(device.poll_event(), None);
    }

    #[test]
    fn the_channel_an_edit_goes_out_on_is_the_one_it_was_given() {
        let channel = Channel::new(3).expect("a channel in range");
        let mut device: Device = Device::new(DeviceId::Unit(0)).with_channel(channel);
        device.assume_program(program("Bass Sweep"));
        device
            .edit(|program| program.set_lfo1_rate(1))
            .expect("a program to diff against");

        let port = drain(&mut device);
        assert_eq!(port.sent().first(), Some(&0xB3));
        assert_eq!(device.channel(), channel);
    }
}
