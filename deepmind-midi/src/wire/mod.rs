//! MIDI bytes: status, channel messages and `SysEx` reassembly.
//!
//! This layer knows nothing about `DeepMind`s. It turns a byte stream from a MIDI
//! port into whole messages, and whole messages back into bytes. What a `SysEx`
//! frame means is [`crate::sysex`]'s question; what a controller number means is
//! the parameter table's.
//!
//! ```
//! use deepmind_midi::wire::{Channel, ChannelMessage, Decoder, Event};
//!
//! // `Decoder` on its own is the default buffer size; `Decoder<64>` picks another.
//! let mut decoder: Decoder = Decoder::new();
//! let mut keys = Vec::new();
//!
//! // Running status: one status byte, then two note-ons.
//! decoder.feed(&[0x90, 0x3C, 0x40, 0x40, 0x40], |event| {
//!     if let Ok(Event::Channel { channel, message: ChannelMessage::NoteOn { key, .. } }) = event {
//!         assert_eq!(channel, Channel::ONE);
//!         keys.push(key);
//!     }
//! });
//!
//! assert_eq!(keys, [0x3C, 0x40]);
//! ```

mod decoder;

pub use decoder::{Decoder, Event, MAX_SYSEX_LEN};

use core::fmt;

use crate::error::{Error, Result};

/// Status byte that opens a `SysEx` frame.
pub const SYSEX_START: u8 = 0xF0;

/// Status byte that closes a `SysEx` frame.
pub const SYSEX_END: u8 = 0xF7;

/// Number of MIDI channels.
pub const CHANNEL_COUNT: u8 = 16;

/// A MIDI channel.
///
/// Stored as the wire value, `0..=15`. [`Display`](fmt::Display) prints the number
/// a musician reads off the front panel, `1..=16`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Channel(u8);

impl Channel {
    /// Channel 1, which is `0` on the wire.
    pub const ONE: Self = Self(0);
    /// Channel 16, which is `15` on the wire.
    pub const SIXTEEN: Self = Self(15);

    /// Builds a channel from its wire value.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ChannelOutOfRange`] for an index of 16 or more.
    pub const fn new(index: u8) -> Result<Self> {
        if index < CHANNEL_COUNT {
            Ok(Self(index))
        } else {
            Err(Error::ChannelOutOfRange(index))
        }
    }

    /// Returns the wire value, `0..=15`.
    #[must_use]
    pub const fn index(self) -> u8 {
        self.0
    }

    /// Returns the number as the front panel shows it, `1..=16`.
    #[must_use]
    pub const fn number(self) -> u8 {
        self.0 + 1
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.number())
    }
}

/// A channel voice message, without its channel.
///
/// Data bytes are 7-bit: every field except [`ChannelMessage::PitchBend`] holds
/// `0..=127`, and the decoder never produces anything else. A note-on with
/// velocity zero is reported as it arrived rather than rewritten into a note-off,
/// because the two are distinguishable on the wire and some devices mean the
/// difference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ChannelMessage {
    /// Key released.
    NoteOff {
        /// Key number, `0..=127`.
        key: u8,
        /// Release velocity, `0..=127`.
        velocity: u8,
    },
    /// Key pressed.
    NoteOn {
        /// Key number, `0..=127`.
        key: u8,
        /// Strike velocity, `0..=127`.
        velocity: u8,
    },
    /// Per-key aftertouch.
    PolyKeyPressure {
        /// Key number, `0..=127`.
        key: u8,
        /// Pressure, `0..=127`.
        pressure: u8,
    },
    /// Continuous controller.
    ControlChange {
        /// Controller number, `0..=127`.
        controller: u8,
        /// Controller value, `0..=127`.
        value: u8,
    },
    /// Program change.
    ProgramChange {
        /// Program number, `0..=127`.
        program: u8,
    },
    /// Channel aftertouch.
    ChannelPressure {
        /// Pressure, `0..=127`.
        pressure: u8,
    },
    /// Pitch bend, 14 bits.
    PitchBend {
        /// Bend position, `0..=16383`, centred on [`ChannelMessage::BEND_CENTRE`].
        value: u16,
    },
}

impl ChannelMessage {
    /// Pitch bend value meaning no bend.
    pub const BEND_CENTRE: u16 = 8192;

    /// Returns the high nibble of this message's status byte.
    #[must_use]
    pub const fn status_nibble(self) -> u8 {
        match self {
            Self::NoteOff { .. } => 0x80,
            Self::NoteOn { .. } => 0x90,
            Self::PolyKeyPressure { .. } => 0xA0,
            Self::ControlChange { .. } => 0xB0,
            Self::ProgramChange { .. } => 0xC0,
            Self::ChannelPressure { .. } => 0xD0,
            Self::PitchBend { .. } => 0xE0,
        }
    }

    /// Returns how many data bytes follow the status byte of `status`.
    ///
    /// Returns `None` for a byte that is not a channel status byte.
    #[must_use]
    pub const fn data_len_for(status: u8) -> Option<usize> {
        match status & 0xF0 {
            0x80 | 0x90 | 0xA0 | 0xB0 | 0xE0 => Some(2),
            0xC0 | 0xD0 => Some(1),
            _ => None,
        }
    }

    /// Returns the number of bytes [`encode_into`](Self::encode_into) writes.
    #[must_use]
    pub const fn encoded_len(self) -> usize {
        match self {
            Self::ProgramChange { .. } | Self::ChannelPressure { .. } => 2,
            _ => 3,
        }
    }

    /// Writes this message to `out`, status byte first.
    ///
    /// Values wider than their field are masked to 7 bits rather than rejected,
    /// so a caller that has already range-checked a value pays nothing and one
    /// that has not cannot emit a byte that would be read as a status.
    ///
    /// # Errors
    ///
    /// Returns [`Error::BufferTooSmall`] when `out` is shorter than
    /// [`encoded_len`](Self::encoded_len).
    pub fn encode_into(self, channel: Channel, out: &mut [u8]) -> Result<usize> {
        let needed = self.encoded_len();
        let available = out.len();
        let out = out
            .get_mut(..needed)
            .ok_or(Error::BufferTooSmall { needed, available })?;
        let (first, second) = match self {
            Self::NoteOff { key, velocity } | Self::NoteOn { key, velocity } => (key, velocity),
            Self::PolyKeyPressure { key, pressure } => (key, pressure),
            Self::ControlChange { controller, value } => (controller, value),
            Self::ProgramChange { program } => (program, 0),
            Self::ChannelPressure { pressure } => (pressure, 0),
            Self::PitchBend { value } => ((value & 0x7F) as u8, ((value >> 7) & 0x7F) as u8),
        };
        let bytes = [
            self.status_nibble() | channel.index(),
            first & 0x7F,
            second & 0x7F,
        ];
        for (slot, byte) in out.iter_mut().zip(bytes) {
            *slot = byte;
        }
        Ok(needed)
    }

    /// Builds a message from a status byte and up to two data bytes.
    ///
    /// Returns `None` when `status` is not a channel status byte.
    #[must_use]
    pub const fn from_bytes(status: u8, first: u8, second: u8) -> Option<Self> {
        let (key, velocity) = (first & 0x7F, second & 0x7F);
        Some(match status & 0xF0 {
            0x80 => Self::NoteOff { key, velocity },
            0x90 => Self::NoteOn { key, velocity },
            0xA0 => Self::PolyKeyPressure {
                key,
                pressure: velocity,
            },
            0xB0 => Self::ControlChange {
                controller: key,
                value: velocity,
            },
            0xC0 => Self::ProgramChange { program: key },
            0xD0 => Self::ChannelPressure { pressure: key },
            0xE0 => Self::PitchBend {
                value: ((velocity as u16) << 7) | key as u16,
            },
            _ => return None,
        })
    }
}

/// A system common message.
///
/// The `DeepMind` sends none of these. They are decoded because a MIDI port
/// carries whatever else is on the cable, and because every one of them clears
/// running status, which a decoder that ignored them would get wrong.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SystemCommon {
    /// One eighth of a MIDI time code frame.
    TimeCodeQuarterFrame(u8),
    /// Song position, in sixteenth notes since the start.
    SongPosition(u16),
    /// Song select.
    SongSelect(u8),
    /// Tune request.
    TuneRequest,
}

/// A system real-time message.
///
/// These are single bytes and may arrive between any two other bytes, including
/// inside a `SysEx` frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Realtime {
    /// Timing clock, 24 per quarter note.
    Clock,
    /// Start playing from the beginning.
    Start,
    /// Resume playing.
    Continue,
    /// Stop playing.
    Stop,
    /// Active sensing.
    ActiveSensing,
    /// System reset.
    SystemReset,
}

impl Realtime {
    /// Builds a real-time message from its status byte.
    ///
    /// Returns `None` for `F9` and `FD`, which are undefined.
    #[must_use]
    pub const fn from_status(status: u8) -> Option<Self> {
        Some(match status {
            0xF8 => Self::Clock,
            0xFA => Self::Start,
            0xFB => Self::Continue,
            0xFC => Self::Stop,
            0xFE => Self::ActiveSensing,
            0xFF => Self::SystemReset,
            _ => return None,
        })
    }

    /// Returns the status byte for this message.
    #[must_use]
    pub const fn to_status(self) -> u8 {
        match self {
            Self::Clock => 0xF8,
            Self::Start => 0xFA,
            Self::Continue => 0xFB,
            Self::Stop => 0xFC,
            Self::ActiveSensing => 0xFE,
            Self::SystemReset => 0xFF,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use super::*;

    #[test]
    fn channels_print_from_one_and_reject_out_of_range_values() {
        assert_eq!(Channel::new(0).expect("0 is in range").number(), 1);
        assert_eq!(Channel::SIXTEEN.index(), 15);
        assert_eq!(Channel::new(16), Err(Error::ChannelOutOfRange(16)));
    }

    #[test]
    fn channel_messages_round_trip_through_their_bytes() {
        let cases = [
            ChannelMessage::NoteOff {
                key: 60,
                velocity: 64,
            },
            ChannelMessage::NoteOn {
                key: 127,
                velocity: 1,
            },
            ChannelMessage::PolyKeyPressure {
                key: 60,
                pressure: 7,
            },
            ChannelMessage::ControlChange {
                controller: 99,
                value: 1,
            },
            ChannelMessage::ProgramChange { program: 42 },
            ChannelMessage::ChannelPressure { pressure: 12 },
            ChannelMessage::PitchBend { value: 0 },
            ChannelMessage::PitchBend {
                value: ChannelMessage::BEND_CENTRE,
            },
            ChannelMessage::PitchBend { value: 16383 },
        ];
        for message in cases {
            let mut buf = [0u8; 3];
            let channel = Channel::new(5).expect("5 is in range");
            let len = message.encode_into(channel, &mut buf).expect("buffer fits");
            assert_eq!(len, message.encoded_len());
            assert_eq!(buf[0] & 0x0F, 5);
            assert_eq!(
                ChannelMessage::from_bytes(buf[0], buf[1], buf[2]),
                Some(message)
            );
            assert_eq!(ChannelMessage::data_len_for(buf[0]), Some(len - 1));
        }
    }

    #[test]
    fn encoding_reports_a_buffer_it_cannot_fill() {
        let mut buf = [0u8; 2];
        assert_eq!(
            ChannelMessage::NoteOn {
                key: 60,
                velocity: 1
            }
            .encode_into(Channel::ONE, &mut buf),
            Err(Error::BufferTooSmall {
                needed: 3,
                available: 2
            })
        );
        // A two byte message fits a two byte buffer.
        assert_eq!(
            ChannelMessage::ProgramChange { program: 3 }.encode_into(Channel::ONE, &mut buf),
            Ok(2)
        );
        assert_eq!(buf, [0xC0, 3]);
    }

    #[test]
    fn pitch_bend_splits_into_lsb_then_msb() {
        let mut buf = [0u8; 3];
        ChannelMessage::PitchBend {
            value: ChannelMessage::BEND_CENTRE,
        }
        .encode_into(Channel::ONE, &mut buf)
        .expect("buffer fits");
        assert_eq!(buf, [0xE0, 0x00, 0x40]);
    }

    #[test]
    fn real_time_status_bytes_round_trip_and_leave_the_undefined_ones_out() {
        for status in 0xF8..=0xFF {
            match Realtime::from_status(status) {
                Some(message) => assert_eq!(message.to_status(), status),
                None => assert!(matches!(status, 0xF9 | 0xFD)),
            }
        }
    }

    #[test]
    fn non_channel_status_bytes_decode_to_nothing() {
        assert_eq!(ChannelMessage::from_bytes(0xF0, 0, 0), None);
        assert_eq!(ChannelMessage::data_len_for(0xF0), None);
    }
}
