//! Turning a byte stream into whole MIDI messages.

use core::fmt;

use crate::error::{Error, Result};

use super::{Channel, ChannelMessage, Realtime, SYSEX_END, SYSEX_START, SystemCommon};

/// Buffer size the default [`Decoder`] uses.
///
/// The longest documented frame is the bank program names dump response: seven
/// header bytes, a version and a bank byte, 2344 packed bytes and `F7`.
pub const MAX_SYSEX_LEN: usize = 2354;

/// One whole message pulled out of a byte stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Event<'a> {
    /// A channel voice message and the channel it arrived on.
    Channel {
        /// Channel from the status byte's low nibble.
        channel: Channel,
        /// The message itself.
        message: ChannelMessage,
    },
    /// A complete `SysEx` frame, `F0` through `F7` inclusive.
    ///
    /// The slice borrows the decoder's buffer and lives until the next byte is
    /// fed in. Anything a host keeps has to be copied out, or parsed on the spot
    /// with [`crate::sysex::Frame::parse`].
    SysEx(&'a [u8]),
    /// A system common message.
    Common(SystemCommon),
    /// A system real-time message.
    Realtime(Realtime),
}

/// Reassembles MIDI messages from bytes arriving in any chunking.
///
/// The decoder handles the three things that make a MIDI stream awkward:
///
/// - **Running status.** A status byte stays in force until another one arrives,
///   so a stream of note-ons may carry one status byte and nothing but data
///   afterwards. System common messages clear it; real-time messages do not.
/// - **Real-time interleaving.** `F8` through `FF` may appear between any two
///   bytes, including between the `F0` and `F7` of a `SysEx` frame. They are
///   reported where they arrive and disturb nothing.
/// - **Chunking.** A USB packet or a serial read boundary can fall anywhere.
///   Feeding one byte at a time and feeding a whole dump at once produce the
///   same events.
///
/// `N` is the `SysEx` buffer size and defaults to [`MAX_SYSEX_LEN`]. A frame
/// longer than the buffer is dropped, with [`Error::SysExTooLong`] reported once.
///
/// ```
/// use deepmind_midi::wire::{Decoder, Event};
///
/// // A firmware-sized buffer is wasted on a host that only sends NRPN edits.
/// let mut decoder: Decoder<64> = Decoder::new();
/// let mut frames = 0;
/// decoder.feed(&[0xF0, 0x00, 0x20, 0x32, 0x20, 0x00, 0x03, 0xF7], |event| {
///     if matches!(event, Ok(Event::SysEx(_))) {
///         frames += 1;
///     }
/// });
/// assert_eq!(frames, 1);
/// ```
pub struct Decoder<const N: usize = MAX_SYSEX_LEN> {
    /// `SysEx` frame under construction, starting at its `F0`.
    frame: [u8; N],
    /// Bytes of `frame` in use.
    len: usize,
    /// Set between `F0` and the `F7` that closes it.
    in_sysex: bool,
    /// Status byte in force, channel or system common. `SysEx` clears it.
    status: Option<u8>,
    /// Data bytes collected for `status` so far.
    data: [u8; 2],
    /// Number of `data` bytes in use.
    have: usize,
}

impl<const N: usize> fmt::Debug for Decoder<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Decoder")
            .field("capacity", &N)
            .field("in_sysex", &self.in_sysex)
            .field("buffered", &self.len)
            .field("status", &self.status)
            .finish_non_exhaustive()
    }
}

impl<const N: usize> Default for Decoder<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Decoder<N> {
    /// Builds a decoder with an empty buffer.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            frame: [0; N],
            len: 0,
            in_sysex: false,
            status: None,
            data: [0; 2],
            have: 0,
        }
    }

    /// Returns the `SysEx` buffer size in bytes.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Returns `true` while a `SysEx` frame is still being reassembled.
    #[must_use]
    pub const fn in_sysex(&self) -> bool {
        self.in_sysex
    }

    /// Forgets any partial message and the running status.
    ///
    /// Worth calling when a port is reopened, since the first bytes after a
    /// reconnect may be the middle of something.
    pub fn reset(&mut self) {
        self.len = 0;
        self.in_sysex = false;
        self.status = None;
        self.have = 0;
    }

    /// Feeds bytes in, calling `on_event` once per message or rejection.
    ///
    /// Chunking is irrelevant: a message split across two calls is reported when
    /// its last byte arrives.
    pub fn feed<F>(&mut self, bytes: &[u8], mut on_event: F)
    where
        F: FnMut(Result<Event<'_>>),
    {
        for &byte in bytes {
            self.step(byte, &mut on_event);
        }
    }

    /// Handles one byte.
    fn step<F>(&mut self, byte: u8, on_event: &mut F)
    where
        F: FnMut(Result<Event<'_>>),
    {
        if byte >= 0xF8 {
            // Real-time. Interrupts anything, disturbs nothing.
            if let Some(message) = Realtime::from_status(byte) {
                on_event(Ok(Event::Realtime(message)));
            }
        } else if byte >= 0x80 {
            self.status_byte(byte, on_event);
        } else {
            self.data_byte(byte, on_event);
        }
    }

    /// Handles a status byte, `0x80..=0xF7`.
    fn status_byte<F>(&mut self, byte: u8, on_event: &mut F)
    where
        F: FnMut(Result<Event<'_>>),
    {
        if self.in_sysex {
            if byte == SYSEX_END {
                self.finish_sysex(on_event);
                return;
            }
            // Any other status byte ends the frame early. The frame is lost; the
            // status byte that ended it is not, and is handled below.
            self.in_sysex = false;
            self.len = 0;
            on_event(Err(Error::SysExInterrupted));
        } else if byte == SYSEX_END {
            // An F7 with no F0 in front of it. Nothing to report.
            return;
        }

        self.have = 0;
        match byte {
            SYSEX_START => {
                self.status = None;
                self.in_sysex = true;
                self.len = 0;
                self.push_sysex(SYSEX_START, on_event);
            }
            // A channel status byte stays in force. F1, F2 and F3 are system
            // common messages that take data bytes, so they are held the same
            // way, and cleared again once their last data byte arrives.
            0x80..=0xEF | 0xF1..=0xF3 => self.status = Some(byte),
            // F6 is a system common message that is complete as it stands.
            0xF6 => {
                self.status = None;
                on_event(Ok(Event::Common(SystemCommon::TuneRequest)));
            }
            // F4 and F5 are undefined, and still clear the running status.
            _ => self.status = None,
        }
    }

    /// Handles a data byte, `0x00..=0x7F`.
    fn data_byte<F>(&mut self, byte: u8, on_event: &mut F)
    where
        F: FnMut(Result<Event<'_>>),
    {
        if self.in_sysex {
            self.push_sysex(byte, on_event);
            return;
        }
        let Some(status) = self.status else {
            // A data byte with no status in front of it, which is what the
            // middle of a message looks like after a reconnect.
            return;
        };
        if let Some(slot) = self.data.get_mut(self.have) {
            *slot = byte;
            self.have += 1;
        }
        if self.have < Self::data_len_for(status) {
            return;
        }
        self.have = 0;
        let (first, second) = (self.data.first().copied(), self.data.get(1).copied());
        let (first, second) = (first.unwrap_or_default(), second.unwrap_or_default());
        if let Some(message) = ChannelMessage::from_bytes(status, first, second) {
            if let Ok(channel) = Channel::new(status & 0x0F) {
                on_event(Ok(Event::Channel { channel, message }));
            }
            return;
        }
        // System common carries no running status: the next data byte belongs to
        // nothing until another status byte arrives.
        self.status = None;
        let common = match status {
            0xF1 => SystemCommon::TimeCodeQuarterFrame(first),
            0xF2 => SystemCommon::SongPosition(u16::from(second) << 7 | u16::from(first)),
            _ => SystemCommon::SongSelect(first),
        };
        on_event(Ok(Event::Common(common)));
    }

    /// Returns how many data bytes the message under `status` carries.
    const fn data_len_for(status: u8) -> usize {
        match ChannelMessage::data_len_for(status) {
            Some(len) => len,
            // F2 is the only two-byte system common message.
            None if status == 0xF2 => 2,
            None => 1,
        }
    }

    /// Appends one byte to the frame under construction.
    fn push_sysex<F>(&mut self, byte: u8, on_event: &mut F)
    where
        F: FnMut(Result<Event<'_>>),
    {
        if let Some(slot) = self.frame.get_mut(self.len) {
            *slot = byte;
            self.len += 1;
            return;
        }
        // Out of room. Drop the frame and every byte up to its F7, which then
        // reads as a stray end and is ignored.
        self.in_sysex = false;
        self.len = 0;
        on_event(Err(Error::SysExTooLong(N)));
    }

    /// Closes the frame under construction and reports it.
    fn finish_sysex<F>(&mut self, on_event: &mut F)
    where
        F: FnMut(Result<Event<'_>>),
    {
        self.push_sysex(SYSEX_END, on_event);
        if !self.in_sysex {
            // The F7 did not fit; push_sysex has already reported the overflow.
            return;
        }
        if let Some(frame) = self.frame.get(..self.len) {
            on_event(Ok(Event::SysEx(frame)));
        }
        self.in_sysex = false;
        self.len = 0;
    }
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use alloc::vec::Vec;

    use super::*;

    /// Collects owned events, since the borrowed frames do not outlive the feed.
    #[derive(Debug, PartialEq, Eq)]
    enum Owned {
        Channel(Channel, ChannelMessage),
        SysEx(Vec<u8>),
        Common(SystemCommon),
        Realtime(Realtime),
        Rejected(Error),
    }

    fn collect<const N: usize>(decoder: &mut Decoder<N>, bytes: &[u8]) -> Vec<Owned> {
        let mut out = Vec::new();
        decoder.feed(bytes, |event| {
            out.push(match event {
                Ok(Event::Channel { channel, message }) => Owned::Channel(channel, message),
                Ok(Event::SysEx(frame)) => Owned::SysEx(frame.to_vec()),
                Ok(Event::Common(message)) => Owned::Common(message),
                Ok(Event::Realtime(message)) => Owned::Realtime(message),
                Err(error) => Owned::Rejected(error),
            });
        });
        out
    }

    fn decode(bytes: &[u8]) -> Vec<Owned> {
        collect(&mut Decoder::<MAX_SYSEX_LEN>::new(), bytes)
    }

    #[test]
    fn running_status_applies_to_every_following_data_pair() {
        let events = decode(&[0x90, 0x3C, 0x40, 0x3E, 0x40, 0x80, 0x3C, 0x00]);
        assert_eq!(
            events,
            [
                Owned::Channel(
                    Channel::ONE,
                    ChannelMessage::NoteOn {
                        key: 0x3C,
                        velocity: 0x40
                    }
                ),
                Owned::Channel(
                    Channel::ONE,
                    ChannelMessage::NoteOn {
                        key: 0x3E,
                        velocity: 0x40
                    }
                ),
                Owned::Channel(
                    Channel::ONE,
                    ChannelMessage::NoteOff {
                        key: 0x3C,
                        velocity: 0x00
                    }
                ),
            ]
        );
    }

    #[test]
    fn chunking_does_not_change_what_comes_out() {
        let stream = [
            0xB0, 0x63, 0x01, 0x62, 0x0E, 0x06, 0x00, 0x26, 0x40, 0xF0, 0x00, 0x20, 0x32, 0x20,
            0x00, 0x03, 0xF7, 0xE1, 0x00, 0x40,
        ];
        let whole = decode(&stream);
        for chunk in 1..=stream.len() {
            let mut decoder = Decoder::<MAX_SYSEX_LEN>::new();
            let mut split = Vec::new();
            for piece in stream.chunks(chunk) {
                split.extend(collect(&mut decoder, piece));
            }
            assert_eq!(split, whole, "chunked {chunk} bytes at a time");
        }
    }

    #[test]
    fn real_time_bytes_interleave_anywhere_without_disturbing_anything() {
        let events = decode(&[
            0x90, 0xF8, 0x3C, 0xF8, 0x40, 0xF0, 0xFE, 0x7F, 0xF7, 0x3E, 0x40,
        ]);
        assert_eq!(
            events,
            [
                Owned::Realtime(Realtime::Clock),
                Owned::Realtime(Realtime::Clock),
                Owned::Channel(
                    Channel::ONE,
                    ChannelMessage::NoteOn {
                        key: 0x3C,
                        velocity: 0x40
                    }
                ),
                Owned::Realtime(Realtime::ActiveSensing),
                Owned::SysEx(alloc::vec![0xF0, 0x7F, 0xF7]),
                // SysEx cleared the running status, so the trailing pair is lost.
            ]
        );
    }

    #[test]
    fn a_status_byte_inside_a_frame_ends_it_and_is_still_acted_on() {
        let events = decode(&[0xF0, 0x00, 0x20, 0x90, 0x3C, 0x40]);
        assert_eq!(
            events,
            [
                Owned::Rejected(Error::SysExInterrupted),
                Owned::Channel(
                    Channel::ONE,
                    ChannelMessage::NoteOn {
                        key: 0x3C,
                        velocity: 0x40
                    }
                ),
            ]
        );
    }

    #[test]
    fn a_frame_larger_than_the_buffer_is_dropped_once_and_the_stream_recovers() {
        let mut decoder: Decoder<8> = Decoder::new();
        let mut stream = alloc::vec![0xF0];
        stream.extend(core::iter::repeat_n(0x00, 32));
        stream.push(0xF7);
        stream.extend([0x90, 0x3C, 0x40]);
        let events = collect(&mut decoder, &stream);
        assert_eq!(
            events,
            [
                Owned::Rejected(Error::SysExTooLong(8)),
                Owned::Channel(
                    Channel::ONE,
                    ChannelMessage::NoteOn {
                        key: 0x3C,
                        velocity: 0x40
                    }
                ),
            ]
        );
        assert!(!decoder.in_sysex());
    }

    #[test]
    fn a_frame_that_exactly_fills_the_buffer_still_comes_out() {
        let mut decoder: Decoder<8> = Decoder::new();
        let frame = [0xF0, 0x00, 0x20, 0x32, 0x20, 0x00, 0x03, 0xF7];
        assert_eq!(
            collect(&mut decoder, &frame),
            [Owned::SysEx(frame.to_vec())]
        );
    }

    #[test]
    fn system_common_messages_clear_the_running_status() {
        let events = decode(&[0x90, 0x3C, 0x40, 0xF2, 0x00, 0x02, 0x3E, 0x40]);
        assert_eq!(
            events,
            [
                Owned::Channel(
                    Channel::ONE,
                    ChannelMessage::NoteOn {
                        key: 0x3C,
                        velocity: 0x40
                    }
                ),
                Owned::Common(SystemCommon::SongPosition(256)),
                // The trailing pair has no status in front of it.
            ]
        );
        assert_eq!(
            decode(&[0xF1, 0x11, 0xF3, 0x04, 0xF6]),
            [
                Owned::Common(SystemCommon::TimeCodeQuarterFrame(0x11)),
                Owned::Common(SystemCommon::SongSelect(0x04)),
                Owned::Common(SystemCommon::TuneRequest),
            ]
        );
    }

    #[test]
    fn data_bytes_with_no_status_and_stray_ends_are_ignored() {
        assert_eq!(decode(&[0x40, 0x40, 0xF7, 0x40]), []);
    }

    #[test]
    fn a_second_f0_restarts_the_frame_and_reports_the_first() {
        let events = decode(&[0xF0, 0x01, 0xF0, 0x02, 0xF7]);
        assert_eq!(
            events,
            [
                Owned::Rejected(Error::SysExInterrupted),
                Owned::SysEx(alloc::vec![0xF0, 0x02, 0xF7]),
            ]
        );
    }

    #[test]
    fn reset_forgets_a_partial_frame_and_the_running_status() {
        let mut decoder = Decoder::<MAX_SYSEX_LEN>::new();
        assert_eq!(collect(&mut decoder, &[0x90, 0x3C, 0xF0, 0x01]), []);
        assert!(decoder.in_sysex());
        decoder.reset();
        assert!(!decoder.in_sysex());
        assert_eq!(collect(&mut decoder, &[0x02, 0xF7]), []);
    }
}
