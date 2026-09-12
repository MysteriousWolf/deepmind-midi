//! Addressing primitives used throughout the protocol.
//!
//! Every type here is a validated newtype. Constructing one is the only place a
//! range check happens, so downstream code never has to re-check a bank or program
//! number that it already holds.

use core::fmt;

use crate::error::Error;

/// Behringer's three-byte MIDI manufacturer ID.
pub const MANUFACTURER_ID: [u8; 3] = [0x00, 0x20, 0x32];

/// Number of banks on every `DeepMind` model.
pub const BANK_COUNT: u8 = 8;

/// Number of programs in each bank.
pub const PROGRAMS_PER_BANK: u8 = 128;

/// Number of user arpeggiator/sequencer patterns.
pub const USER_PATTERN_COUNT: u8 = 32;

/// `SysEx` target address of a synthesizer on the wire.
///
/// The `DeepMind` reuses its global MIDI channel as its `SysEx` device ID. `Broadcast`
/// (`0x7F`) addresses every unit on the port, which is what factory preset packs
/// ship with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DeviceId {
    /// A specific unit, 0 through 15.
    Unit(u8),
    /// Every unit on the port (`0x7F`).
    Broadcast,
}

impl DeviceId {
    /// Wire value used for [`DeviceId::Broadcast`].
    pub const BROADCAST_BYTE: u8 = 0x7F;

    /// Builds a device ID from its wire byte.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidDeviceId`] for anything outside `0..=15` and `0x7F`.
    pub const fn from_byte(byte: u8) -> Result<Self, Error> {
        match byte {
            Self::BROADCAST_BYTE => Ok(Self::Broadcast),
            0..=15 => Ok(Self::Unit(byte)),
            _ => Err(Error::InvalidDeviceId(byte)),
        }
    }

    /// Returns the wire byte for this device ID.
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        match self {
            Self::Unit(id) => id,
            Self::Broadcast => Self::BROADCAST_BYTE,
        }
    }

    /// Returns `true` when a message addressed to `self` should be accepted by a
    /// unit listening as `listener`.
    #[must_use]
    pub const fn addresses(self, listener: Self) -> bool {
        matches!(self, Self::Broadcast) || self.to_byte() == listener.to_byte()
    }
}

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unit(id) => write!(f, "unit {id}"),
            Self::Broadcast => f.write_str("broadcast"),
        }
    }
}

/// Synthesizer model.
///
/// Every variant shares model ID `0x20` and one protocol, so the wire cannot tell
/// them apart. They differ only in voice count and whether there is a keyboard.
/// The X series is a later reskin with the same architecture and specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Model {
    /// 6 voices, 37 keys.
    DeepMind6,
    /// 6 voices, 37 keys.
    DeepMind6X,
    /// 12 voices, 49 keys.
    #[default]
    DeepMind12,
    /// 12 voices, 49 keys.
    DeepMind12X,
    /// 12 voices, desktop.
    DeepMind12D,
    /// 12 voices, desktop.
    DeepMind12XD,
}

impl Model {
    /// Model ID byte shared by every variant.
    pub const MODEL_ID: u8 = 0x20;

    /// Every known variant.
    pub const ALL: [Self; 6] = [
        Self::DeepMind6,
        Self::DeepMind6X,
        Self::DeepMind12,
        Self::DeepMind12X,
        Self::DeepMind12D,
        Self::DeepMind12XD,
    ];

    /// Returns the number of analog voices.
    #[must_use]
    pub const fn voice_count(self) -> u8 {
        match self {
            Self::DeepMind6 | Self::DeepMind6X => 6,
            Self::DeepMind12 | Self::DeepMind12X | Self::DeepMind12D | Self::DeepMind12XD => 12,
        }
    }

    /// Returns `true` if the model has a keyboard.
    #[must_use]
    pub const fn has_keyboard(self) -> bool {
        !matches!(self, Self::DeepMind12D | Self::DeepMind12XD)
    }

    /// Returns the model name as Behringer writes it.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::DeepMind6 => "DeepMind 6",
            Self::DeepMind6X => "DeepMind 6X",
            Self::DeepMind12 => "DeepMind 12",
            Self::DeepMind12X => "DeepMind 12X",
            Self::DeepMind12D => "DeepMind 12D",
            Self::DeepMind12XD => "DeepMind 12XD",
        }
    }
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Comms protocol version carried by every dump response.
///
/// The firmware stamps this into each dump and the payload length depends on it:
/// version 6 carries 242 bytes of program data, version 7 carries 245. Parsers
/// must branch on it rather than assume a length.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProtocolVersion(pub u8);

impl ProtocolVersion {
    /// The version documented in the `DeepMind` 12 user manual.
    pub const V6: Self = Self(6);

    /// The version emitted by current firmware and by the factory preset packs.
    pub const V7: Self = Self(7);

    /// Returns the unpacked program data length for this version, if known.
    #[must_use]
    pub const fn program_data_len(self) -> Option<usize> {
        match self.0 {
            6 => Some(242),
            7 => Some(245),
            _ => None,
        }
    }
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}", self.0)
    }
}

/// One of the eight program banks, `A` through `H`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bank(u8);

impl Bank {
    /// Bank A.
    pub const A: Self = Self(0);
    /// Bank H, the last bank.
    pub const H: Self = Self(7);

    /// Builds a bank from its zero-based index.
    ///
    /// # Errors
    ///
    /// Returns [`Error::BankOutOfRange`] for an index of 8 or more.
    pub const fn new(index: u8) -> Result<Self, Error> {
        if index < BANK_COUNT {
            Ok(Self(index))
        } else {
            Err(Error::BankOutOfRange(index))
        }
    }

    /// Builds a bank from its letter, case-insensitively.
    ///
    /// # Errors
    ///
    /// Returns [`Error::BankOutOfRange`] for a letter outside `A..=H`.
    pub const fn from_letter(letter: char) -> Result<Self, Error> {
        match letter {
            'A'..='H' => Self::new(letter as u8 - b'A'),
            'a'..='h' => Self::new(letter as u8 - b'a'),
            _ => Err(Error::BankOutOfRange(u8::MAX)),
        }
    }

    /// Returns the zero-based index.
    #[must_use]
    pub const fn index(self) -> u8 {
        self.0
    }

    /// Returns the bank letter, `A` through `H`.
    #[must_use]
    pub const fn letter(self) -> char {
        (b'A' + self.0) as char
    }
}

impl fmt::Display for Bank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self.0 {
            0 => "A",
            1 => "B",
            2 => "C",
            3 => "D",
            4 => "E",
            5 => "F",
            6 => "G",
            _ => "H",
        })
    }
}

/// A program slot within a bank, 0 through 127.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProgramNumber(u8);

impl ProgramNumber {
    /// The first program in a bank.
    pub const FIRST: Self = Self(0);
    /// The last program in a bank.
    pub const LAST: Self = Self(127);

    /// Builds a program number.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ProgramOutOfRange`] for a number of 128 or more.
    pub const fn new(number: u8) -> Result<Self, Error> {
        if number < PROGRAMS_PER_BANK {
            Ok(Self(number))
        } else {
            Err(Error::ProgramOutOfRange(number))
        }
    }

    /// Returns the zero-based slot number.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl fmt::Display for ProgramNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The front panel numbers programs from 1.
        write!(f, "{}", u16::from(self.0) + 1)
    }
}

/// One of the 1024 places a program is stored: a bank and a number in it.
///
/// The two travel together everywhere a stored program does - a dump names both,
/// a `.syx` file says both, and the front panel shows them as one label.
///
/// ```
/// use deepmind_midi::ids::{Bank, ProgramNumber, Slot};
///
/// let slot = Slot::new(Bank::A, ProgramNumber::FIRST);
/// assert_eq!(slot.to_string(), "A1");
/// # Ok::<(), deepmind_midi::Error>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Slot {
    /// Bank the program is stored in.
    pub bank: Bank,
    /// Program within that bank.
    pub number: ProgramNumber,
}

impl Slot {
    /// The first program of the first bank.
    pub const FIRST: Self = Self {
        bank: Bank::A,
        number: ProgramNumber::FIRST,
    };

    /// Builds a slot.
    #[must_use]
    pub const fn new(bank: Bank, number: ProgramNumber) -> Self {
        Self { bank, number }
    }
}

impl fmt::Display for Slot {
    /// Writes the slot the way the front panel does: `A1` through `H128`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.bank, self.number)
    }
}

/// A user arpeggiator or sequencer pattern slot, 0 through 31.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PatternNumber(u8);

impl PatternNumber {
    /// The first user pattern.
    pub const FIRST: Self = Self(0);
    /// The last user pattern.
    pub const LAST: Self = Self(31);

    /// Builds a pattern number.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PatternOutOfRange`] for a number of 32 or more.
    pub const fn new(number: u8) -> Result<Self, Error> {
        if number < USER_PATTERN_COUNT {
            Ok(Self(number))
        } else {
            Err(Error::PatternOutOfRange(number))
        }
    }

    /// Returns the zero-based slot number.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl fmt::Display for PatternNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The front panel numbers patterns from 1.
        write!(f, "{}", u16::from(self.0) + 1)
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
    fn a_slot_reads_as_the_front_panel_writes_it() {
        assert_eq!(Slot::FIRST.to_string(), "A1");
        assert_eq!(Slot::new(Bank::H, ProgramNumber::LAST).to_string(), "H128");
    }

    #[test]
    fn device_id_round_trips_through_its_wire_byte() {
        for byte in (0..=15).chain(core::iter::once(0x7F)) {
            let id = DeviceId::from_byte(byte).expect("byte is in range");
            assert_eq!(id.to_byte(), byte);
        }
        assert_eq!(DeviceId::from_byte(16), Err(Error::InvalidDeviceId(16)));
        assert_eq!(DeviceId::from_byte(0x7E), Err(Error::InvalidDeviceId(0x7E)));
    }

    #[test]
    fn broadcast_addresses_every_unit_but_a_unit_only_addresses_itself() {
        let unit_3 = DeviceId::Unit(3);
        assert!(DeviceId::Broadcast.addresses(unit_3));
        assert!(unit_3.addresses(unit_3));
        assert!(!unit_3.addresses(DeviceId::Unit(4)));
    }

    #[test]
    fn every_model_reports_a_plausible_voice_count_and_name() {
        for model in Model::ALL {
            assert!(matches!(model.voice_count(), 6 | 12));
            assert!(model.name().starts_with("DeepMind "));
        }
        assert!(!Model::DeepMind12XD.has_keyboard());
        assert!(Model::DeepMind6X.has_keyboard());
    }

    #[test]
    fn banks_map_between_index_and_letter() {
        for index in 0..BANK_COUNT {
            let bank = Bank::new(index).expect("index is in range");
            assert_eq!(bank.letter(), (b'A' + index) as char);
            assert_eq!(Bank::from_letter(bank.letter()), Ok(bank));
            assert_eq!(
                Bank::from_letter(bank.letter().to_ascii_lowercase()),
                Ok(bank)
            );
        }
        assert!(Bank::new(BANK_COUNT).is_err());
        assert!(Bank::from_letter('I').is_err());
    }

    #[test]
    fn program_numbers_reject_out_of_range_slots() {
        assert_eq!(ProgramNumber::new(127).map(ProgramNumber::get), Ok(127));
        assert_eq!(ProgramNumber::new(128), Err(Error::ProgramOutOfRange(128)));
    }

    #[test]
    fn pattern_numbers_reject_out_of_range_slots() {
        assert_eq!(PatternNumber::new(31).map(PatternNumber::get), Ok(31));
        assert_eq!(PatternNumber::new(32), Err(Error::PatternOutOfRange(32)));
    }

    #[test]
    fn protocol_versions_carry_their_program_data_length() {
        assert_eq!(ProtocolVersion::V6.program_data_len(), Some(242));
        assert_eq!(ProtocolVersion::V7.program_data_len(), Some(245));
        assert_eq!(ProtocolVersion(99).program_data_len(), None);
    }
}
