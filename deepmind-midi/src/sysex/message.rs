//! `DeepMind` `SysEx` framing and the typed messages inside it.

use core::fmt;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::error::{Error, Result};
use crate::ids::{
    Bank, DeviceId, MANUFACTURER_ID, Model, PatternNumber, ProgramNumber, ProtocolVersion,
};
use crate::wire::{SYSEX_END, SYSEX_START};

use super::command::Command;
use super::packed;

/// Bytes every frame carries before its payload: `F0`, the manufacturer ID, the
/// model ID, the device ID and the command.
pub const HEADER_LEN: usize = 7;

/// Bytes a frame carries besides its payload, the closing `F7` included.
pub const OVERHEAD_LEN: usize = HEADER_LEN + 1;

/// Unpacked length of a global settings dump.
pub const GLOBAL_DATA_LEN: usize = 45;

/// Unpacked length of a user pattern: a length byte, 32 velocities, 32 gates.
pub const PATTERN_DATA_LEN: usize = 65;

/// Unpacked length of one program name.
pub const PROGRAM_NAME_LEN: usize = 16;

/// Unpacked length of a bank's program names, 128 of them.
pub const BANK_NAMES_LEN: usize = PROGRAM_NAME_LEN * 128;

/// Unpacked length of the chord memory.
pub const CHORD_MEMORY_LEN: usize = 26;

/// Unpacked length of the poly chord memory.
pub const POLY_CHORD_MEMORY_LEN: usize = 512;

/// Which of the synthesizer's three MIDI interfaces a message concerns.
///
/// They are not interchangeable: each keeps its own NRPN selected-parameter
/// register, and a control application is announced per interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Interface {
    /// The DIN sockets.
    Midi,
    /// The USB port.
    Usb,
    /// The Wi-Fi module.
    WiFi,
}

impl Interface {
    /// Builds an interface from its wire byte.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidInterface`] for anything but 0, 1 or 2.
    pub const fn from_byte(byte: u8) -> Result<Self> {
        Ok(match byte {
            0 => Self::Midi,
            1 => Self::Usb,
            2 => Self::WiFi,
            _ => return Err(Error::InvalidInterface(byte)),
        })
    }

    /// Returns the wire byte for this interface.
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        match self {
            Self::Midi => 0,
            Self::Usb => 1,
            Self::WiFi => 2,
        }
    }
}

impl TryFrom<u8> for Interface {
    type Error = Error;

    fn try_from(byte: u8) -> Result<Self> {
        Self::from_byte(byte)
    }
}

impl From<Interface> for u8 {
    fn from(value: Interface) -> Self {
        value.to_byte()
    }
}

impl fmt::Display for Interface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Midi => "MIDI",
            Self::Usb => "USB",
            Self::WiFi => "Wi-Fi",
        })
    }
}

/// A `DeepMind` `SysEx` message, borrowing any bulk payload where it lies.
///
/// Bulk payloads stay packed. Unpacking needs somewhere to put 242 or 2048
/// bytes, which a `no_std` host has to choose for itself, so
/// [`packed::unpack_into`] is a separate step the caller drives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Message<'a> {
    /// Announces a control application on the interface it arrives over.
    ControlAppNotifyRequest {
        /// The one reserved payload byte, preserved rather than assumed zero.
        reserved: u8,
    },
    /// Asks for one stored program.
    ProgramDumpRequest {
        /// Bank to read from.
        bank: Bank,
        /// Program within the bank.
        program: ProgramNumber,
    },
    /// Carries one stored program.
    ProgramDumpResponse {
        /// Comms protocol version, which decides the unpacked length.
        version: ProtocolVersion,
        /// Bank the program came from.
        bank: Bank,
        /// Program within the bank.
        program: ProgramNumber,
        /// Packed program data.
        packed: &'a [u8],
    },
    /// Asks for the edit buffer, the program as it currently sounds.
    EditBufferDumpRequest,
    /// Carries the edit buffer.
    EditBufferDumpResponse {
        /// Comms protocol version, which decides the unpacked length.
        version: ProtocolVersion,
        /// Packed program data.
        packed: &'a [u8],
    },
    /// Asks for the device-wide settings.
    GlobalParameterDumpRequest,
    /// Carries the device-wide settings.
    GlobalParameterDumpResponse {
        /// Comms protocol version.
        version: ProtocolVersion,
        /// Packed global data, [`GLOBAL_DATA_LEN`] bytes unpacked.
        packed: &'a [u8],
    },
    /// Asks for one stored user pattern.
    SingleUserPatternDumpRequest {
        /// Pattern slot to read.
        pattern: PatternNumber,
    },
    /// Carries a user pattern, or the pattern edit buffer.
    ///
    /// One command answers both requests. The stored form names its slot and the
    /// edit buffer form does not, so the payload is one byte longer for a stored
    /// pattern and that is what tells them apart.
    UserPatternDumpResponse {
        /// Comms protocol version.
        version: ProtocolVersion,
        /// Slot the pattern came from, absent for the edit buffer.
        pattern: Option<PatternNumber>,
        /// Packed pattern data, [`PATTERN_DATA_LEN`] bytes unpacked.
        packed: &'a [u8],
    },
    /// Asks for a run of programs, answered one program dump at a time.
    ProgramBankDumpRequest {
        /// Bank to read from.
        bank: Bank,
        /// First program of the run.
        first: ProgramNumber,
        /// Last program of the run.
        last: ProgramNumber,
    },
    /// Asks for the names of every program in a bank.
    BankProgramNamesDumpRequest {
        /// Bank to read.
        bank: Bank,
    },
    /// Carries 128 program names.
    BankProgramNamesDumpResponse {
        /// Comms protocol version.
        version: ProtocolVersion,
        /// Bank the names came from.
        bank: Bank,
        /// Packed names, [`BANK_NAMES_LEN`] bytes unpacked.
        packed: &'a [u8],
    },
    /// Asks for one program's name.
    SingleProgramNameDumpRequest {
        /// Bank to read from.
        bank: Bank,
        /// Program within the bank.
        program: ProgramNumber,
    },
    /// Carries one program's name.
    ///
    /// The manual gives this payload a version and a bank byte and no program
    /// number, so the reply does not say which program it describes and a host
    /// has to remember what it asked for. Transcribed as printed; unverified.
    SingleProgramNameDumpResponse {
        /// Comms protocol version.
        version: ProtocolVersion,
        /// Bank the name came from.
        bank: Bank,
        /// Packed name, [`PROGRAM_NAME_LEN`] bytes unpacked.
        packed: &'a [u8],
    },
    /// Asks for the pattern edit buffer.
    EditBufferPatternDumpRequest,
    /// Answers [`Message::ControlAppNotifyRequest`] with the interface's state.
    ControlAppNotifyResponse {
        /// Receive channel, as the manual prints it: 0 through 16.
        rx_channel: u8,
        /// Transmit channel, 0 through 15.
        tx_channel: u8,
        /// Interface this answer describes.
        interface: Interface,
        /// Bank currently selected.
        bank: Bank,
        /// Program currently selected.
        program: ProgramNumber,
    },
    /// Asks for the voice and controller calibration data.
    CalibrationDataDumpRequest,
    /// Carries the calibration data.
    ///
    /// The manual gives this one a packed length and no unpacked length, so
    /// nothing here checks it.
    CalibrationDataDumpResponse {
        /// Comms protocol version.
        version: ProtocolVersion,
        /// Packed calibration data.
        packed: &'a [u8],
    },
    /// Asks for the chord memory.
    ChordMemoryDumpRequest,
    /// Carries the chord memory. Unused locations are `0xFF`.
    ChordMemoryDumpResponse {
        /// Comms protocol version.
        version: ProtocolVersion,
        /// Packed chord memory, [`CHORD_MEMORY_LEN`] bytes unpacked.
        packed: &'a [u8],
    },
    /// Asks for the poly chord memory.
    PolyChordMemoryDumpRequest,
    /// Carries the poly chord memory. Unused locations are `0xFF`.
    PolyChordMemoryDumpResponse {
        /// Comms protocol version.
        version: ProtocolVersion,
        /// Packed poly chord memory, [`POLY_CHORD_MEMORY_LEN`] bytes unpacked.
        packed: &'a [u8],
    },
}

impl<'a> Message<'a> {
    /// Returns the command byte this message is carried under.
    #[must_use]
    pub const fn command(&self) -> Command {
        match self {
            Self::ControlAppNotifyRequest { .. } => Command::ControlAppNotifyRequest,
            Self::ProgramDumpRequest { .. } => Command::ProgramDumpRequest,
            Self::ProgramDumpResponse { .. } => Command::ProgramDumpResponse,
            Self::EditBufferDumpRequest => Command::EditBufferDumpRequest,
            Self::EditBufferDumpResponse { .. } => Command::EditBufferDumpResponse,
            Self::GlobalParameterDumpRequest => Command::GlobalParameterDumpRequest,
            Self::GlobalParameterDumpResponse { .. } => Command::GlobalParameterDumpResponse,
            Self::SingleUserPatternDumpRequest { .. } => Command::SingleUserPatternDumpRequest,
            Self::UserPatternDumpResponse { .. } => Command::UserPatternDumpResponse,
            Self::ProgramBankDumpRequest { .. } => Command::ProgramBankDumpRequest,
            Self::BankProgramNamesDumpRequest { .. } => Command::BankProgramNamesDumpRequest,
            Self::BankProgramNamesDumpResponse { .. } => Command::BankProgramNamesDumpResponse,
            Self::SingleProgramNameDumpRequest { .. } => Command::SingleProgramNameDumpRequest,
            Self::SingleProgramNameDumpResponse { .. } => Command::SingleProgramNameDumpResponse,
            Self::EditBufferPatternDumpRequest => Command::EditBufferPatternDumpRequest,
            Self::ControlAppNotifyResponse { .. } => Command::ControlAppNotifyResponse,
            Self::CalibrationDataDumpRequest => Command::CalibrationDataDumpRequest,
            Self::CalibrationDataDumpResponse { .. } => Command::CalibrationDataDumpResponse,
            Self::ChordMemoryDumpRequest => Command::ChordMemoryDumpRequest,
            Self::ChordMemoryDumpResponse { .. } => Command::ChordMemoryDumpResponse,
            Self::PolyChordMemoryDumpRequest => Command::PolyChordMemoryDumpRequest,
            Self::PolyChordMemoryDumpResponse { .. } => Command::PolyChordMemoryDumpResponse,
        }
    }

    /// Returns the packed bulk payload, for the messages that carry one.
    #[must_use]
    pub const fn packed(&self) -> Option<&'a [u8]> {
        match *self {
            Self::ProgramDumpResponse { packed, .. }
            | Self::EditBufferDumpResponse { packed, .. }
            | Self::GlobalParameterDumpResponse { packed, .. }
            | Self::UserPatternDumpResponse { packed, .. }
            | Self::BankProgramNamesDumpResponse { packed, .. }
            | Self::SingleProgramNameDumpResponse { packed, .. }
            | Self::CalibrationDataDumpResponse { packed, .. }
            | Self::ChordMemoryDumpResponse { packed, .. }
            | Self::PolyChordMemoryDumpResponse { packed, .. } => Some(packed),
            _ => None,
        }
    }

    /// Returns the comms protocol version, for the messages that carry one.
    #[must_use]
    pub const fn version(&self) -> Option<ProtocolVersion> {
        match *self {
            Self::ProgramDumpResponse { version, .. }
            | Self::EditBufferDumpResponse { version, .. }
            | Self::GlobalParameterDumpResponse { version, .. }
            | Self::UserPatternDumpResponse { version, .. }
            | Self::BankProgramNamesDumpResponse { version, .. }
            | Self::SingleProgramNameDumpResponse { version, .. }
            | Self::CalibrationDataDumpResponse { version, .. }
            | Self::ChordMemoryDumpResponse { version, .. }
            | Self::PolyChordMemoryDumpResponse { version, .. } => Some(version),
            _ => None,
        }
    }

    /// Returns the payload bytes before the packed run, and the run itself.
    fn parts(&self) -> ([u8; 5], usize, &'a [u8]) {
        let mut prefix = [0u8; 5];
        let (len, packed): (usize, &[u8]) = match *self {
            Self::EditBufferDumpRequest
            | Self::GlobalParameterDumpRequest
            | Self::EditBufferPatternDumpRequest
            | Self::CalibrationDataDumpRequest
            | Self::ChordMemoryDumpRequest
            | Self::PolyChordMemoryDumpRequest => (0, &[]),
            Self::ControlAppNotifyRequest { reserved } => {
                prefix = [reserved, 0, 0, 0, 0];
                (1, &[])
            }
            Self::ProgramDumpRequest { bank, program }
            | Self::SingleProgramNameDumpRequest { bank, program } => {
                prefix = [bank.index(), program.get(), 0, 0, 0];
                (2, &[])
            }
            Self::SingleUserPatternDumpRequest { pattern } => {
                prefix = [pattern.get(), 0, 0, 0, 0];
                (1, &[])
            }
            Self::BankProgramNamesDumpRequest { bank } => {
                prefix = [bank.index(), 0, 0, 0, 0];
                (1, &[])
            }
            Self::ProgramBankDumpRequest { bank, first, last } => {
                prefix = [bank.index(), first.get(), last.get(), 0, 0];
                (3, &[])
            }
            Self::ProgramDumpResponse {
                version,
                bank,
                program,
                packed,
            } => {
                prefix = [version.get(), bank.index(), program.get(), 0, 0];
                (3, packed)
            }
            Self::EditBufferDumpResponse { version, packed }
            | Self::GlobalParameterDumpResponse { version, packed }
            | Self::CalibrationDataDumpResponse { version, packed }
            | Self::ChordMemoryDumpResponse { version, packed }
            | Self::PolyChordMemoryDumpResponse { version, packed } => {
                prefix = [version.get(), 0, 0, 0, 0];
                (1, packed)
            }
            Self::BankProgramNamesDumpResponse {
                version,
                bank,
                packed,
            }
            | Self::SingleProgramNameDumpResponse {
                version,
                bank,
                packed,
            } => {
                prefix = [version.get(), bank.index(), 0, 0, 0];
                (2, packed)
            }
            Self::UserPatternDumpResponse {
                version,
                pattern,
                packed,
            } => {
                if let Some(pattern) = pattern {
                    prefix = [version.get(), pattern.get(), 0, 0, 0];
                    (2, packed)
                } else {
                    prefix = [version.get(), 0, 0, 0, 0];
                    (1, packed)
                }
            }
            Self::ControlAppNotifyResponse {
                rx_channel,
                tx_channel,
                interface,
                bank,
                program,
            } => {
                prefix = [
                    rx_channel,
                    tx_channel,
                    interface.to_byte(),
                    bank.index(),
                    program.get(),
                ];
                (5, &[])
            }
        };
        (prefix, len, packed)
    }

    /// Reads a message out of the payload that follows a command byte.
    fn parse(command: Command, payload: &'a [u8]) -> Result<Self> {
        match command {
            Command::ProgramDumpResponse
            | Command::EditBufferDumpResponse
            | Command::GlobalParameterDumpResponse
            | Command::UserPatternDumpResponse
            | Command::BankProgramNamesDumpResponse
            | Command::SingleProgramNameDumpResponse
            | Command::CalibrationDataDumpResponse
            | Command::ChordMemoryDumpResponse
            | Command::PolyChordMemoryDumpResponse => Self::parse_dump(command, payload),
            _ => Self::parse_short(command, payload),
        }
    }

    /// Reads a message whose payload is a fixed handful of bytes.
    fn parse_short(command: Command, payload: &'a [u8]) -> Result<Self> {
        let wrong = wrong_length(command, payload.len());
        Ok(match command {
            Command::ControlAppNotifyRequest => match *payload {
                [reserved] => Self::ControlAppNotifyRequest { reserved },
                _ => return Err(wrong(1)),
            },
            Command::ProgramDumpRequest => match *payload {
                [bank, program] => Self::ProgramDumpRequest {
                    bank: Bank::new(bank)?,
                    program: ProgramNumber::new(program)?,
                },
                _ => return Err(wrong(2)),
            },
            Command::SingleProgramNameDumpRequest => match *payload {
                [bank, program] => Self::SingleProgramNameDumpRequest {
                    bank: Bank::new(bank)?,
                    program: ProgramNumber::new(program)?,
                },
                _ => return Err(wrong(2)),
            },
            Command::ProgramBankDumpRequest => match *payload {
                [bank, first, last] => Self::ProgramBankDumpRequest {
                    bank: Bank::new(bank)?,
                    first: ProgramNumber::new(first)?,
                    last: ProgramNumber::new(last)?,
                },
                _ => return Err(wrong(3)),
            },
            Command::SingleUserPatternDumpRequest => match *payload {
                [pattern] => Self::SingleUserPatternDumpRequest {
                    pattern: PatternNumber::new(pattern)?,
                },
                _ => return Err(wrong(1)),
            },
            Command::BankProgramNamesDumpRequest => match *payload {
                [bank] => Self::BankProgramNamesDumpRequest {
                    bank: Bank::new(bank)?,
                },
                _ => return Err(wrong(1)),
            },
            Command::ControlAppNotifyResponse => match *payload {
                [rx_channel, tx_channel, interface, bank, program] => {
                    Self::ControlAppNotifyResponse {
                        rx_channel,
                        tx_channel,
                        interface: Interface::from_byte(interface)?,
                        bank: Bank::new(bank)?,
                        program: ProgramNumber::new(program)?,
                    }
                }
                _ => return Err(wrong(5)),
            },
            // What is left takes no payload at all.
            Command::EditBufferDumpRequest
            | Command::GlobalParameterDumpRequest
            | Command::EditBufferPatternDumpRequest
            | Command::CalibrationDataDumpRequest
            | Command::ChordMemoryDumpRequest
            | Command::PolyChordMemoryDumpRequest => {
                if !payload.is_empty() {
                    return Err(wrong(0));
                }
                match command {
                    Command::EditBufferDumpRequest => Self::EditBufferDumpRequest,
                    Command::GlobalParameterDumpRequest => Self::GlobalParameterDumpRequest,
                    Command::EditBufferPatternDumpRequest => Self::EditBufferPatternDumpRequest,
                    Command::CalibrationDataDumpRequest => Self::CalibrationDataDumpRequest,
                    Command::ChordMemoryDumpRequest => Self::ChordMemoryDumpRequest,
                    _ => Self::PolyChordMemoryDumpRequest,
                }
            }
            // Unreachable while `parse` dispatches on the same list, and an
            // error rather than a wrong message if that ever stops being true.
            _ => return Err(Error::UnknownCommand(command.to_byte())),
        })
    }

    /// Reads a message carrying a comms protocol version and a packed run.
    fn parse_dump(command: Command, payload: &'a [u8]) -> Result<Self> {
        let found = payload.len();
        let wrong = wrong_length(command, found);
        Ok(match command {
            Command::ProgramDumpResponse => {
                let (version, rest) = split_version(payload).ok_or_else(|| wrong(3))??;
                let [bank, program, ref packed @ ..] = *rest else {
                    return Err(wrong(3));
                };
                check_run(packed, Some(version.program_data_len()), 3, found, command)?;
                Self::ProgramDumpResponse {
                    version,
                    bank: Bank::new(bank)?,
                    program: ProgramNumber::new(program)?,
                    packed,
                }
            }
            Command::EditBufferDumpResponse => {
                let (version, packed) = split_version(payload).ok_or_else(|| wrong(1))??;
                check_run(packed, Some(version.program_data_len()), 1, found, command)?;
                Self::EditBufferDumpResponse { version, packed }
            }
            Command::GlobalParameterDumpResponse => {
                let (version, packed) = split_version(payload).ok_or_else(|| wrong(1))??;
                check_run(packed, Some(GLOBAL_DATA_LEN), 1, found, command)?;
                Self::GlobalParameterDumpResponse { version, packed }
            }
            Command::CalibrationDataDumpResponse => {
                let (version, packed) = split_version(payload).ok_or_else(|| wrong(1))??;
                check_run(packed, None, 1, found, command)?;
                Self::CalibrationDataDumpResponse { version, packed }
            }
            Command::ChordMemoryDumpResponse => {
                let (version, packed) = split_version(payload).ok_or_else(|| wrong(1))??;
                check_run(packed, Some(CHORD_MEMORY_LEN), 1, found, command)?;
                Self::ChordMemoryDumpResponse { version, packed }
            }
            Command::PolyChordMemoryDumpResponse => {
                let (version, packed) = split_version(payload).ok_or_else(|| wrong(1))??;
                check_run(packed, Some(POLY_CHORD_MEMORY_LEN), 1, found, command)?;
                Self::PolyChordMemoryDumpResponse { version, packed }
            }
            Command::BankProgramNamesDumpResponse => {
                let (version, bank, packed) =
                    split_version_and_bank(payload).ok_or_else(|| wrong(2))??;
                check_run(packed, Some(BANK_NAMES_LEN), 2, found, command)?;
                Self::BankProgramNamesDumpResponse {
                    version,
                    bank: Bank::new(bank)?,
                    packed,
                }
            }
            Command::SingleProgramNameDumpResponse => {
                let (version, bank, packed) =
                    split_version_and_bank(payload).ok_or_else(|| wrong(2))??;
                check_run(packed, Some(PROGRAM_NAME_LEN), 2, found, command)?;
                Self::SingleProgramNameDumpResponse {
                    version,
                    bank: Bank::new(bank)?,
                    packed,
                }
            }
            Command::UserPatternDumpResponse => {
                let (version, rest) = split_version(payload).ok_or_else(|| wrong(1))??;
                let (pattern, packed) = split_pattern(rest, PATTERN_DATA_LEN)
                    .ok_or_else(|| wrong(1 + packed::packed_len(PATTERN_DATA_LEN)))?;
                Self::UserPatternDumpResponse {
                    version,
                    pattern: match pattern {
                        Some(number) => Some(PatternNumber::new(number)?),
                        None => None,
                    },
                    packed,
                }
            }
            // Unreachable while `parse` dispatches on the same list, and an
            // error rather than a wrong message if that ever stops being true.
            _ => return Err(Error::UnknownCommand(command.to_byte())),
        })
    }
}

/// Returns a function building the length rejection for `command`.
fn wrong_length(command: Command, found: usize) -> impl Fn(usize) -> Error {
    move |expected| Error::PayloadLength {
        command: command.to_byte(),
        expected,
        found,
    }
}

/// Splits the comms protocol version byte off the front of a payload.
///
/// `None` for an empty payload, which is a length error for the caller to
/// name, and [`Error::UnsupportedProtocolVersion`] for a version this build
/// does not know the dump layout of.
fn split_version(payload: &[u8]) -> Option<Result<(ProtocolVersion, &[u8])>> {
    let (version, rest) = payload.split_first()?;
    Some(ProtocolVersion::new(*version).map(|version| (version, rest)))
}

/// Splits the version and bank bytes off the front of a payload.
fn split_version_and_bank(payload: &[u8]) -> Option<Result<(ProtocolVersion, u8, &[u8])>> {
    let (bank, packed) = payload.get(1..)?.split_first()?;
    Some(split_version(payload)?.map(|(version, _)| (version, *bank, packed)))
}

/// Tells the two user pattern dump responses apart by length.
///
/// The stored form names its slot, the edit buffer form does not, and the
/// command byte is the same. Whichever reading leaves a packed run that can hold
/// `raw_len` bytes is the right one. The edit buffer reading is tried first, so
/// a length both readings accept (which needs the synthesizer to send a short
/// last group rather than a padded one) is taken as the edit buffer.
fn split_pattern(payload: &[u8], raw_len: usize) -> Option<(Option<u8>, &[u8])> {
    if holds(payload, raw_len) {
        return Some((None, payload));
    }
    let (pattern, packed) = payload.split_first()?;
    holds(packed, raw_len).then_some((Some(*pattern), packed))
}

/// Returns whether a packed run can hold `raw_len` bytes and no whole group more.
fn holds(packed: &[u8], raw_len: usize) -> bool {
    match packed::unpacked_len(packed.len()) {
        Ok(available) => available >= raw_len && available < raw_len + packed::RAW_GROUP,
        Err(_) => false,
    }
}

/// Checks a packed run against the unpacked length its message declares.
///
/// A run is accepted when it can hold that many bytes and is no more than one
/// short group longer, which covers a last group padded out to eight bytes and
/// one sent short. `None` only asks that the run unpacks at all, for the one
/// message whose unpacked length the manual does not give.
fn check_run(
    packed: &[u8],
    raw_len: Option<usize>,
    prefix: usize,
    payload_len: usize,
    command: Command,
) -> Result<()> {
    let Some(raw_len) = raw_len else {
        return packed::unpacked_len(packed.len()).map(|_| ());
    };
    if holds(packed, raw_len) {
        return Ok(());
    }
    Err(Error::PayloadLength {
        command: command.to_byte(),
        expected: prefix + packed::packed_len(raw_len),
        found: payload_len,
    })
}

/// A whole `SysEx` frame: who it is addressed to, and what it says.
///
/// ```
/// use deepmind_midi::ids::{Bank, DeviceId, ProgramNumber};
/// use deepmind_midi::sysex::{Frame, Message};
///
/// let request = Frame::new(
///     DeviceId::Unit(0),
///     Message::ProgramDumpRequest {
///         bank: Bank::A,
///         program: ProgramNumber::FIRST,
///     },
/// );
///
/// let mut bytes = [0u8; 16];
/// let len = request.encode_into(&mut bytes).expect("the buffer fits");
/// assert_eq!(
///     bytes.get(..len),
///     Some([0xF0, 0x00, 0x20, 0x32, 0x20, 0x00, 0x01, 0x00, 0x00, 0xF7].as_slice())
/// );
/// assert_eq!(Frame::parse(&bytes[..len]), Ok(request));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame<'a> {
    /// Unit the frame is addressed to.
    pub device: DeviceId,
    /// What the frame says.
    pub message: Message<'a>,
}

impl<'a> Frame<'a> {
    /// Builds a frame.
    #[must_use]
    pub const fn new(device: DeviceId, message: Message<'a>) -> Self {
        Self { device, message }
    }

    /// Reads a frame from `F0` through `F7`.
    ///
    /// Bulk payloads are borrowed from `bytes`, still packed.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Unframed`] when the `F0` or the `F7` is missing,
    /// [`Error::NotSevenBit`] when a byte between them has its high bit set,
    /// [`Error::ShortFrame`] when there is no room for a header,
    /// [`Error::Foreign`] for another manufacturer's or model's frame,
    /// [`Error::UnknownCommand`] for an undocumented command, and
    /// [`Error::PayloadLength`] when the payload does not fit the command.
    pub fn parse(bytes: &'a [u8]) -> Result<Self> {
        let (first, rest) = bytes.split_first().ok_or(Error::Unframed)?;
        let (last, body) = rest.split_last().ok_or(Error::Unframed)?;
        if *first != SYSEX_START || *last != SYSEX_END {
            return Err(Error::Unframed);
        }
        // A port never delivers one, since a status byte ends a frame; a file
        // or a caller can, and a frame holding one is not a frame.
        if let Some(byte) = body.iter().find(|byte| **byte >= 0x80) {
            return Err(Error::NotSevenBit(*byte));
        }
        let [a, b, c, model, device, command, payload @ ..] = body else {
            return Err(Error::ShortFrame(bytes.len()));
        };
        let manufacturer = [*a, *b, *c];
        if manufacturer != MANUFACTURER_ID || *model != Model::MODEL_ID {
            return Err(Error::Foreign {
                manufacturer,
                model: *model,
            });
        }
        Ok(Self {
            device: DeviceId::from_byte(*device)?,
            message: Message::parse(Command::from_byte(*command)?, payload)?,
        })
    }

    /// Returns the number of bytes [`encode_into`](Self::encode_into) writes.
    #[must_use]
    pub fn encoded_len(&self) -> usize {
        let (_, prefix, packed) = self.message.parts();
        OVERHEAD_LEN + prefix + packed.len()
    }

    /// Writes the frame to `out`, `F0` through `F7`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::BufferTooSmall`] when `out` is shorter than
    /// [`encoded_len`](Self::encoded_len), and [`Error::NotSevenBit`] when the
    /// message holds a byte that cannot travel inside a frame: a channel or a
    /// reserved byte of 128 or more, or a packed run that was not packed. Nothing
    /// is written in either case.
    pub fn encode_into(&self, out: &mut [u8]) -> Result<usize> {
        let needed = self.encoded_len();
        let available = out.len();
        let out = out
            .get_mut(..needed)
            .ok_or(Error::BufferTooSmall { needed, available })?;
        let (prefix, prefix_len, packed) = self.message.parts();
        let prefix = prefix.get(..prefix_len).unwrap_or_default();
        if let Some(byte) = prefix.iter().chain(packed).find(|byte| **byte >= 0x80) {
            return Err(Error::NotSevenBit(*byte));
        }
        let header = [
            SYSEX_START,
            MANUFACTURER_ID[0],
            MANUFACTURER_ID[1],
            MANUFACTURER_ID[2],
            Model::MODEL_ID,
            self.device.to_byte(),
            self.message.command().to_byte(),
        ];
        let (head, rest) = out.split_at_mut(header.len().min(out.len()));
        head.copy_from_slice(header.get(..head.len()).unwrap_or_default());
        let (front, rest) = rest.split_at_mut(prefix.len().min(rest.len()));
        front.copy_from_slice(prefix.get(..front.len()).unwrap_or_default());
        let (body, tail) = rest.split_at_mut(packed.len().min(rest.len()));
        body.copy_from_slice(packed.get(..body.len()).unwrap_or_default());
        if let Some(end) = tail.first_mut() {
            *end = SYSEX_END;
        }
        Ok(needed)
    }

    /// Writes the frame to a new vector.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[must_use]
    pub fn to_vec(&self) -> Vec<u8> {
        let mut out = alloc::vec![0; self.encoded_len()];
        let written = self.encode_into(&mut out).unwrap_or(0);
        out.truncate(written);
        out
    }
}

#[cfg(all(test, feature = "alloc"))]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use super::*;

    fn program(n: u8) -> ProgramNumber {
        ProgramNumber::new(n).expect("a program in range")
    }

    fn pattern(n: u8) -> PatternNumber {
        PatternNumber::new(n).expect("a pattern in range")
    }

    /// A packed run long enough to unpack to `raw_len` bytes, padded as the
    /// synthesizer's own figures say it pads.
    fn run(raw_len: usize) -> Vec<u8> {
        alloc::vec![0x7F; packed::packed_len(raw_len)]
    }

    /// Every message, with payloads of the documented lengths.
    fn every_message(runs: &'_ [Vec<u8>]) -> Vec<Message<'_>> {
        let (program_data, globals, pattern_data, names, name, chord, poly, calibration) = (
            runs.first().expect("a program run").as_slice(),
            runs.get(1).expect("a globals run").as_slice(),
            runs.get(2).expect("a pattern run").as_slice(),
            runs.get(3).expect("a names run").as_slice(),
            runs.get(4).expect("a name run").as_slice(),
            runs.get(5).expect("a chord run").as_slice(),
            runs.get(6).expect("a poly chord run").as_slice(),
            runs.get(7).expect("a calibration run").as_slice(),
        );
        alloc::vec![
            Message::ControlAppNotifyRequest { reserved: 0 },
            Message::ProgramDumpRequest {
                bank: Bank::H,
                program: program(127),
            },
            Message::ProgramDumpResponse {
                version: ProtocolVersion::V7,
                bank: Bank::A,
                program: program(0),
                packed: program_data,
            },
            Message::EditBufferDumpRequest,
            Message::EditBufferDumpResponse {
                version: ProtocolVersion::V7,
                packed: program_data,
            },
            Message::GlobalParameterDumpRequest,
            Message::GlobalParameterDumpResponse {
                version: ProtocolVersion::V7,
                packed: globals,
            },
            Message::SingleUserPatternDumpRequest {
                pattern: pattern(31),
            },
            Message::UserPatternDumpResponse {
                version: ProtocolVersion::V7,
                pattern: Some(pattern(5)),
                packed: pattern_data,
            },
            Message::UserPatternDumpResponse {
                version: ProtocolVersion::V7,
                pattern: None,
                packed: pattern_data,
            },
            Message::ProgramBankDumpRequest {
                bank: Bank::A,
                first: program(0),
                last: program(127),
            },
            Message::BankProgramNamesDumpRequest { bank: Bank::A },
            Message::BankProgramNamesDumpResponse {
                version: ProtocolVersion::V7,
                bank: Bank::A,
                packed: names,
            },
            Message::SingleProgramNameDumpRequest {
                bank: Bank::A,
                program: program(3),
            },
            Message::SingleProgramNameDumpResponse {
                version: ProtocolVersion::V7,
                bank: Bank::A,
                packed: name,
            },
            Message::EditBufferPatternDumpRequest,
            Message::ControlAppNotifyResponse {
                rx_channel: 16,
                tx_channel: 15,
                interface: Interface::WiFi,
                bank: Bank::H,
                program: program(127),
            },
            Message::CalibrationDataDumpRequest,
            Message::CalibrationDataDumpResponse {
                version: ProtocolVersion::V7,
                packed: calibration,
            },
            Message::ChordMemoryDumpRequest,
            Message::ChordMemoryDumpResponse {
                version: ProtocolVersion::V7,
                packed: chord,
            },
            Message::PolyChordMemoryDumpRequest,
            Message::PolyChordMemoryDumpResponse {
                version: ProtocolVersion::V7,
                packed: poly,
            },
        ]
    }

    fn runs() -> Vec<Vec<u8>> {
        alloc::vec![
            run(245),
            run(GLOBAL_DATA_LEN),
            run(PATTERN_DATA_LEN),
            run(BANK_NAMES_LEN),
            run(PROGRAM_NAME_LEN),
            run(CHORD_MEMORY_LEN),
            run(POLY_CHORD_MEMORY_LEN),
            alloc::vec![0x7F; 688],
        ]
    }

    #[test]
    fn every_message_round_trips_through_its_frame() {
        let runs = runs();
        for message in every_message(&runs) {
            for device in [DeviceId::Unit(0), DeviceId::Unit(15), DeviceId::Broadcast] {
                let frame = Frame::new(device, message);
                let bytes = frame.to_vec();
                assert_eq!(bytes.len(), frame.encoded_len());
                assert_eq!(bytes.first(), Some(&SYSEX_START));
                assert_eq!(bytes.last(), Some(&SYSEX_END));
                assert!(
                    bytes
                        .get(1..bytes.len() - 1)
                        .is_some_and(|body| body.iter().all(|byte| *byte < 0x80)),
                    "{message:?} put a status byte inside a frame"
                );
                assert_eq!(Frame::parse(&bytes), Ok(frame), "{message:?}");
            }
        }
    }

    /// A byte with its high bit set would end the frame early on the wire, so
    /// a message holding one is refused rather than written.
    #[test]
    fn a_message_that_cannot_travel_inside_a_frame_is_refused() {
        let mut out = [0; 512];
        let bad_channel = Frame::new(
            DeviceId::Unit(0),
            Message::ControlAppNotifyResponse {
                rx_channel: 0x90,
                tx_channel: 0,
                interface: Interface::Midi,
                bank: Bank::A,
                program: ProgramNumber::FIRST,
            },
        );
        assert_eq!(
            bad_channel.encode_into(&mut out),
            Err(Error::NotSevenBit(0x90))
        );
        assert!(out.iter().all(|byte| *byte == 0), "nothing is written");

        let unpacked = [0xFF; 280];
        let bad_run = Frame::new(
            DeviceId::Unit(0),
            Message::EditBufferDumpResponse {
                version: ProtocolVersion::V6,
                packed: &unpacked,
            },
        );
        assert_eq!(bad_run.encode_into(&mut out), Err(Error::NotSevenBit(0xFF)));
    }

    /// The one dump with no documented length still has to unpack.
    #[test]
    fn a_calibration_dump_that_cannot_unpack_is_refused() {
        let mut bytes = alloc::vec![0xF0, 0x00, 0x20, 0x32, 0x20, 0x00, 0x12, 0x07];
        bytes.extend_from_slice(&[0; 9]);
        bytes.push(0xF7);
        assert_eq!(Frame::parse(&bytes), Err(Error::PackedRunLength(9)));
    }

    /// A dump stamped with a version this build has no layout for is refused
    /// where it is parsed, since nothing downstream could read it.
    #[test]
    fn an_unknown_protocol_version_is_refused_at_the_frame() {
        let mut bytes = alloc::vec![0xF0, 0x00, 0x20, 0x32, 0x20, 0x00, 0x04, 0x09];
        bytes.extend_from_slice(&[0; 280]);
        bytes.push(0xF7);
        assert_eq!(
            Frame::parse(&bytes),
            Err(Error::UnsupportedProtocolVersion(9))
        );
    }

    #[test]
    fn every_command_the_table_knows_is_reachable_from_a_message() {
        let runs = runs();
        let mut seen = alloc::vec::Vec::new();
        for message in every_message(&runs) {
            seen.push(message.command());
        }
        for command in Command::ALL {
            assert!(seen.contains(command), "{command} has no message");
        }
    }

    #[test]
    fn a_program_dump_is_measured_against_its_own_protocol_version() {
        let v6 = run(242);
        let v7 = run(245);
        // Both pad to the same 280 bytes, which is why the version byte and not
        // the length is what says how much program data is really there.
        assert_eq!(v6.len(), v7.len());
        for version in [ProtocolVersion::V6, ProtocolVersion::V7] {
            let message = Message::EditBufferDumpResponse {
                version,
                packed: &v7,
            };
            let bytes = Frame::new(DeviceId::Unit(0), message).to_vec();
            assert_eq!(Frame::parse(&bytes).map(|frame| frame.message), Ok(message));
        }
    }

    #[test]
    fn a_last_group_sent_short_is_accepted_as_readily_as_a_padded_one() {
        let padded = run(242);
        for len in [277, 278, 280] {
            let packed = padded.get(..len).expect("a run to cut down");
            let message = Message::EditBufferDumpResponse {
                version: ProtocolVersion::V6,
                packed,
            };
            let bytes = Frame::new(DeviceId::Unit(0), message).to_vec();
            assert_eq!(
                Frame::parse(&bytes).map(|frame| frame.message),
                Ok(message),
                "a {len} byte run"
            );
        }
        // A run that cannot hold 242 bytes is not a program, whatever it is.
        let short = padded.get(..272).expect("a run to cut down");
        assert!(matches!(
            Frame::parse(
                &Frame::new(
                    DeviceId::Unit(0),
                    Message::EditBufferDumpResponse {
                        version: ProtocolVersion::V6,
                        packed: short,
                    },
                )
                .to_vec()
            ),
            Err(Error::PayloadLength { .. })
        ));
    }

    #[test]
    fn the_two_pattern_responses_are_told_apart_by_length() {
        let packed = run(PATTERN_DATA_LEN);
        let stored = Message::UserPatternDumpResponse {
            version: ProtocolVersion::V7,
            pattern: Some(pattern(7)),
            packed: &packed,
        };
        let buffer = Message::UserPatternDumpResponse {
            version: ProtocolVersion::V7,
            pattern: None,
            packed: &packed,
        };
        let stored_bytes = Frame::new(DeviceId::Unit(0), stored).to_vec();
        let buffer_bytes = Frame::new(DeviceId::Unit(0), buffer).to_vec();
        assert_eq!(stored_bytes.len(), buffer_bytes.len() + 1);
        assert_eq!(
            Frame::parse(&stored_bytes).map(|frame| frame.message),
            Ok(stored)
        );
        assert_eq!(
            Frame::parse(&buffer_bytes).map(|frame| frame.message),
            Ok(buffer)
        );
    }

    #[test]
    fn a_frame_for_another_manufacturer_or_model_is_left_alone() {
        let mut bytes = Frame::new(DeviceId::Unit(0), Message::EditBufferDumpRequest).to_vec();
        let roland = {
            let mut other = bytes.clone();
            if let Some(slot) = other.get_mut(1) {
                *slot = 0x41;
            }
            other
        };
        assert_eq!(
            Frame::parse(&roland),
            Err(Error::Foreign {
                manufacturer: [0x41, 0x20, 0x32],
                model: 0x20
            })
        );
        if let Some(slot) = bytes.get_mut(4) {
            *slot = 0x21;
        }
        assert_eq!(
            Frame::parse(&bytes),
            Err(Error::Foreign {
                manufacturer: MANUFACTURER_ID,
                model: 0x21
            })
        );
    }

    #[test]
    fn a_frame_that_is_not_framed_or_not_long_enough_is_rejected() {
        assert_eq!(Frame::parse(&[]), Err(Error::Unframed));
        assert_eq!(Frame::parse(&[0xF0]), Err(Error::Unframed));
        assert_eq!(Frame::parse(&[0xF0, 0x00]), Err(Error::Unframed));
        assert_eq!(
            Frame::parse(&[0x00, 0x20, 0x32, 0x20, 0x00, 0x03, 0xF7]),
            Err(Error::Unframed)
        );
        assert_eq!(
            Frame::parse(&[0xF0, 0x00, 0x20, 0x32, 0x20, 0x00, 0xF7]),
            Err(Error::ShortFrame(7))
        );
    }

    #[test]
    fn a_payload_that_does_not_fit_its_command_names_both_lengths() {
        // A program dump request with a program number and no bank.
        assert_eq!(
            Frame::parse(&[0xF0, 0x00, 0x20, 0x32, 0x20, 0x00, 0x01, 0x00, 0xF7]),
            Err(Error::PayloadLength {
                command: 0x01,
                expected: 2,
                found: 1
            })
        );
        // A request that takes no payload at all.
        assert_eq!(
            Frame::parse(&[0xF0, 0x00, 0x20, 0x32, 0x20, 0x00, 0x03, 0x00, 0xF7]),
            Err(Error::PayloadLength {
                command: 0x03,
                expected: 0,
                found: 1
            })
        );
    }

    #[test]
    fn out_of_range_fields_are_rejected_rather_than_wrapped() {
        // Bank 8, one past H.
        assert_eq!(
            Frame::parse(&[0xF0, 0x00, 0x20, 0x32, 0x20, 0x00, 0x01, 0x08, 0x00, 0xF7]),
            Err(Error::BankOutOfRange(8))
        );
        // Pattern 32, one past the last.
        assert_eq!(
            Frame::parse(&[0xF0, 0x00, 0x20, 0x32, 0x20, 0x00, 0x07, 0x20, 0xF7]),
            Err(Error::PatternOutOfRange(32))
        );
        // Interface 3, which is no interface.
        assert_eq!(
            Frame::parse(&[
                0xF0, 0x00, 0x20, 0x32, 0x20, 0x00, 0x10, 0x00, 0x00, 0x03, 0x00, 0x00, 0xF7
            ]),
            Err(Error::InvalidInterface(3))
        );
    }

    #[test]
    fn an_undocumented_command_is_named_rather_than_guessed_at() {
        assert_eq!(
            Frame::parse(&[0xF0, 0x00, 0x20, 0x32, 0x20, 0x00, 0x0F, 0xF7]),
            Err(Error::UnknownCommand(0x0F))
        );
    }

    #[test]
    fn a_bulk_payload_is_borrowed_where_it_lies() {
        let packed = run(GLOBAL_DATA_LEN);
        let bytes = Frame::new(
            DeviceId::Unit(0),
            Message::GlobalParameterDumpResponse {
                version: ProtocolVersion::V7,
                packed: &packed,
            },
        )
        .to_vec();
        let frame = Frame::parse(&bytes).expect("a global dump");
        let borrowed = frame.message.packed().expect("a packed payload");
        assert_eq!(borrowed, packed.as_slice());
        assert!(core::ptr::eq(
            borrowed.as_ptr(),
            bytes.as_ptr().wrapping_add(HEADER_LEN + 1)
        ));
        assert_eq!(frame.message.version(), Some(ProtocolVersion::V7));

        // A padded run unpacks to more than the dump holds: 56 packed bytes
        // carry 49, of which the last four are padding the caller drops.
        let mut unpacked = [0u8; 49];
        let written = packed::unpack_into(borrowed, &mut unpacked).expect("the run unpacks");
        assert_eq!(written, 49);
        assert_eq!(unpacked.get(..GLOBAL_DATA_LEN).map(<[u8]>::len), Some(45));
    }

    #[test]
    fn a_frame_reports_a_buffer_it_cannot_fill() {
        let frame = Frame::new(DeviceId::Unit(0), Message::EditBufferDumpRequest);
        let mut small = [0u8; 4];
        assert_eq!(
            frame.encode_into(&mut small),
            Err(Error::BufferTooSmall {
                needed: 8,
                available: 4
            })
        );
    }
}
