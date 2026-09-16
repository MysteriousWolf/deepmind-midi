//! The blocking adapter: a port, a clock, and the loop between them.
//!
//! [`Device`] is a state machine with no idea what IO is. This is the one module
//! that knows, through two traits: [`Port`] sends and receives bytes, [`Clock`]
//! tells the time and waits. The rest is the loop a host would otherwise write
//! itself.
//!
//! ```
//! use deepmind_midi::device::Device;
//! use deepmind_midi::ids::{DeviceId, ProtocolVersion};
//! use deepmind_midi::program::{Program, ProgramName};
//! use deepmind_midi::sysex::{Frame, Message};
//! use deepmind_midi::transport::{Port, StdClock, Transport};
//!
//! // Stand in for the synthesizer: a port that has one dump waiting on it.
//! struct Loopback {
//!     waiting: Vec<u8>,
//! }
//!
//! impl Port for Loopback {
//!     type Error = std::convert::Infallible;
//!
//!     fn send(&mut self, _bytes: &[u8]) -> Result<(), Self::Error> {
//!         Ok(())
//!     }
//!
//!     fn receive(&mut self, into: &mut [u8]) -> Result<usize, Self::Error> {
//!         let taken = self.waiting.len().min(into.len());
//!         into[..taken].copy_from_slice(&self.waiting[..taken]);
//!         self.waiting.drain(..taken);
//!         Ok(taken)
//!     }
//! }
//!
//! // The dump that port will hand over.
//! let mut sound = Program::new(ProtocolVersion::V6);
//! sound.set_name(ProgramName::new("Bass Sweep")?);
//! let mut packed = [0; Program::PACKED_MAX_LEN];
//! let packed_len = sound.pack_into(&mut packed)?;
//! let mut dump = [0; 512];
//! let dump_len = Frame::new(
//!     DeviceId::Unit(0),
//!     Message::EditBufferDumpResponse {
//!         version: sound.version(),
//!         packed: &packed[..packed_len],
//!     },
//! )
//! .encode_into(&mut dump)?;
//!
//! let port = Loopback { waiting: dump[..dump_len].to_vec() };
//! let mut synth: Transport<_, _> =
//!     Transport::new(Device::new(DeviceId::Unit(0)), port, StdClock::new());
//!
//! // Ask, and wait for the answer.
//! let program = synth.edit_buffer().expect("the dump waiting on the port");
//! assert_eq!(program.name().as_str(), "Bass Sweep");
//!
//! // Change one thing. The messages go out before this returns.
//! let changed = synth.edit(|program| program.set_lfo1_rate(64)).expect("the port takes it");
//! assert_eq!(changed, 1);
//! # Ok::<(), deepmind_midi::Error>(())
//! ```
//!
//! # Why this is opt-in
//!
//! Blocking is a policy, not a protocol. An async host wants its own cancellation
//! and backpressure, and gets both by writing the four calls in
//! [`device`](crate::device) into its own loop, about thirty lines. This module
//! is for hosts that would write the same thirty lines: a command-line tool, a
//! test harness, a thread that owns the port.
//!
//! It needs neither `std` nor an allocator. [`StdClock`] is the only thing here
//! that wants `std`; a host without it supplies its own [`Clock`].
//!
//! # The loop
//!
//! [`pump`](Transport::pump) is one pass: read what has arrived, feed it in,
//! advance the clock, send whatever the device queued. A host that wants its own
//! loop calls that and [`poll_event`](Transport::poll_event) and nothing else
//! here.
//!
//! ```
//! # use deepmind_midi::device::{Device, Event};
//! # use deepmind_midi::ids::DeviceId;
//! # use deepmind_midi::transport::{Port, StdClock, Transport};
//! # struct Quiet;
//! # impl Port for Quiet {
//! #     type Error = std::convert::Infallible;
//! #     fn send(&mut self, _: &[u8]) -> Result<(), Self::Error> { Ok(()) }
//! #     fn receive(&mut self, _: &mut [u8]) -> Result<usize, Self::Error> { Ok(0) }
//! # }
//! # let mut synth: Transport<_, _> = Transport::new(Device::new(DeviceId::Unit(0)), Quiet, StdClock::new());
//! # for _ in 0..1 {
//! synth.pump()?;
//! while let Some(event) = synth.poll_event() {
//!     println!("{event}");
//! }
//! # }
//! # Ok::<(), deepmind_midi::transport::Error<std::convert::Infallible>>(())
//! ```
//!
//! # Waiting, and what ends it
//!
//! The blocking calls each queue one request and wait for that request's answer.
//! Three things end the wait: the answer arrives, the device raises
//! [`Event::Timeout`] for it, or nothing has arrived for longer than
//! [`Device::timeout`] allows. The last two are both [`Error::Timeout`].
//!
//! Nothing is retried. Only the host knows whether a request that failed is worth
//! sending again.
//!
//! Progress is an event, not traffic. A bank transfer is one request and 128
//! answers, and each answer counts as progress, so a long transfer never times
//! out while it is still arriving, as [`Request::Bank`] promises. A port
//! streaming clock bytes at a silent synthesizer is not progress and does not
//! hold the wait open.
//!
//! # Events a wait was not waiting for
//!
//! A blocking call has to look at every event to find the one it wants. The
//! others go into a queue of this module's own, `EV` deep like the device's, and
//! come back out of [`poll_event`](Transport::poll_event) in order, so turning a
//! knob while a host reads a bank does not lose the knob.
//!
//! That queue is the transport's one cost: a second `EV` events held inline. An
//! event that does not fit is counted and arrives as [`Event::Lost`], the same as
//! in [`device`](crate::device).

mod clock;
mod error;
mod port;

pub use clock::Clock;
#[cfg(feature = "std")]
pub use clock::StdClock;
pub use error::Error;
pub use port::Port;

use crate::device::{ControlApp, DEFAULT_EVENT_DEPTH, DEFAULT_TX_DEPTH, Device, Event, Request};
use crate::ids::{Bank, ProgramNumber, Slot};
use crate::program::Program;
use crate::queue::Queue;
use crate::sysex::Identity;
use crate::wire::MAX_SYSEX_LEN;

/// Milliseconds a quiet loop waits before reading the port again.
///
/// Only reached when nothing arrived and nothing happened, so it is latency on
/// an idle connection and nothing else. Small because a MIDI port is not
/// expensive to read; [`Transport::with_poll_interval`] sets another, and zero
/// turns the loop into a spin.
pub const DEFAULT_POLL_MS: u64 = 1;

/// A [`Device`], a [`Port`] and a [`Clock`], driven together.
///
/// See the [module documentation](self) for the loop this runs and for what
/// waiting means. The three capacities are the device's, described in
/// [`device`](crate::device); `EV` bounds this module's own event queue too.
#[derive(Debug)]
pub struct Transport<
    P,
    C,
    const RX: usize = MAX_SYSEX_LEN,
    const TX: usize = DEFAULT_TX_DEPTH,
    const EV: usize = DEFAULT_EVENT_DEPTH,
> {
    device: Device<RX, TX, EV>,
    port: P,
    clock: C,
    /// What a wait looked at and was not waiting for.
    held: Queue<Event, EV>,
    /// Events there was no room to hold.
    lost: u16,
    /// Milliseconds a quiet loop waits between reads.
    interval: u64,
    /// Where bytes off the port land. As long as the longest frame the decoder
    /// behind it can reassemble, so nothing the library can read is ever split
    /// by this buffer rather than by the port.
    inbound: [u8; RX],
}

impl<P, C, const RX: usize, const TX: usize, const EV: usize> Transport<P, C, RX, TX, EV>
where
    P: Port,
    C: Clock,
{
    /// Builds a transport over a device, a port and a clock.
    ///
    /// The device is built by the caller rather than here, because how it is
    /// addressed and how patient it is are the caller's to say:
    /// [`Device::with_channel`] and [`Device::with_timeout`] are the two that
    /// usually matter.
    ///
    /// The three capacities have defaults, but a default is not something
    /// inference can reach for, so the binding says which transport this is:
    ///
    /// ```
    /// # use deepmind_midi::device::Device;
    /// # use deepmind_midi::ids::DeviceId;
    /// # use deepmind_midi::transport::{Port, StdClock, Transport};
    /// # struct Quiet;
    /// # impl Port for Quiet {
    /// #     type Error = std::convert::Infallible;
    /// #     fn send(&mut self, _: &[u8]) -> Result<(), Self::Error> { Ok(()) }
    /// #     fn receive(&mut self, _: &mut [u8]) -> Result<usize, Self::Error> { Ok(0) }
    /// # }
    /// # let (port, clock) = (Quiet, StdClock::new());
    /// // The defaults.
    /// let synth: Transport<_, _> = Transport::new(Device::new(DeviceId::Unit(0)), port, clock);
    ///
    /// // Or, for a host that sends single edits and never reads a bank:
    /// # let (port, clock) = (Quiet, StdClock::new());
    /// let small: Transport<_, _, 300, 8, 2> =
    ///     Transport::new(Device::new(DeviceId::Unit(0)), port, clock);
    /// # let _ = (synth, small);
    /// ```
    #[must_use]
    pub const fn new(device: Device<RX, TX, EV>, port: P, clock: C) -> Self {
        Self {
            device,
            port,
            clock,
            held: Queue::new(),
            lost: 0,
            interval: DEFAULT_POLL_MS,
            inbound: [0; RX],
        }
    }

    /// Sets how long a quiet loop waits before reading the port again.
    ///
    /// [`DEFAULT_POLL_MS`] otherwise. Zero spins, and with a clock that only
    /// moves in [`sleep_ms`](Clock::sleep_ms) it never moves at all, so a test
    /// clock wants at least one.
    #[must_use]
    pub const fn with_poll_interval(mut self, milliseconds: u64) -> Self {
        self.interval = milliseconds;
        self
    }

    /// Returns the device being driven, which is what holds the tracked state.
    #[must_use]
    pub const fn device(&self) -> &Device<RX, TX, EV> {
        &self.device
    }

    /// Returns the device, to queue something this module has no call for.
    ///
    /// Anything queued here goes out at the next [`pump`](Transport::pump) or
    /// [`flush`](Transport::flush).
    pub const fn device_mut(&mut self) -> &mut Device<RX, TX, EV> {
        &mut self.device
    }

    /// Returns the port, for whatever the host's own backend needs of it.
    pub const fn port_mut(&mut self) -> &mut P {
        &mut self.port
    }

    /// Gives back the three pieces, so a host can put the port down and keep
    /// what was learned through it.
    #[must_use]
    pub fn into_parts(self) -> (Device<RX, TX, EV>, P, C) {
        (self.device, self.port, self.clock)
    }

    /// Runs one pass of the loop: read, feed, tick, send.
    ///
    /// Returns how many bytes came off the port, which is zero on a quiet one.
    /// It does not wait and it does not look at events; a host driving its own
    /// loop calls this and [`poll_event`](Transport::poll_event).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Port`] when the port fails either way. A send that fails
    /// leaves its item queued, so draining again after a recovery sends it.
    pub fn pump(&mut self) -> Result<usize, Error<P::Error>> {
        let read = self
            .port
            .receive(&mut self.inbound)
            .map_err(Error::Port)?
            .min(RX);
        if read > 0 {
            self.device
                .feed(self.inbound.get(..read).unwrap_or_default());
        }
        let now = self.clock.now_ms();
        self.device.tick(now);
        self.flush()?;
        Ok(read)
    }

    /// Sends everything the device has queued, and returns how many items went.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Port`] when the port fails. Whatever had not gone yet
    /// stays queued.
    pub fn flush(&mut self) -> Result<usize, Error<P::Error>> {
        let Self { device, port, .. } = self;
        device
            .drain_tx(|bytes| port.send(bytes))
            .map_err(Error::Port)
    }

    /// Takes the oldest event: the ones a wait set aside first, then the
    /// device's.
    pub fn poll_event(&mut self) -> Option<Event> {
        if let Some(event) = self.held.pop() {
            return Some(event);
        }
        if let Some(event) = self.device.poll_event() {
            return Some(event);
        }
        let lost = core::mem::take(&mut self.lost);
        (lost > 0).then_some(Event::Lost(lost))
    }

    /// Drives the loop until `ready` picks something out of the events, or
    /// `limit_ms` passes.
    ///
    /// The escape hatch for waiting on something this module has no method for:
    /// a front-panel edit, a dump a host asked for through
    /// [`device_mut`](Transport::device_mut), a particular parameter landing.
    /// Events `ready` passes over are set aside for
    /// [`poll_event`](Transport::poll_event), and events that were already set
    /// aside are not offered again: this waits for what has not happened yet.
    ///
    /// The limit is the caller's patience and nothing else. A request's own
    /// timeout still arrives as [`Event::Timeout`], and `ready` is free to match
    /// it.
    ///
    /// ```
    /// # use deepmind_midi::device::{Device, Event};
    /// # use deepmind_midi::ids::DeviceId;
    /// # use deepmind_midi::param::ParamId;
    /// # use deepmind_midi::transport::{Port, StdClock, Transport};
    /// # struct Knob(bool);
    /// # impl Port for Knob {
    /// #     type Error = std::convert::Infallible;
    /// #     fn send(&mut self, _: &[u8]) -> Result<(), Self::Error> { Ok(()) }
    /// #     fn receive(&mut self, into: &mut [u8]) -> Result<usize, Self::Error> {
    /// #         if core::mem::replace(&mut self.0, true) { return Ok(0); }
    /// #         into[..3].copy_from_slice(&[0xB0, 0x63, 0x00]);
    /// #         Ok(3)
    /// #     }
    /// # }
    /// # let mut synth: Transport<_, _> = Transport::new(Device::new(DeviceId::Unit(0)), Knob(false), StdClock::new());
    /// // Wait up to a second for the synthesizer to say something about a parameter.
    /// let changed = synth.wait_for(1_000, |event| match event {
    ///     Event::Parameter { parameter, value } => Some((*parameter, *value)),
    ///     _ => None,
    /// });
    /// # let _ = changed;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Elapsed`] when `limit_ms` passes first, and
    /// [`Error::Port`] when the port fails.
    pub fn wait_for<T, F>(&mut self, limit_ms: u64, mut ready: F) -> Result<T, Error<P::Error>>
    where
        F: FnMut(&Event) -> Option<T>,
    {
        let deadline = self.clock.now_ms().saturating_add(limit_ms);
        loop {
            let read = self.pump()?;
            let mut seen = false;
            while let Some(event) = self.device.poll_event() {
                seen = true;
                if let Some(value) = ready(&event) {
                    return Ok(value);
                }
                self.hold(event);
            }
            if self.clock.now_ms() >= deadline {
                return Err(Error::Elapsed);
            }
            if read == 0 && !seen {
                self.clock.sleep_ms(self.interval);
            }
        }
    }

    /// Asks what the unit is, and waits for it to say.
    ///
    /// The only thing that reports firmware, which three value tables are read
    /// against. A host that cares which modulation sources it can name asks this
    /// first.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Timeout`] when nothing answers, [`Error::Port`] when the
    /// port fails, and [`Error::Protocol`] when the outbound queue has no room.
    pub fn identity(&mut self) -> Result<Identity, Error<P::Error>> {
        self.device.request_identity()?;
        self.answer(Request::Identity, |event| match event {
            Event::Identity(identity) => Offer::Answer(identity),
            other => Offer::Other(other),
        })
    }

    /// Announces the host and waits for what the interface answers.
    ///
    /// The only thing that reports which program the synthesizer has selected,
    /// which is otherwise something a host infers. The answer is handed back
    /// rather than tracked: see [`ControlApp`] for why.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Timeout`] when nothing answers, [`Error::Port`] when the
    /// port fails, and [`Error::Protocol`] when the outbound queue has no room.
    pub fn control_app(&mut self) -> Result<ControlApp, Error<P::Error>> {
        self.device.request_control_app()?;
        self.answer(Request::ControlApp, |event| match event {
            Event::ControlApp(control_app) => Offer::Answer(control_app),
            other => Offer::Other(other),
        })
    }

    /// Asks for the edit buffer, the program as it currently sounds, and waits
    /// for the dump.
    ///
    /// The answer is also what [`Device::program`] confirms afterwards, so a
    /// following [`edit`](Transport::edit) has something to diff against.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Timeout`] when nothing answers, [`Error::Port`] when the
    /// port fails, and [`Error::Protocol`] when the outbound queue has no room.
    pub fn edit_buffer(&mut self) -> Result<Program, Error<P::Error>> {
        self.device.request_edit_buffer()?;
        self.answer(Request::EditBuffer, |event| match event {
            Event::EditBuffer(program) => Offer::Answer(program),
            other => Offer::Other(other),
        })
    }

    /// Asks for one stored program and waits for the dump.
    ///
    /// A stored program is not the sound the synthesizer is making, so this
    /// returns it and leaves [`Device::program`] alone.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Timeout`] when nothing answers, [`Error::Port`] when the
    /// port fails, and [`Error::Protocol`] when the outbound queue has no room.
    pub fn program(&mut self, slot: Slot) -> Result<Program, Error<P::Error>> {
        self.device.request_program(slot)?;
        self.answer(Request::Program(slot), |event| match event {
            Event::Program { slot: at, program } if at == slot => Offer::Answer(program),
            other => Offer::Other(other),
        })
    }

    /// Asks for a run of stored programs and hands each dump to `on_program` as
    /// it arrives.
    ///
    /// One request, one answer per program. The dumps are handed over by
    /// reference and not collected, because 128 of them is thirty kilobytes and
    /// most hosts are writing each one somewhere as it lands; nor are they set
    /// aside for [`poll_event`](Transport::poll_event), since `on_program` is
    /// where they went. Returns how many arrived, which is the length of the run
    /// when nothing went wrong.
    ///
    /// Every dump is progress, so a slow transfer does not time out while it is
    /// still arriving; a gap longer than [`Device::timeout`] does.
    ///
    /// ```no_run
    /// # use deepmind_midi::device::Device;
    /// # use deepmind_midi::ids::{Bank, DeviceId, ProgramNumber};
    /// # use deepmind_midi::transport::{Port, StdClock, Transport};
    /// # fn run<P: Port>(synth: &mut Transport<P, StdClock>) -> Result<(), Box<dyn std::error::Error>>
    /// # where P::Error: std::error::Error + 'static {
    /// let mut names = Vec::new();
    /// let read = synth.bank(Bank::A, ProgramNumber::FIRST, ProgramNumber::LAST, |slot, program| {
    ///     names.push((slot, program.name()));
    /// })?;
    /// assert_eq!(read, 128);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Timeout`] when a dump does not arrive in time. The run
    /// is abandoned where it stopped, and the dumps already handed over stay
    /// handed over. Returns [`Error::Port`] when the port fails, and
    /// [`Error::Protocol`] when `last` is before `first` or the outbound queue
    /// has no room.
    pub fn bank<F>(
        &mut self,
        bank: Bank,
        first: ProgramNumber,
        last: ProgramNumber,
        mut on_program: F,
    ) -> Result<usize, Error<P::Error>>
    where
        F: FnMut(Slot, &Program),
    {
        self.device.request_bank(bank, first, last)?;
        let mut read = 0usize;
        self.answer(Request::Bank { bank, first, last }, |event| match event {
            Event::Program { slot, program }
                if slot.bank == bank && (first.get()..=last.get()).contains(&slot.number.get()) =>
            {
                on_program(slot, &program);
                read += 1;
                if slot.number.get() == last.get() {
                    Offer::Answer(())
                } else {
                    Offer::Progress
                }
            }
            other => Offer::Other(other),
        })?;
        Ok(read)
    }

    /// Edits the tracked program and sends the messages that carry the change.
    ///
    /// [`Device::edit`] with the sending done: the closure is handed the program
    /// as the synthesizer is believed to hold it, and what it changes is what
    /// goes out. Returns how many parameters changed. Nothing answers an edit,
    /// so this does not wait.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Protocol`] when no program is known yet, when the
    /// closure left a parameter out of range, or when the change needs more room
    /// than the outbound queue has. Nothing is queued in any of those cases.
    /// Returns [`Error::Port`] when the port fails partway, which leaves the
    /// rest of the edit queued for the next [`flush`](Transport::flush).
    pub fn edit<F>(&mut self, edit: F) -> Result<usize, Error<P::Error>>
    where
        F: FnOnce(&mut Program),
    {
        let changed = self.device.edit(edit)?;
        self.flush()?;
        Ok(changed)
    }

    /// Waits for the answer to `request`, holding every event that is not it.
    ///
    /// The caller queues the request and this sends it: the first
    /// [`pump`](Transport::pump) drains it to the port, which is also when its
    /// timeout starts.
    ///
    /// `extract` is offered each event and says what it was: the answer,
    /// progress it consumed, or none of its business, in which case it gives
    /// the event back to be set aside for [`poll_event`](Transport::poll_event).
    ///
    /// Ends on the answer, on the device raising the timeout for the request, or
    /// on a silence longer than the device allows. The last is the backstop for
    /// a request the device is not timing, which happens past
    /// [`MAX_PENDING`](crate::device::MAX_PENDING) concurrent ones. Silence is
    /// the absence of `SysEx` traffic and of progress: notes and knob turns
    /// arriving from the panel say the synthesizer is on, not that it is
    /// answering.
    fn answer<T, F>(&mut self, request: Request, mut extract: F) -> Result<T, Error<P::Error>>
    where
        F: FnMut(Event) -> Offer<T>,
    {
        let grace = self.device.timeout().saturating_add(self.interval);
        let mut quiet_since = self.clock.now_ms();
        loop {
            let read = self.pump()?;
            let mut seen = false;
            let mut progress = false;
            while let Some(event) = self.device.poll_event() {
                seen = true;
                progress |= is_sysex_traffic(&event);
                let event = match extract(event) {
                    Offer::Answer(value) => return Ok(value),
                    Offer::Progress => {
                        progress = true;
                        continue;
                    }
                    Offer::Other(event) => event,
                };
                if event == Event::Timeout(request) {
                    return Err(Error::Timeout(request));
                }
                self.hold(event);
            }
            let now = self.clock.now_ms();
            if progress {
                quiet_since = now;
            } else if now.saturating_sub(quiet_since) > grace {
                return Err(Error::Timeout(request));
            }
            if read == 0 && !seen {
                self.clock.sleep_ms(self.interval);
            }
        }
    }

    /// Sets an event aside for [`poll_event`](Transport::poll_event), counting
    /// it as lost when there is no room.
    fn hold(&mut self, event: Event) {
        if !self.held.push(event) {
            self.lost = self.lost.saturating_add(1);
        }
    }
}

/// What a wait made of an event it was offered.
#[expect(
    clippy::large_enum_variant,
    reason = "the event is given back rather than copied, and lives only as long as one match"
)]
enum Offer<T> {
    /// The answer, which ends the wait.
    Answer(T),
    /// Part of the answer, consumed; the wait goes on.
    Progress,
    /// Not the wait's business, given back to be set aside.
    Other(Event),
}

/// Returns whether the event came out of a `SysEx` frame, which is the
/// synthesizer talking to the host rather than the panel being played.
const fn is_sysex_traffic(event: &Event) -> bool {
    matches!(
        event,
        Event::EditBuffer(_)
            | Event::Program { .. }
            | Event::Identity(_)
            | Event::Unhandled(_)
            | Event::Failed(_)
            | Event::Foreign { .. }
    )
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use super::*;
    use crate::device::{DEFAULT_TIMEOUT_MS, MAX_PENDING};
    use crate::ids::{DeviceId, ProtocolVersion};
    use crate::program::ProgramName;
    use crate::sysex::{Frame, Message, inquiry};
    use crate::wire::ChannelMessage;

    /// Room for the longest frame these tests build, which is a program dump.
    const FRAME: usize = 512;
    /// Room for everything one test hands the port, which is a few of those.
    const SCRIPT: usize = 4096;

    /// What the fake port fails with, when a test asks it to.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Fault;

    impl core::fmt::Display for Fault {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_str("the cable fell out")
        }
    }

    /// A port a test scripts.
    ///
    /// It remembers everything sent through it, and hands back what was queued
    /// for it only once a request has actually gone out, so the order a test
    /// sees is the order a synthesizer would produce.
    struct Port {
        sent: [u8; SCRIPT],
        sent_len: usize,
        writes: usize,
        inbound: [u8; SCRIPT],
        inbound_len: usize,
        at: usize,
        chunk: usize,
        reads: usize,
        faulty_send: bool,
        faulty_receive: bool,
    }

    impl Port {
        const fn new() -> Self {
            Self {
                sent: [0; SCRIPT],
                sent_len: 0,
                writes: 0,
                inbound: [0; SCRIPT],
                inbound_len: 0,
                at: 0,
                chunk: SCRIPT,
                reads: 0,
                faulty_send: false,
                faulty_receive: false,
            }
        }

        /// Queues bytes for the synthesizer to "answer" with.
        fn queue(&mut self, bytes: &[u8]) -> &mut Self {
            let end = self.inbound_len + bytes.len();
            assert!(end <= SCRIPT, "the script does not fit");
            self.inbound[self.inbound_len..end].copy_from_slice(bytes);
            self.inbound_len = end;
            self
        }

        /// Hands back at most `bytes` per read, to chunk a frame up.
        const fn in_chunks_of(mut self, bytes: usize) -> Self {
            self.chunk = bytes;
            self
        }

        fn sent(&self) -> &[u8] {
            &self.sent[..self.sent_len]
        }
    }

    impl super::Port for Port {
        type Error = Fault;

        fn send(&mut self, bytes: &[u8]) -> Result<(), Fault> {
            if self.faulty_send {
                return Err(Fault);
            }
            let end = self.sent_len + bytes.len();
            assert!(end <= SCRIPT, "more was sent than the test holds");
            self.sent[self.sent_len..end].copy_from_slice(bytes);
            self.sent_len = end;
            self.writes += 1;
            Ok(())
        }

        fn receive(&mut self, into: &mut [u8]) -> Result<usize, Fault> {
            if self.faulty_receive {
                return Err(Fault);
            }
            self.reads += 1;
            // Nothing answers a question nobody asked.
            if self.writes == 0 {
                return Ok(0);
            }
            let taken = (self.inbound_len - self.at).min(into.len()).min(self.chunk);
            into[..taken].copy_from_slice(&self.inbound[self.at..self.at + taken]);
            self.at += taken;
            Ok(taken)
        }
    }

    /// A clock that only moves when something waits on it.
    #[derive(Debug, Clone, Copy)]
    struct Fake {
        now: u64,
    }

    impl Fake {
        const fn new() -> Self {
            Self { now: 0 }
        }
    }

    impl Clock for Fake {
        fn now_ms(&mut self) -> u64 {
            self.now
        }

        fn sleep_ms(&mut self, milliseconds: u64) {
            // At least a millisecond, so a quiet loop cannot spin forever in a
            // test the way a spinning host legitimately can.
            self.now = self.now.saturating_add(milliseconds.max(1));
        }
    }

    fn program(name: &str) -> Program {
        let mut program = Program::new(ProtocolVersion::V6);
        program.set_name(ProgramName::new(name).expect("a name the display can write"));
        program
    }

    fn slot(number: u8) -> Slot {
        Slot::new(
            Bank::A,
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

    /// The dump that answers an edit buffer request.
    fn edit_buffer_dump<'a>(out: &'a mut [u8; FRAME], sound: &Program) -> &'a [u8] {
        let mut packed = [0; Program::PACKED_MAX_LEN];
        let len = sound.pack_into(&mut packed).expect("the buffer fits");
        frame(
            out,
            Message::EditBufferDumpResponse {
                version: sound.version(),
                packed: &packed[..len],
            },
        )
    }

    /// The dump that answers a stored program request.
    fn program_dump<'a>(out: &'a mut [u8; FRAME], at: Slot, sound: &Program) -> &'a [u8] {
        let mut packed = [0; Program::PACKED_MAX_LEN];
        let len = sound.pack_into(&mut packed).expect("the buffer fits");
        frame(
            out,
            Message::ProgramDumpResponse {
                version: sound.version(),
                bank: at.bank,
                program: at.number,
                packed: &packed[..len],
            },
        )
    }

    fn transport(port: Port) -> Transport<Port, Fake> {
        Transport::new(Device::new(DeviceId::Unit(0)), port, Fake::new())
    }

    /// A transport as patient as the test needs it to be.
    fn patient_for(port: Port, timeout: u64) -> Transport<Port, Fake> {
        Transport::new(
            Device::new(DeviceId::Unit(0)).with_timeout(timeout),
            port,
            Fake::new(),
        )
    }

    /// A transport that gives up quickly, for the tests about giving up.
    fn impatient(port: Port) -> Transport<Port, Fake> {
        patient_for(port, 50)
    }

    #[test]
    fn the_request_goes_out_before_the_answer_is_waited_for() {
        let mut out = [0; FRAME];
        let mut port = Port::new();
        port.queue(edit_buffer_dump(&mut out, &program("Bass Sweep")));

        let mut synth = transport(port);
        let sound = synth.edit_buffer().expect("the dump the port holds");

        assert_eq!(sound.name().as_str(), "Bass Sweep");
        let (_, port, _) = synth.into_parts();
        assert_eq!(
            port.sent(),
            &[0xF0, 0x00, 0x20, 0x32, 0x20, 0x00, 0x03, 0xF7],
            "the edit buffer request, and nothing else"
        );
    }

    #[test]
    fn the_answer_is_also_what_the_device_now_confirms() {
        let mut out = [0; FRAME];
        let mut port = Port::new();
        port.queue(edit_buffer_dump(&mut out, &program("Bass Sweep")));

        let mut synth = transport(port);
        synth.edit_buffer().expect("the dump the port holds");

        assert!(synth.device().program().is_confirmed());
        assert_eq!(
            synth
                .device()
                .program()
                .value()
                .map(|sound| sound.name().as_str().to_owned()),
            Some("Bass Sweep".to_owned())
        );
    }

    #[test]
    fn a_dump_split_a_byte_at_a_time_still_arrives() {
        let mut out = [0; FRAME];
        let mut port = Port::new().in_chunks_of(1);
        port.queue(edit_buffer_dump(&mut out, &program("Slow Port")));

        let mut synth = transport(port);
        let sound = synth
            .edit_buffer()
            .expect("the dump, however it was cut up");

        assert_eq!(sound.name().as_str(), "Slow Port");
    }

    #[test]
    fn a_synthesizer_that_says_nothing_becomes_a_timeout() {
        let mut synth = impatient(Port::new());

        assert_eq!(
            synth.edit_buffer(),
            Err(Error::Timeout(Request::EditBuffer)),
            "the request timed out, and says which request"
        );
    }

    #[test]
    fn a_timeout_does_not_fire_while_the_answer_is_still_in_time() {
        let mut out = [0; FRAME];
        let mut port = Port::new();
        port.queue(edit_buffer_dump(&mut out, &program("Just In Time")));

        let mut synth = transport(port);
        synth.edit_buffer().expect("an answer inside the timeout");

        let (device, _, clock) = synth.into_parts();
        assert!(
            clock.now < DEFAULT_TIMEOUT_MS,
            "the wait ended on the answer, not on the clock"
        );
        assert_eq!(device.outstanding().count(), 0);
    }

    #[test]
    fn an_identity_answers_the_inquiry() {
        let identity = Identity {
            device: DeviceId::Unit(0),
            firmware: inquiry::Version { major: 1, minor: 1 },
            voice: inquiry::Version { major: 1, minor: 0 },
        };
        let mut reply = [0; inquiry::REPLY_LEN];
        inquiry::reply_into(&identity, &mut reply).expect("the buffer fits");
        let mut port = Port::new();
        port.queue(&reply);

        let mut synth = transport(port);

        assert_eq!(synth.identity(), Ok(identity));
        assert_eq!(synth.device().firmware(), identity.firmware);
    }

    #[test]
    fn a_stored_program_comes_back_without_becoming_the_edit_buffer() {
        let mut out = [0; FRAME];
        let mut port = Port::new();
        port.queue(program_dump(&mut out, slot(7), &program("Stored")));

        let mut synth = transport(port);
        let sound = synth.program(slot(7)).expect("the dump the port holds");

        assert_eq!(sound.name().as_str(), "Stored");
        assert!(
            synth.device().program().value().is_none(),
            "a stored program is not the sound the synthesizer is making"
        );
    }

    #[test]
    fn a_dump_for_another_slot_is_held_rather_than_answered() {
        let mut wrong = [0; FRAME];
        let mut right = [0; FRAME];
        let mut port = Port::new();
        port.queue(program_dump(&mut wrong, slot(3), &program("Not It")));
        port.queue(program_dump(&mut right, slot(7), &program("Stored")));

        let mut synth = transport(port);
        let sound = synth.program(slot(7)).expect("the dump that was asked for");

        assert_eq!(sound.name().as_str(), "Stored");
        assert!(
            matches!(synth.poll_event(), Some(Event::Program { slot: at, .. }) if at == slot(3)),
            "the dump nobody asked for is still there to be seen"
        );
    }

    #[test]
    fn a_bank_ends_on_its_last_program() {
        let mut port = Port::new();
        let mut frames = [[0; FRAME]; 3];
        for (number, out) in frames.iter_mut().enumerate() {
            let number = u8::try_from(number).expect("three fits in a byte");
            port.queue(program_dump(out, slot(number), &program("Bank Sound")));
        }

        let mut synth = transport(port);
        let mut seen = [0u8; 3];
        let mut count = 0;
        let read = synth
            .bank(
                Bank::A,
                ProgramNumber::new(0).expect("a program in range"),
                ProgramNumber::new(2).expect("a program in range"),
                |at, _| {
                    seen[count] = at.number.get();
                    count += 1;
                },
            )
            .expect("three dumps");

        assert_eq!(read, 3);
        assert_eq!(seen, [0, 1, 2]);
        assert_eq!(
            synth.device().outstanding().count(),
            0,
            "the run ended with its last program"
        );
    }

    /// A dump from outside the run is somebody else's business: it is set
    /// aside like any other event and does not count towards the run.
    #[test]
    fn a_dump_outside_the_run_is_held_rather_than_counted() {
        let mut port = Port::new();
        let mut frames = [[0; FRAME]; 3];
        let numbers = [0, 5, 1];
        for (number, out) in numbers.into_iter().zip(frames.iter_mut()) {
            port.queue(program_dump(out, slot(number), &program("Bank Sound")));
        }

        let mut synth = transport(port);
        let read = synth
            .bank(
                Bank::A,
                ProgramNumber::new(0).expect("a program in range"),
                ProgramNumber::new(1).expect("a program in range"),
                |at, _| assert!(at.number.get() <= 1, "A6 is not in the run"),
            )
            .expect("two dumps");
        assert_eq!(read, 2);
        assert!(
            matches!(synth.poll_event(), Some(Event::Program { slot, .. }) if slot == self::slot(5)),
            "the stray dump comes back out afterwards"
        );
        assert_eq!(synth.poll_event(), None, "the run's own dumps do not");
    }

    /// Notes from the keyboard say the synthesizer is on, not that it is
    /// answering, so they do not hold a wait open past the timeout.
    #[test]
    fn playing_the_keyboard_does_not_hold_a_wait_open() {
        let mut port = Port::new().in_chunks_of(3);
        // More notes than the impatient timeout has milliseconds of polls.
        for _ in 0..200 {
            port.queue(&[0x90, 0x3C, 0x40]);
        }
        let mut synth = impatient(port);
        // Past MAX_PENDING, so the device is not timing this one and the
        // backstop is what ends the wait.
        for _ in 0..MAX_PENDING {
            synth.device_mut().request_identity().expect("room");
        }
        assert_eq!(
            synth.edit_buffer(),
            Err(Error::Timeout(Request::EditBuffer)),
            "notes are not an answer"
        );
    }

    #[test]
    fn a_bank_that_stops_partway_keeps_what_arrived() {
        let mut port = Port::new();
        let mut frames = [[0; FRAME]; 2];
        for (number, out) in frames.iter_mut().enumerate() {
            let number = u8::try_from(number).expect("two fits in a byte");
            port.queue(program_dump(out, slot(number), &program("Bank Sound")));
        }

        let mut synth = impatient(port);
        let mut count = 0;
        let outcome = synth.bank(
            Bank::A,
            ProgramNumber::new(0).expect("a program in range"),
            ProgramNumber::new(2).expect("a program in range"),
            |_, _| count += 1,
        );

        assert!(matches!(outcome, Err(Error::Timeout(Request::Bank { .. }))));
        assert_eq!(count, 2, "the dumps that did arrive were handed over");
    }

    #[test]
    fn every_dump_holds_a_bank_transfer_open() {
        // Three dumps arriving a byte at a time take far longer than one
        // timeout, and the transfer survives it because every dump is progress.
        let mut port = Port::new().in_chunks_of(1);
        let mut frames = [[0; FRAME]; 3];
        for (number, out) in frames.iter_mut().enumerate() {
            let number = u8::try_from(number).expect("three fits in a byte");
            port.queue(program_dump(out, slot(number), &program("Bank Sound")));
        }

        let mut synth = patient_for(port, 400);
        let read = synth
            .bank(
                Bank::A,
                ProgramNumber::new(0).expect("a program in range"),
                ProgramNumber::new(2).expect("a program in range"),
                |_, _| {},
            )
            .expect("a slow transfer is still a transfer");

        assert_eq!(read, 3);
    }

    #[test]
    fn events_a_wait_passed_over_come_back_afterwards() {
        let mut out = [0; FRAME];
        let mut port = Port::new().in_chunks_of(3);
        port.queue(&[0x90, 60, 100]);
        port.queue(edit_buffer_dump(&mut out, &program("Bass Sweep")));

        let mut synth = transport(port);
        synth.edit_buffer().expect("the dump the port holds");

        assert!(
            matches!(
                synth.poll_event(),
                Some(Event::Channel {
                    message: ChannelMessage::NoteOn { .. },
                    ..
                })
            ),
            "the note played while the host was waiting is still there"
        );
        assert_eq!(synth.poll_event(), None);
    }

    #[test]
    fn events_that_do_not_fit_are_counted_rather_than_dropped_quietly() {
        let mut out = [0; FRAME];
        let mut port = Port::new().in_chunks_of(3);
        for _ in 0..3 {
            port.queue(&[0x90, 60, 100]);
        }
        port.queue(edit_buffer_dump(&mut out, &program("Bass Sweep")));

        // Two events deep: the third note has nowhere to go.
        let mut synth: Transport<Port, Fake, MAX_SYSEX_LEN, DEFAULT_TX_DEPTH, 2> =
            Transport::new(Device::new(DeviceId::Unit(0)), port, Fake::new());
        synth.edit_buffer().expect("the dump the port holds");

        assert!(matches!(synth.poll_event(), Some(Event::Channel { .. })));
        assert!(matches!(synth.poll_event(), Some(Event::Channel { .. })));
        assert_eq!(synth.poll_event(), Some(Event::Lost(1)));
        assert_eq!(synth.poll_event(), None);
    }

    #[test]
    fn an_edit_sends_one_message_for_the_one_parameter_that_changed() {
        let mut out = [0; FRAME];
        let mut port = Port::new();
        port.queue(edit_buffer_dump(&mut out, &program("Bass Sweep")));

        let mut synth = transport(port);
        synth.edit_buffer().expect("the dump the port holds");
        let before = synth.port_mut().writes;

        assert_eq!(synth.edit(|sound| sound.set_lfo1_rate(64)), Ok(1));
        assert_eq!(
            synth.port_mut().writes - before,
            1,
            "one NRPN edit is one write"
        );
        assert_eq!(
            synth.device().queued(),
            0,
            "and it went out, not just queued"
        );
        assert!(synth.device().program().is_assumed());
    }

    #[test]
    fn an_edit_that_changes_nothing_sends_nothing() {
        let mut out = [0; FRAME];
        let mut port = Port::new();
        port.queue(edit_buffer_dump(&mut out, &program("Bass Sweep")));

        let mut synth = transport(port);
        let sound = synth.edit_buffer().expect("the dump the port holds");
        let rate = sound.lfo1_rate();
        let before = synth.port_mut().writes;

        assert_eq!(synth.edit(|sound| sound.set_lfo1_rate(rate)), Ok(0));
        assert_eq!(synth.port_mut().writes, before);
    }

    #[test]
    fn an_edit_before_any_program_is_known_is_refused() {
        let mut synth = transport(Port::new());

        assert_eq!(
            synth.edit(|sound| sound.set_lfo1_rate(64)),
            Err(Error::Protocol(crate::Error::ProgramNotKnown))
        );
    }

    #[test]
    fn a_port_that_cannot_send_says_so_and_keeps_the_item() {
        let mut port = Port::new();
        port.faulty_send = true;

        let mut synth = transport(port);

        assert_eq!(synth.edit_buffer(), Err(Error::Port(Fault)));
        assert_eq!(
            synth.device().queued(),
            1,
            "the request is still there to send again"
        );
    }

    #[test]
    fn a_port_that_cannot_read_says_so() {
        let mut port = Port::new();
        port.faulty_receive = true;

        let mut synth = transport(port);

        assert_eq!(synth.pump(), Err(Error::Port(Fault)));
        assert_eq!(synth.edit_buffer(), Err(Error::Port(Fault)));
    }

    #[test]
    fn the_ports_error_travels_through_unchanged() {
        let error: Error<Fault> = Error::Port(Fault);

        assert_eq!(error.port(), Some(&Fault));
        assert_eq!(error.request(), None);
        assert_eq!(
            Error::<Fault>::Timeout(Request::EditBuffer).request(),
            Some(Request::EditBuffer)
        );
    }

    #[test]
    fn waiting_for_something_the_host_asked_for_itself() {
        let mut out = [0; FRAME];
        let mut port = Port::new();
        port.queue(edit_buffer_dump(&mut out, &program("Bass Sweep")));

        let mut synth = transport(port);
        synth
            .device_mut()
            .request_edit_buffer()
            .expect("room in the queue");
        let name = synth
            .wait_for(DEFAULT_TIMEOUT_MS, |event| match event {
                Event::EditBuffer(sound) => Some(sound.name()),
                _ => None,
            })
            .expect("the dump the port holds");

        assert_eq!(name.as_str(), "Bass Sweep");
    }

    #[test]
    fn waiting_gives_up_when_the_callers_own_limit_passes() {
        let mut synth = transport(Port::new());

        assert_eq!(
            synth.wait_for(10, |_| Some(())),
            Err(Error::Elapsed),
            "nothing happened, so nothing was offered"
        );
        let (_, _, clock) = synth.into_parts();
        assert!(clock.now >= 10);
    }

    #[test]
    fn a_quiet_loop_waits_rather_than_spinning() {
        let mut synth = transport(Port::new()).with_poll_interval(5);
        let _ = synth.wait_for(20, |_| Some(()));

        let (_, port, clock) = synth.into_parts();
        assert!(clock.now >= 20);
        assert!(
            port.reads <= 8,
            "a five millisecond wait over twenty read the port {} times",
            port.reads
        );
    }

    #[test]
    fn pumping_reports_what_came_off_the_port() {
        let mut out = [0; FRAME];
        let mut port = Port::new();
        let dump = edit_buffer_dump(&mut out, &program("Bass Sweep")).len();
        port.queue(edit_buffer_dump(&mut out, &program("Bass Sweep")));

        let mut synth = transport(port);
        assert_eq!(synth.pump(), Ok(0), "nothing has been asked yet");
        synth
            .device_mut()
            .request_edit_buffer()
            .expect("room in the queue");
        synth.flush().expect("the port takes it");

        assert_eq!(synth.pump(), Ok(dump));
        assert!(matches!(synth.poll_event(), Some(Event::EditBuffer(_))));
    }

    #[test]
    fn the_pieces_come_back_apart() {
        let mut out = [0; FRAME];
        let mut port = Port::new();
        port.queue(edit_buffer_dump(&mut out, &program("Bass Sweep")));

        let mut synth = transport(port);
        synth.edit_buffer().expect("the dump the port holds");
        let (device, port, _) = synth.into_parts();

        assert!(
            device.program().is_confirmed(),
            "what was learned outlives the port it was learned through"
        );
        assert!(!port.sent().is_empty());
    }
}
