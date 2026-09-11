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
    /// A dump carried a comms protocol version this build does not understand.
    UnsupportedProtocolVersion(u8),
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
            Self::UnsupportedProtocolVersion(version) => {
                write!(
                    f,
                    "unsupported comms protocol version {version}, expected 6 or 7"
                )
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
