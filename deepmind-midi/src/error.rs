//! Crate-wide error type.
//!
//! The library never panics on malformed input. Anything that arrives from a MIDI
//! port or a `.syx` file is untrusted and every rejection carries the offending
//! value so a host application can log something actionable.

use core::fmt;

/// Convenience alias for results produced by this crate.
pub type Result<T, E = Error> = core::result::Result<T, E>;

/// Everything that can go wrong while parsing or building `DeepMind` messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// A device ID byte was neither `0..=15` nor `0x7F`.
    InvalidDeviceId(u8),
    /// A bank index was 8 or greater.
    BankOutOfRange(u8),
    /// A program number was 128 or greater.
    ProgramOutOfRange(u8),
    /// A user pattern number was 32 or greater.
    PatternOutOfRange(u8),
    /// A MIDI channel was 16 or greater.
    ChannelOutOfRange(u8),
    /// A parameter offset was 242 or greater, so it names no parameter.
    ParameterOutOfRange(u8),
    /// A value was outside the range its parameter accepts.
    ValueOutOfRange {
        /// Offset of the parameter the value was meant for.
        parameter: u8,
        /// The value offered.
        value: u16,
        /// Highest value the parameter accepts. The lowest is always zero.
        max: u16,
    },
    /// A dump carried a comms protocol version this build does not understand.
    UnsupportedProtocolVersion(u8),
    /// A `SysEx` frame did not start with `F0` or did not end with `F7`.
    Unframed,
    /// A `SysEx` frame was too short to carry the header every message has.
    ShortFrame(usize),
    /// A `SysEx` frame carried another manufacturer's or another model's ID.
    Foreign {
        /// The three manufacturer ID bytes the frame carried.
        manufacturer: [u8; 3],
        /// The model ID byte the frame carried.
        model: u8,
    },
    /// A command byte no documented message uses.
    UnknownCommand(u8),
    /// A payload was not the length its command requires.
    PayloadLength {
        /// Command byte of the message being parsed.
        command: u8,
        /// Payload length the command calls for.
        expected: usize,
        /// Payload length the frame actually carried.
        found: usize,
    },
    /// A packed run ended in a high-bit byte with no data bytes to apply it to.
    PackedRunLength(usize),
    /// An output buffer was too small for what had to be written into it.
    BufferTooSmall {
        /// Bytes the operation needed.
        needed: usize,
        /// Bytes the caller supplied.
        available: usize,
    },
    /// A `SysEx` frame was longer than the decoder's buffer, and was dropped.
    SysExTooLong(usize),
    /// A status byte arrived before a `SysEx` frame's `F7`, ending it early.
    SysExInterrupted,
    /// An interface byte was neither MIDI (0), USB (1) nor Wi-Fi (2).
    InvalidInterface(u8),
    /// A universal `SysEx` frame was not a `DeepMind` device inquiry response.
    NotAnInquiryResponse,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDeviceId(byte) => {
                write!(
                    f,
                    "invalid device ID {byte:#04X}, expected 0x00..=0x0F or 0x7F"
                )
            }
            Self::BankOutOfRange(index) => write!(f, "bank {index} out of range, expected 0..=7"),
            Self::ProgramOutOfRange(number) => {
                write!(f, "program {number} out of range, expected 0..=127")
            }
            Self::PatternOutOfRange(number) => {
                write!(f, "pattern {number} out of range, expected 0..=31")
            }
            Self::ChannelOutOfRange(channel) => {
                write!(f, "MIDI channel {channel} out of range, expected 0..=15")
            }
            Self::ParameterOutOfRange(offset) => {
                write!(
                    f,
                    "parameter offset {offset} out of range, expected 0..=241"
                )
            }
            Self::ValueOutOfRange {
                parameter,
                value,
                max,
            } => write!(
                f,
                "value {value} out of range for parameter {parameter}, expected 0..={max}"
            ),
            Self::UnsupportedProtocolVersion(version) => {
                write!(
                    f,
                    "unsupported comms protocol version {version}, expected 6 or 7"
                )
            }
            Self::Unframed => f.write_str("not a SysEx frame: expected F0 ... F7"),
            Self::ShortFrame(len) => {
                write!(
                    f,
                    "SysEx frame of {len} bytes is too short to carry a header"
                )
            }
            Self::Foreign {
                manufacturer: [a, b, c],
                model,
            } => write!(
                f,
                "SysEx frame for manufacturer {a:02X} {b:02X} {c:02X} model {model:02X}, \
                 not Behringer 00 20 32 model 20"
            ),
            Self::UnknownCommand(command) => {
                write!(f, "unknown SysEx command {command:#04X}")
            }
            Self::PayloadLength {
                command,
                expected,
                found,
            } => write!(
                f,
                "command {command:#04X} carries a {found} byte payload, expected {expected}"
            ),
            Self::PackedRunLength(len) => write!(
                f,
                "packed run of {len} bytes ends in a high-bit byte with no data"
            ),
            Self::BufferTooSmall { needed, available } => {
                write!(f, "buffer of {available} bytes is too small, need {needed}")
            }
            Self::SysExTooLong(capacity) => {
                write!(f, "SysEx frame longer than the {capacity} byte buffer")
            }
            Self::SysExInterrupted => {
                f.write_str("SysEx frame interrupted by a status byte before its F7")
            }
            Self::InvalidInterface(byte) => write!(
                f,
                "invalid interface {byte}, expected 0 (MIDI), 1 (USB) or 2 (Wi-Fi)"
            ),
            Self::NotAnInquiryResponse => f.write_str("not a DeepMind device inquiry response"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
