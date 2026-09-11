//! The command byte that follows the device ID.

use core::fmt;

use crate::error::{Error, Result};

/// Which way round the wire a message travels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Direction {
    /// Sent by a host to the synthesizer.
    ToDevice,
    /// Sent by the synthesizer to a host.
    FromDevice,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ToDevice => "host to synth",
            Self::FromDevice => "synth to host",
        })
    }
}

/// A `DeepMind` `SysEx` command.
///
/// The 22 commands the manual documents, named as it names them. A direction is
/// what the synthesizer does with a command, not a rule about who may send one:
/// a preset pack is a file of program dump responses that a host sends to the
/// synthesizer to write them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Command {
    /// `00`, tells the synthesizer a control application is attached.
    ControlAppNotifyRequest,
    /// `01`, asks for one stored program.
    ProgramDumpRequest,
    /// `02`, carries one stored program.
    ProgramDumpResponse,
    /// `03`, asks for the edit buffer.
    EditBufferDumpRequest,
    /// `04`, carries the edit buffer.
    EditBufferDumpResponse,
    /// `05`, asks for the device-wide settings.
    GlobalParameterDumpRequest,
    /// `06`, carries the device-wide settings.
    GlobalParameterDumpResponse,
    /// `07`, asks for one user pattern.
    SingleUserPatternDumpRequest,
    /// `08`, carries a user pattern or the pattern edit buffer.
    UserPatternDumpResponse,
    /// `09`, asks for a run of programs, answered one program at a time.
    ProgramBankDumpRequest,
    /// `0A`, asks for the names of every program in a bank.
    BankProgramNamesDumpRequest,
    /// `0B`, carries 128 program names.
    BankProgramNamesDumpResponse,
    /// `0C`, asks for one program's name.
    SingleProgramNameDumpRequest,
    /// `0D`, carries one program's name.
    SingleProgramNameDumpResponse,
    /// `0E`, asks for the pattern edit buffer.
    EditBufferPatternDumpRequest,
    /// `10`, answers a control application notify request.
    ControlAppNotifyResponse,
    /// `11`, asks for the calibration data.
    CalibrationDataDumpRequest,
    /// `12`, carries the calibration data.
    CalibrationDataDumpResponse,
    /// `1B`, asks for the chord memory.
    ChordMemoryDumpRequest,
    /// `1C`, carries the chord memory.
    ChordMemoryDumpResponse,
    /// `1D`, asks for the poly chord memory.
    PolyChordMemoryDumpRequest,
    /// `1E`, carries the poly chord memory.
    PolyChordMemoryDumpResponse,
}

impl Command {
    /// Every documented command, ordered by command byte.
    pub const ALL: [Self; 22] = [
        Self::ControlAppNotifyRequest,
        Self::ProgramDumpRequest,
        Self::ProgramDumpResponse,
        Self::EditBufferDumpRequest,
        Self::EditBufferDumpResponse,
        Self::GlobalParameterDumpRequest,
        Self::GlobalParameterDumpResponse,
        Self::SingleUserPatternDumpRequest,
        Self::UserPatternDumpResponse,
        Self::ProgramBankDumpRequest,
        Self::BankProgramNamesDumpRequest,
        Self::BankProgramNamesDumpResponse,
        Self::SingleProgramNameDumpRequest,
        Self::SingleProgramNameDumpResponse,
        Self::EditBufferPatternDumpRequest,
        Self::ControlAppNotifyResponse,
        Self::CalibrationDataDumpRequest,
        Self::CalibrationDataDumpResponse,
        Self::ChordMemoryDumpRequest,
        Self::ChordMemoryDumpResponse,
        Self::PolyChordMemoryDumpRequest,
        Self::PolyChordMemoryDumpResponse,
    ];

    /// Builds a command from its wire byte.
    ///
    /// # Errors
    ///
    /// Returns [`Error::UnknownCommand`] for a byte no documented message uses,
    /// including the gaps at `0F` and `13`-`1A`.
    pub const fn from_byte(byte: u8) -> Result<Self> {
        Ok(match byte {
            0x00 => Self::ControlAppNotifyRequest,
            0x01 => Self::ProgramDumpRequest,
            0x02 => Self::ProgramDumpResponse,
            0x03 => Self::EditBufferDumpRequest,
            0x04 => Self::EditBufferDumpResponse,
            0x05 => Self::GlobalParameterDumpRequest,
            0x06 => Self::GlobalParameterDumpResponse,
            0x07 => Self::SingleUserPatternDumpRequest,
            0x08 => Self::UserPatternDumpResponse,
            0x09 => Self::ProgramBankDumpRequest,
            0x0A => Self::BankProgramNamesDumpRequest,
            0x0B => Self::BankProgramNamesDumpResponse,
            0x0C => Self::SingleProgramNameDumpRequest,
            0x0D => Self::SingleProgramNameDumpResponse,
            0x0E => Self::EditBufferPatternDumpRequest,
            0x10 => Self::ControlAppNotifyResponse,
            0x11 => Self::CalibrationDataDumpRequest,
            0x12 => Self::CalibrationDataDumpResponse,
            0x1B => Self::ChordMemoryDumpRequest,
            0x1C => Self::ChordMemoryDumpResponse,
            0x1D => Self::PolyChordMemoryDumpRequest,
            0x1E => Self::PolyChordMemoryDumpResponse,
            _ => return Err(Error::UnknownCommand(byte)),
        })
    }

    /// Returns the wire byte for this command.
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        match self {
            Self::ControlAppNotifyRequest => 0x00,
            Self::ProgramDumpRequest => 0x01,
            Self::ProgramDumpResponse => 0x02,
            Self::EditBufferDumpRequest => 0x03,
            Self::EditBufferDumpResponse => 0x04,
            Self::GlobalParameterDumpRequest => 0x05,
            Self::GlobalParameterDumpResponse => 0x06,
            Self::SingleUserPatternDumpRequest => 0x07,
            Self::UserPatternDumpResponse => 0x08,
            Self::ProgramBankDumpRequest => 0x09,
            Self::BankProgramNamesDumpRequest => 0x0A,
            Self::BankProgramNamesDumpResponse => 0x0B,
            Self::SingleProgramNameDumpRequest => 0x0C,
            Self::SingleProgramNameDumpResponse => 0x0D,
            Self::EditBufferPatternDumpRequest => 0x0E,
            Self::ControlAppNotifyResponse => 0x10,
            Self::CalibrationDataDumpRequest => 0x11,
            Self::CalibrationDataDumpResponse => 0x12,
            Self::ChordMemoryDumpRequest => 0x1B,
            Self::ChordMemoryDumpResponse => 0x1C,
            Self::PolyChordMemoryDumpRequest => 0x1D,
            Self::PolyChordMemoryDumpResponse => 0x1E,
        }
    }

    /// Returns the command's name, exactly as `spec/messages.toml` writes it.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::ControlAppNotifyRequest => "Control App Notify Request",
            Self::ProgramDumpRequest => "Program Dump Request",
            Self::ProgramDumpResponse => "Program Dump Response",
            Self::EditBufferDumpRequest => "Edit Buffer Dump Request",
            Self::EditBufferDumpResponse => "Edit Buffer Dump Response",
            Self::GlobalParameterDumpRequest => "Global Parameter Dump Request",
            Self::GlobalParameterDumpResponse => "Global Parameter Dump Response",
            Self::SingleUserPatternDumpRequest => "Single User Pattern Dump Request",
            Self::UserPatternDumpResponse => "User Pattern Dump Response",
            Self::ProgramBankDumpRequest => "Program Bank Dump Request",
            Self::BankProgramNamesDumpRequest => "Bank Program Names Dump Request",
            Self::BankProgramNamesDumpResponse => "Bank Program Names Dump Response",
            Self::SingleProgramNameDumpRequest => "Single Program Name Dump Request",
            Self::SingleProgramNameDumpResponse => "Single Program Name Dump Response",
            Self::EditBufferPatternDumpRequest => "Edit Buffer Pattern Dump Request",
            Self::ControlAppNotifyResponse => "Control App Notify Response",
            Self::CalibrationDataDumpRequest => "Calibration Data Dump Request",
            Self::CalibrationDataDumpResponse => "Calibration Data Dump Response",
            Self::ChordMemoryDumpRequest => "Chord Memory Dump Request",
            Self::ChordMemoryDumpResponse => "Chord Memory Dump Response",
            Self::PolyChordMemoryDumpRequest => "Poly Chord Memory Dump Request",
            Self::PolyChordMemoryDumpResponse => "Poly Chord Memory Dump Response",
        }
    }

    /// Returns which way round the wire the message travels.
    #[must_use]
    pub const fn direction(self) -> Direction {
        match self {
            Self::ProgramDumpResponse
            | Self::EditBufferDumpResponse
            | Self::GlobalParameterDumpResponse
            | Self::UserPatternDumpResponse
            | Self::BankProgramNamesDumpResponse
            | Self::SingleProgramNameDumpResponse
            | Self::ControlAppNotifyResponse
            | Self::CalibrationDataDumpResponse
            | Self::ChordMemoryDumpResponse
            | Self::PolyChordMemoryDumpResponse => Direction::FromDevice,
            _ => Direction::ToDevice,
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_command_round_trips_through_its_byte() {
        for command in Command::ALL {
            assert_eq!(Command::from_byte(command.to_byte()), Ok(command));
        }
    }

    #[test]
    fn the_table_is_ordered_by_command_byte_and_has_no_duplicates() {
        let mut previous = None;
        for command in Command::ALL {
            if let Some(previous) = previous {
                assert!(command.to_byte() > previous, "{command} is out of order");
            }
            previous = Some(command.to_byte());
        }
    }

    #[test]
    fn the_gaps_in_the_command_range_are_rejected() {
        for byte in [0x0F, 0x13, 0x1A, 0x1F, 0x7F] {
            assert_eq!(Command::from_byte(byte), Err(Error::UnknownCommand(byte)));
        }
    }

    #[test]
    fn a_response_comes_from_the_device_and_a_request_goes_to_it() {
        for command in Command::ALL {
            let expected = if command.name().ends_with("Response") {
                Direction::FromDevice
            } else {
                Direction::ToDevice
            };
            assert_eq!(command.direction(), expected, "{command}");
        }
    }
}
