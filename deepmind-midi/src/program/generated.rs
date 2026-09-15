//! The program layer's value types and typed accessors, generated from `spec/`.
//!
//! Do not edit. `cargo xtask codegen` rewrites this file, and `cargo test` fails
//! when it does not match the specification it came from. [`Program`] itself,
//! and everything that is not one parameter or one value table, is in the parent
//! module.

// A value table lands here as one match arm per value, and the largest has 133
// of them. Chunking those to satisfy a length lint would only hide what they
// are, and two values a firmware does not have are two arms answering `None`.
#![expect(
    clippy::too_many_lines,
    clippy::match_same_arms,
    reason = "generated tables, not logic"
)]

use core::fmt;

use super::Program;
use crate::param::{DEFAULT_FIRMWARE, ParamId, TableId};
use crate::sysex::inquiry::Version;

/// Offset of the first byte of the program's name.
pub const NAME_OFFSET: u8 = 223;

/// Bytes the name occupies. It holds sixteen characters and a terminator.
pub const NAME_LEN: usize = 17;

/// The parameters the program's name is stored in, in the order they print.
///
/// The instrument stores a name one character to a parameter, which is the
/// truth about the wire and a lie about what a person is editing. A host
/// drawing the parameter table needs to know which of its slots are the name,
/// so that seventeen faders become the one field the display shows; this is
/// that answer, and it moves with the field rather than being counted again.
///
/// Typing a letter still costs one message to the one parameter it moved:
/// [`Program::set_name`] writes the field and
/// [`Program::changes`](super::Program::changes) works out what that cost.
pub static NAME_PARAMETERS: [ParamId; NAME_LEN] = [
    ParamId::ProgramNameChar1,
    ParamId::ProgramNameChar2,
    ParamId::ProgramNameChar3,
    ParamId::ProgramNameChar4,
    ParamId::ProgramNameChar5,
    ParamId::ProgramNameChar6,
    ParamId::ProgramNameChar7,
    ParamId::ProgramNameChar8,
    ParamId::ProgramNameChar9,
    ParamId::ProgramNameChar10,
    ParamId::ProgramNameChar11,
    ParamId::ProgramNameChar12,
    ParamId::ProgramNameChar13,
    ParamId::ProgramNameChar14,
    ParamId::ProgramNameChar15,
    ParamId::ProgramNameChar16,
    ParamId::ProgramNameChar17,
];

/// Arpeggiator Mode.
///
/// Every value the synthesizer has a name for. A byte outside the table is not
/// one of these: [`ArpMode::from_raw`] answers `None` for it, and the byte stays
/// in the program either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum ArpMode {
    /// Up.
    Up,
    /// Down.
    Down,
    /// Up & Down.
    UpAndDown,
    /// Up Inv.
    UpInv,
    /// Down Inv.
    DownInv,
    /// Up & Down Inv.
    UpAndDownInv,
    /// Up Alt.
    UpAlt,
    /// Down Alt.
    DownAlt,
    /// Random.
    Random,
    /// As Played.
    AsPlayed,
    /// Chord.
    Chord,
}

impl ArpMode {
    /// The value table this type lists.
    pub const TABLE: TableId = TableId::ArpMode;

    /// Every value, in the order the newest firmware numbers them.
    pub const ALL: &'static [Self] = &[
        Self::Up,
        Self::Down,
        Self::UpAndDown,
        Self::UpInv,
        Self::DownInv,
        Self::UpAndDownInv,
        Self::UpAlt,
        Self::DownAlt,
        Self::Random,
        Self::AsPlayed,
        Self::Chord,
    ];

    /// Returns the value this byte selects on [`DEFAULT_FIRMWARE`].
    ///
    /// `None` when the table does not list the byte.
    #[must_use]
    pub const fn from_raw(raw: u8) -> Option<Self> {
        Self::from_raw_for(raw, DEFAULT_FIRMWARE)
    }

    /// Returns the value this byte selects on the firmware a device inquiry
    /// reported.
    ///
    /// Every firmware numbers this table the same way, so the version makes no
    /// difference here; it is taken so that every value type reads alike.
    #[must_use]
    pub const fn from_raw_for(raw: u8, _firmware: Version) -> Option<Self> {
        match raw {
            0 => Some(Self::Up),
            1 => Some(Self::Down),
            2 => Some(Self::UpAndDown),
            3 => Some(Self::UpInv),
            4 => Some(Self::DownInv),
            5 => Some(Self::UpAndDownInv),
            6 => Some(Self::UpAlt),
            7 => Some(Self::DownAlt),
            8 => Some(Self::Random),
            9 => Some(Self::AsPlayed),
            10 => Some(Self::Chord),
            _ => None,
        }
    }

    /// Returns the byte that selects this value on [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
            Self::Up => 0,
            Self::Down => 1,
            Self::UpAndDown => 2,
            Self::UpInv => 3,
            Self::DownInv => 4,
            Self::UpAndDownInv => 5,
            Self::UpAlt => 6,
            Self::DownAlt => 7,
            Self::Random => 8,
            Self::AsPlayed => 9,
            Self::Chord => 10,
        }
    }

    /// Returns the byte that selects this value on the firmware a device inquiry
    /// reported.
    ///
    /// Always `Some` for this table, which every firmware numbers the same way.
    #[must_use]
    pub const fn raw_for(self, _firmware: Version) -> Option<u8> {
        Some(self.raw())
    }

    /// Returns the name the synthesizer's display writes, as of
    /// [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub fn name(self) -> &'static str {
        self.name_for(DEFAULT_FIRMWARE).unwrap_or_default()
    }

    /// Returns the name the synthesizer's display writes on the firmware a
    /// device inquiry reported.
    ///
    /// `None` for a value that firmware does not have. The names live in the
    /// value table the parameter layer already carries; nothing here holds a
    /// second copy of them.
    #[must_use]
    pub fn name_for(self, firmware: Version) -> Option<&'static str> {
        let raw = self.raw_for(firmware)?;
        Self::TABLE.table_for(firmware).name_of(u16::from(raw))
    }
}

impl fmt::Display for ArpMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Envelope Trigger Source.
///
/// Every value the synthesizer has a name for. A byte outside the table is not
/// one of these: [`EnvelopeTrigger::from_raw`] answers `None` for it, and the byte stays
/// in the program either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum EnvelopeTrigger {
    /// Key.
    Key,
    /// LFO 1.
    Lfo1,
    /// LFO 2.
    Lfo2,
    /// Loop.
    Loop,
    /// Control Sequencer Step.
    ControlSequencerStep,
}

impl EnvelopeTrigger {
    /// The value table this type lists.
    pub const TABLE: TableId = TableId::EnvelopeTrigger;

    /// Every value, in the order the newest firmware numbers them.
    pub const ALL: &'static [Self] = &[
        Self::Key,
        Self::Lfo1,
        Self::Lfo2,
        Self::Loop,
        Self::ControlSequencerStep,
    ];

    /// Returns the value this byte selects on [`DEFAULT_FIRMWARE`].
    ///
    /// `None` when the table does not list the byte.
    #[must_use]
    pub const fn from_raw(raw: u8) -> Option<Self> {
        Self::from_raw_for(raw, DEFAULT_FIRMWARE)
    }

    /// Returns the value this byte selects on the firmware a device inquiry
    /// reported.
    ///
    /// Every firmware numbers this table the same way, so the version makes no
    /// difference here; it is taken so that every value type reads alike.
    #[must_use]
    pub const fn from_raw_for(raw: u8, _firmware: Version) -> Option<Self> {
        match raw {
            0 => Some(Self::Key),
            1 => Some(Self::Lfo1),
            2 => Some(Self::Lfo2),
            3 => Some(Self::Loop),
            4 => Some(Self::ControlSequencerStep),
            _ => None,
        }
    }

    /// Returns the byte that selects this value on [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
            Self::Key => 0,
            Self::Lfo1 => 1,
            Self::Lfo2 => 2,
            Self::Loop => 3,
            Self::ControlSequencerStep => 4,
        }
    }

    /// Returns the byte that selects this value on the firmware a device inquiry
    /// reported.
    ///
    /// Always `Some` for this table, which every firmware numbers the same way.
    #[must_use]
    pub const fn raw_for(self, _firmware: Version) -> Option<u8> {
        Some(self.raw())
    }

    /// Returns the name the synthesizer's display writes, as of
    /// [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub fn name(self) -> &'static str {
        self.name_for(DEFAULT_FIRMWARE).unwrap_or_default()
    }

    /// Returns the name the synthesizer's display writes on the firmware a
    /// device inquiry reported.
    ///
    /// `None` for a value that firmware does not have. The names live in the
    /// value table the parameter layer already carries; nothing here holds a
    /// second copy of them.
    #[must_use]
    pub fn name_for(self, firmware: Version) -> Option<&'static str> {
        let raw = self.raw_for(firmware)?;
        Self::TABLE.table_for(firmware).name_of(u16::from(raw))
    }
}

impl fmt::Display for EnvelopeTrigger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// FX Mode.
///
/// Every value the synthesizer has a name for. A byte outside the table is not
/// one of these: [`FxMode::from_raw`] answers `None` for it, and the byte stays
/// in the program either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum FxMode {
    /// Insert.
    Insert,
    /// Send.
    Send,
    /// Bypass.
    Bypass,
}

impl FxMode {
    /// The value table this type lists.
    pub const TABLE: TableId = TableId::FxMode;

    /// Every value, in the order the newest firmware numbers them.
    pub const ALL: &'static [Self] = &[Self::Insert, Self::Send, Self::Bypass];

    /// Returns the value this byte selects on [`DEFAULT_FIRMWARE`].
    ///
    /// `None` when the table does not list the byte.
    #[must_use]
    pub const fn from_raw(raw: u8) -> Option<Self> {
        Self::from_raw_for(raw, DEFAULT_FIRMWARE)
    }

    /// Returns the value this byte selects on the firmware a device inquiry
    /// reported.
    ///
    /// Every firmware numbers this table the same way, so the version makes no
    /// difference here; it is taken so that every value type reads alike.
    #[must_use]
    pub const fn from_raw_for(raw: u8, _firmware: Version) -> Option<Self> {
        match raw {
            0 => Some(Self::Insert),
            1 => Some(Self::Send),
            2 => Some(Self::Bypass),
            _ => None,
        }
    }

    /// Returns the byte that selects this value on [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
            Self::Insert => 0,
            Self::Send => 1,
            Self::Bypass => 2,
        }
    }

    /// Returns the byte that selects this value on the firmware a device inquiry
    /// reported.
    ///
    /// Always `Some` for this table, which every firmware numbers the same way.
    #[must_use]
    pub const fn raw_for(self, _firmware: Version) -> Option<u8> {
        Some(self.raw())
    }

    /// Returns the name the synthesizer's display writes, as of
    /// [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub fn name(self) -> &'static str {
        self.name_for(DEFAULT_FIRMWARE).unwrap_or_default()
    }

    /// Returns the name the synthesizer's display writes on the firmware a
    /// device inquiry reported.
    ///
    /// `None` for a value that firmware does not have. The names live in the
    /// value table the parameter layer already carries; nothing here holds a
    /// second copy of them.
    #[must_use]
    pub fn name_for(self, firmware: Version) -> Option<&'static str> {
        let raw = self.raw_for(firmware)?;
        Self::TABLE.table_for(firmware).name_of(u16::from(raw))
    }
}

impl fmt::Display for FxMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// FX Connection Mode.
///
/// Every value the synthesizer has a name for. A byte outside the table is not
/// one of these: [`FxRouting::from_raw`] answers `None` for it, and the byte stays
/// in the program either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum FxRouting {
    /// Serial 1-2-3-4.
    Serial1234,
    /// Parallel 1/2, serial 3-4.
    Parallel12Serial34,
    /// Parallel 1/2, parallel 3/4.
    Parallel12Parallel34,
    /// Parallel 1/2/3/4.
    Parallel1234,
    /// Parallel 1/2/3, serial 4.
    Parallel123Serial4,
    /// Serial 1-2, parallel 3/4.
    Serial12Parallel34,
    /// Serial 1, parallel 2/3/4.
    Serial1Parallel234,
    /// Parallel (serial 1-2-3)/4.
    ParallelSerial1234,
    /// Serial 3-4 feedback 4(1-2).
    Serial34Feedback412,
    /// Serial 4 feedback 4(1-2-3).
    Serial4Feedback4123,
}

impl FxRouting {
    /// The value table this type lists.
    pub const TABLE: TableId = TableId::FxRouting;

    /// Every value, in the order the newest firmware numbers them.
    pub const ALL: &'static [Self] = &[
        Self::Serial1234,
        Self::Parallel12Serial34,
        Self::Parallel12Parallel34,
        Self::Parallel1234,
        Self::Parallel123Serial4,
        Self::Serial12Parallel34,
        Self::Serial1Parallel234,
        Self::ParallelSerial1234,
        Self::Serial34Feedback412,
        Self::Serial4Feedback4123,
    ];

    /// Returns the value this byte selects on [`DEFAULT_FIRMWARE`].
    ///
    /// `None` when the table does not list the byte.
    #[must_use]
    pub const fn from_raw(raw: u8) -> Option<Self> {
        Self::from_raw_for(raw, DEFAULT_FIRMWARE)
    }

    /// Returns the value this byte selects on the firmware a device inquiry
    /// reported.
    ///
    /// Every firmware numbers this table the same way, so the version makes no
    /// difference here; it is taken so that every value type reads alike.
    #[must_use]
    pub const fn from_raw_for(raw: u8, _firmware: Version) -> Option<Self> {
        match raw {
            0 => Some(Self::Serial1234),
            1 => Some(Self::Parallel12Serial34),
            2 => Some(Self::Parallel12Parallel34),
            3 => Some(Self::Parallel1234),
            4 => Some(Self::Parallel123Serial4),
            5 => Some(Self::Serial12Parallel34),
            6 => Some(Self::Serial1Parallel234),
            7 => Some(Self::ParallelSerial1234),
            8 => Some(Self::Serial34Feedback412),
            9 => Some(Self::Serial4Feedback4123),
            _ => None,
        }
    }

    /// Returns the byte that selects this value on [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
            Self::Serial1234 => 0,
            Self::Parallel12Serial34 => 1,
            Self::Parallel12Parallel34 => 2,
            Self::Parallel1234 => 3,
            Self::Parallel123Serial4 => 4,
            Self::Serial12Parallel34 => 5,
            Self::Serial1Parallel234 => 6,
            Self::ParallelSerial1234 => 7,
            Self::Serial34Feedback412 => 8,
            Self::Serial4Feedback4123 => 9,
        }
    }

    /// Returns the byte that selects this value on the firmware a device inquiry
    /// reported.
    ///
    /// Always `Some` for this table, which every firmware numbers the same way.
    #[must_use]
    pub const fn raw_for(self, _firmware: Version) -> Option<u8> {
        Some(self.raw())
    }

    /// Returns the name the synthesizer's display writes, as of
    /// [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub fn name(self) -> &'static str {
        self.name_for(DEFAULT_FIRMWARE).unwrap_or_default()
    }

    /// Returns the name the synthesizer's display writes on the firmware a
    /// device inquiry reported.
    ///
    /// `None` for a value that firmware does not have. The names live in the
    /// value table the parameter layer already carries; nothing here holds a
    /// second copy of them.
    #[must_use]
    pub fn name_for(self, firmware: Version) -> Option<&'static str> {
        let raw = self.raw_for(firmware)?;
        Self::TABLE.table_for(firmware).name_of(u16::from(raw))
    }
}

impl fmt::Display for FxRouting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// FX Type.
///
/// Firmware renumbered this table rather than extending it, so a variant is what
/// the value means and not the byte it travels as. [`FxType::raw_for`] is that
/// byte on a given firmware, and [`FxType::raw`] is it on the newest.
///
/// Every value the synthesizer has a name for. A byte outside the table is not
/// one of these: [`FxType::from_raw`] answers `None` for it, and the byte stays
/// in the program either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum FxType {
    /// `TC-DeepVRB`.
    TcDeepVrb,
    /// `AmbVerb`.
    AmbVerb,
    /// `RoomRev`.
    RoomRev,
    /// `VintageRev`.
    VintageRev,
    /// `HallRev`.
    HallRev,
    /// `ChamberRev`.
    ChamberRev,
    /// `PlateRev`.
    PlateRev,
    /// `RichPltRev`.
    RichPltRev,
    /// `GatedRev`.
    GatedRev,
    /// Reverse.
    Reverse,
    /// `ChorusVerb`.
    ChorusVerb,
    /// `DelayVerb`.
    DelayVerb,
    /// `FlangVerb`.
    FlangVerb,
    /// `MidasEQ`.
    MidasEq,
    /// Enhancer.
    Enhancer,
    /// `FairComp`.
    FairComp,
    /// `MulBndDist`.
    MulBndDist,
    /// `RackAmp`.
    RackAmp,
    /// `EdisonEX1`.
    EdisonEx1,
    /// Auto Pan.
    AutoPan,
    /// `NoiseGate`.
    NoiseGate,
    /// Delay.
    Delay,
    /// `3TapDelay`.
    ThreeTapDelay,
    /// `4TapDelay`.
    FourTapDelay,
    /// `T-RayDelay`.
    TRayDelay,
    /// `DecimDelay`.
    DecimDelay,
    /// `ModDlyRev`.
    ModDlyRev,
    /// Chorus.
    Chorus,
    /// `Chorus-D`.
    ChorusD,
    /// Flanger.
    Flanger,
    /// Phaser.
    Phaser,
    /// `MoodFilter`.
    MoodFilter,
    /// `DualPitch`.
    DualPitch,
    /// Vintage Pitch.
    VintagePitch,
    /// `RotarySpkr`.
    RotarySpkr,
}

impl FxType {
    /// The value table this type lists.
    pub const TABLE: TableId = TableId::FxType;

    /// Every value, in the order the newest firmware numbers them.
    pub const ALL: &'static [Self] = &[
        Self::TcDeepVrb,
        Self::AmbVerb,
        Self::RoomRev,
        Self::VintageRev,
        Self::HallRev,
        Self::ChamberRev,
        Self::PlateRev,
        Self::RichPltRev,
        Self::GatedRev,
        Self::Reverse,
        Self::ChorusVerb,
        Self::DelayVerb,
        Self::FlangVerb,
        Self::MidasEq,
        Self::Enhancer,
        Self::FairComp,
        Self::MulBndDist,
        Self::RackAmp,
        Self::EdisonEx1,
        Self::AutoPan,
        Self::NoiseGate,
        Self::Delay,
        Self::ThreeTapDelay,
        Self::FourTapDelay,
        Self::TRayDelay,
        Self::DecimDelay,
        Self::ModDlyRev,
        Self::Chorus,
        Self::ChorusD,
        Self::Flanger,
        Self::Phaser,
        Self::MoodFilter,
        Self::DualPitch,
        Self::VintagePitch,
        Self::RotarySpkr,
    ];

    /// Returns the value this byte selects on [`DEFAULT_FIRMWARE`].
    ///
    /// `None` when the table does not list the byte.
    #[must_use]
    pub const fn from_raw(raw: u8) -> Option<Self> {
        Self::from_raw_for(raw, DEFAULT_FIRMWARE)
    }

    /// Returns the value this byte selects on the firmware a device inquiry
    /// reported.
    ///
    /// Worth reaching for whenever the byte came from a stored program: nothing
    /// in a dump says which firmware wrote it, and this table was renumbered.
    #[must_use]
    pub const fn from_raw_for(raw: u8, firmware: Version) -> Option<Self> {
        if firmware.at_least(1, 1) {
            match raw {
                0 => Some(Self::TcDeepVrb),
                1 => Some(Self::AmbVerb),
                2 => Some(Self::RoomRev),
                3 => Some(Self::VintageRev),
                4 => Some(Self::HallRev),
                5 => Some(Self::ChamberRev),
                6 => Some(Self::PlateRev),
                7 => Some(Self::RichPltRev),
                8 => Some(Self::GatedRev),
                9 => Some(Self::Reverse),
                10 => Some(Self::ChorusVerb),
                11 => Some(Self::DelayVerb),
                12 => Some(Self::FlangVerb),
                13 => Some(Self::MidasEq),
                14 => Some(Self::Enhancer),
                15 => Some(Self::FairComp),
                16 => Some(Self::MulBndDist),
                17 => Some(Self::RackAmp),
                18 => Some(Self::EdisonEx1),
                19 => Some(Self::AutoPan),
                20 => Some(Self::NoiseGate),
                21 => Some(Self::Delay),
                22 => Some(Self::ThreeTapDelay),
                23 => Some(Self::FourTapDelay),
                24 => Some(Self::TRayDelay),
                25 => Some(Self::DecimDelay),
                26 => Some(Self::ModDlyRev),
                27 => Some(Self::Chorus),
                28 => Some(Self::ChorusD),
                29 => Some(Self::Flanger),
                30 => Some(Self::Phaser),
                31 => Some(Self::MoodFilter),
                32 => Some(Self::DualPitch),
                33 => Some(Self::VintagePitch),
                34 => Some(Self::RotarySpkr),
                _ => None,
            }
        } else {
            match raw {
                0 => Some(Self::TcDeepVrb),
                1 => Some(Self::AmbVerb),
                2 => Some(Self::RoomRev),
                3 => Some(Self::VintageRev),
                4 => Some(Self::HallRev),
                5 => Some(Self::ChamberRev),
                6 => Some(Self::PlateRev),
                7 => Some(Self::RichPltRev),
                8 => Some(Self::GatedRev),
                9 => Some(Self::Reverse),
                10 => Some(Self::ChorusVerb),
                11 => Some(Self::DelayVerb),
                12 => Some(Self::FlangVerb),
                13 => Some(Self::MidasEq),
                14 => Some(Self::Enhancer),
                15 => Some(Self::FairComp),
                16 => Some(Self::MulBndDist),
                17 => Some(Self::RackAmp),
                18 => Some(Self::EdisonEx1),
                19 => Some(Self::AutoPan),
                20 => Some(Self::NoiseGate),
                21 => Some(Self::Delay),
                22 => Some(Self::ThreeTapDelay),
                23 => Some(Self::FourTapDelay),
                24 => Some(Self::TRayDelay),
                25 => Some(Self::DecimDelay),
                26 => Some(Self::ModDlyRev),
                27 => Some(Self::Chorus),
                28 => Some(Self::ChorusD),
                29 => Some(Self::Flanger),
                30 => Some(Self::Phaser),
                31 => Some(Self::MoodFilter),
                32 => Some(Self::DualPitch),
                33 => Some(Self::RotarySpkr),
                _ => None,
            }
        }
    }

    /// Returns the byte that selects this value on [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
            Self::TcDeepVrb => 0,
            Self::AmbVerb => 1,
            Self::RoomRev => 2,
            Self::VintageRev => 3,
            Self::HallRev => 4,
            Self::ChamberRev => 5,
            Self::PlateRev => 6,
            Self::RichPltRev => 7,
            Self::GatedRev => 8,
            Self::Reverse => 9,
            Self::ChorusVerb => 10,
            Self::DelayVerb => 11,
            Self::FlangVerb => 12,
            Self::MidasEq => 13,
            Self::Enhancer => 14,
            Self::FairComp => 15,
            Self::MulBndDist => 16,
            Self::RackAmp => 17,
            Self::EdisonEx1 => 18,
            Self::AutoPan => 19,
            Self::NoiseGate => 20,
            Self::Delay => 21,
            Self::ThreeTapDelay => 22,
            Self::FourTapDelay => 23,
            Self::TRayDelay => 24,
            Self::DecimDelay => 25,
            Self::ModDlyRev => 26,
            Self::Chorus => 27,
            Self::ChorusD => 28,
            Self::Flanger => 29,
            Self::Phaser => 30,
            Self::MoodFilter => 31,
            Self::DualPitch => 32,
            Self::VintagePitch => 33,
            Self::RotarySpkr => 34,
        }
    }

    /// Returns the byte that selects this value on the firmware a device inquiry
    /// reported.
    ///
    /// `None` for a value that firmware does not have.
    #[must_use]
    pub const fn raw_for(self, firmware: Version) -> Option<u8> {
        if firmware.at_least(1, 1) {
            match self {
                Self::TcDeepVrb => Some(0),
                Self::AmbVerb => Some(1),
                Self::RoomRev => Some(2),
                Self::VintageRev => Some(3),
                Self::HallRev => Some(4),
                Self::ChamberRev => Some(5),
                Self::PlateRev => Some(6),
                Self::RichPltRev => Some(7),
                Self::GatedRev => Some(8),
                Self::Reverse => Some(9),
                Self::ChorusVerb => Some(10),
                Self::DelayVerb => Some(11),
                Self::FlangVerb => Some(12),
                Self::MidasEq => Some(13),
                Self::Enhancer => Some(14),
                Self::FairComp => Some(15),
                Self::MulBndDist => Some(16),
                Self::RackAmp => Some(17),
                Self::EdisonEx1 => Some(18),
                Self::AutoPan => Some(19),
                Self::NoiseGate => Some(20),
                Self::Delay => Some(21),
                Self::ThreeTapDelay => Some(22),
                Self::FourTapDelay => Some(23),
                Self::TRayDelay => Some(24),
                Self::DecimDelay => Some(25),
                Self::ModDlyRev => Some(26),
                Self::Chorus => Some(27),
                Self::ChorusD => Some(28),
                Self::Flanger => Some(29),
                Self::Phaser => Some(30),
                Self::MoodFilter => Some(31),
                Self::DualPitch => Some(32),
                Self::VintagePitch => Some(33),
                Self::RotarySpkr => Some(34),
            }
        } else {
            match self {
                Self::TcDeepVrb => Some(0),
                Self::AmbVerb => Some(1),
                Self::RoomRev => Some(2),
                Self::VintageRev => Some(3),
                Self::HallRev => Some(4),
                Self::ChamberRev => Some(5),
                Self::PlateRev => Some(6),
                Self::RichPltRev => Some(7),
                Self::GatedRev => Some(8),
                Self::Reverse => Some(9),
                Self::ChorusVerb => Some(10),
                Self::DelayVerb => Some(11),
                Self::FlangVerb => Some(12),
                Self::MidasEq => Some(13),
                Self::Enhancer => Some(14),
                Self::FairComp => Some(15),
                Self::MulBndDist => Some(16),
                Self::RackAmp => Some(17),
                Self::EdisonEx1 => Some(18),
                Self::AutoPan => Some(19),
                Self::NoiseGate => Some(20),
                Self::Delay => Some(21),
                Self::ThreeTapDelay => Some(22),
                Self::FourTapDelay => Some(23),
                Self::TRayDelay => Some(24),
                Self::DecimDelay => Some(25),
                Self::ModDlyRev => Some(26),
                Self::Chorus => Some(27),
                Self::ChorusD => Some(28),
                Self::Flanger => Some(29),
                Self::Phaser => Some(30),
                Self::MoodFilter => Some(31),
                Self::DualPitch => Some(32),
                Self::VintagePitch => None,
                Self::RotarySpkr => Some(33),
            }
        }
    }

    /// Returns the name the synthesizer's display writes, as of
    /// [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub fn name(self) -> &'static str {
        self.name_for(DEFAULT_FIRMWARE).unwrap_or_default()
    }

    /// Returns the name the synthesizer's display writes on the firmware a
    /// device inquiry reported.
    ///
    /// `None` for a value that firmware does not have. The names live in the
    /// value table the parameter layer already carries; nothing here holds a
    /// second copy of them.
    #[must_use]
    pub fn name_for(self, firmware: Version) -> Option<&'static str> {
        let raw = self.raw_for(firmware)?;
        Self::TABLE.table_for(firmware).name_of(u16::from(raw))
    }
}

impl fmt::Display for FxType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Envelope Trigger Mode.
///
/// Every value the synthesizer has a name for. A byte outside the table is not
/// one of these: [`KeyAssignMode::from_raw`] answers `None` for it, and the byte stays
/// in the program either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum KeyAssignMode {
    /// Mono.
    Mono,
    /// `Re-Trigger`.
    ReTrigger,
    /// Legato.
    Legato,
    /// `One-Shot`.
    OneShot,
}

impl KeyAssignMode {
    /// The value table this type lists.
    pub const TABLE: TableId = TableId::KeyAssignMode;

    /// Every value, in the order the newest firmware numbers them.
    pub const ALL: &'static [Self] = &[Self::Mono, Self::ReTrigger, Self::Legato, Self::OneShot];

    /// Returns the value this byte selects on [`DEFAULT_FIRMWARE`].
    ///
    /// `None` when the table does not list the byte.
    #[must_use]
    pub const fn from_raw(raw: u8) -> Option<Self> {
        Self::from_raw_for(raw, DEFAULT_FIRMWARE)
    }

    /// Returns the value this byte selects on the firmware a device inquiry
    /// reported.
    ///
    /// Every firmware numbers this table the same way, so the version makes no
    /// difference here; it is taken so that every value type reads alike.
    #[must_use]
    pub const fn from_raw_for(raw: u8, _firmware: Version) -> Option<Self> {
        match raw {
            0 => Some(Self::Mono),
            1 => Some(Self::ReTrigger),
            2 => Some(Self::Legato),
            3 => Some(Self::OneShot),
            _ => None,
        }
    }

    /// Returns the byte that selects this value on [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
            Self::Mono => 0,
            Self::ReTrigger => 1,
            Self::Legato => 2,
            Self::OneShot => 3,
        }
    }

    /// Returns the byte that selects this value on the firmware a device inquiry
    /// reported.
    ///
    /// Always `Some` for this table, which every firmware numbers the same way.
    #[must_use]
    pub const fn raw_for(self, _firmware: Version) -> Option<u8> {
        Some(self.raw())
    }

    /// Returns the name the synthesizer's display writes, as of
    /// [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub fn name(self) -> &'static str {
        self.name_for(DEFAULT_FIRMWARE).unwrap_or_default()
    }

    /// Returns the name the synthesizer's display writes on the firmware a
    /// device inquiry reported.
    ///
    /// `None` for a value that firmware does not have. The names live in the
    /// value table the parameter layer already carries; nothing here holds a
    /// second copy of them.
    #[must_use]
    pub fn name_for(self, firmware: Version) -> Option<&'static str> {
        let raw = self.raw_for(firmware)?;
        Self::TABLE.table_for(firmware).name_of(u16::from(raw))
    }
}

impl fmt::Display for KeyAssignMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// LFO Shape.
///
/// Every value the synthesizer has a name for. A byte outside the table is not
/// one of these: [`LfoShape::from_raw`] answers `None` for it, and the byte stays
/// in the program either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum LfoShape {
    /// Sine.
    Sine,
    /// Triangle.
    Triangle,
    /// Square.
    Square,
    /// Ramp Up.
    RampUp,
    /// Ramp Down.
    RampDown,
    /// Sample & Hold.
    SampleAndHold,
    /// Sample & Glide.
    SampleAndGlide,
}

impl LfoShape {
    /// The value table this type lists.
    pub const TABLE: TableId = TableId::LfoShape;

    /// Every value, in the order the newest firmware numbers them.
    pub const ALL: &'static [Self] = &[
        Self::Sine,
        Self::Triangle,
        Self::Square,
        Self::RampUp,
        Self::RampDown,
        Self::SampleAndHold,
        Self::SampleAndGlide,
    ];

    /// Returns the value this byte selects on [`DEFAULT_FIRMWARE`].
    ///
    /// `None` when the table does not list the byte.
    #[must_use]
    pub const fn from_raw(raw: u8) -> Option<Self> {
        Self::from_raw_for(raw, DEFAULT_FIRMWARE)
    }

    /// Returns the value this byte selects on the firmware a device inquiry
    /// reported.
    ///
    /// Every firmware numbers this table the same way, so the version makes no
    /// difference here; it is taken so that every value type reads alike.
    #[must_use]
    pub const fn from_raw_for(raw: u8, _firmware: Version) -> Option<Self> {
        match raw {
            0 => Some(Self::Sine),
            1 => Some(Self::Triangle),
            2 => Some(Self::Square),
            3 => Some(Self::RampUp),
            4 => Some(Self::RampDown),
            5 => Some(Self::SampleAndHold),
            6 => Some(Self::SampleAndGlide),
            _ => None,
        }
    }

    /// Returns the byte that selects this value on [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
            Self::Sine => 0,
            Self::Triangle => 1,
            Self::Square => 2,
            Self::RampUp => 3,
            Self::RampDown => 4,
            Self::SampleAndHold => 5,
            Self::SampleAndGlide => 6,
        }
    }

    /// Returns the byte that selects this value on the firmware a device inquiry
    /// reported.
    ///
    /// Always `Some` for this table, which every firmware numbers the same way.
    #[must_use]
    pub const fn raw_for(self, _firmware: Version) -> Option<u8> {
        Some(self.raw())
    }

    /// Returns the name the synthesizer's display writes, as of
    /// [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub fn name(self) -> &'static str {
        self.name_for(DEFAULT_FIRMWARE).unwrap_or_default()
    }

    /// Returns the name the synthesizer's display writes on the firmware a
    /// device inquiry reported.
    ///
    /// `None` for a value that firmware does not have. The names live in the
    /// value table the parameter layer already carries; nothing here holds a
    /// second copy of them.
    #[must_use]
    pub fn name_for(self, firmware: Version) -> Option<&'static str> {
        let raw = self.raw_for(firmware)?;
        Self::TABLE.table_for(firmware).name_of(u16::from(raw))
    }
}

impl fmt::Display for LfoShape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Modulation Matrix Destination.
///
/// Firmware renumbered this table rather than extending it, so a variant is what
/// the value means and not the byte it travels as. [`ModDestination::raw_for`] is that
/// byte on a given firmware, and [`ModDestination::raw`] is it on the newest.
///
/// Every value the synthesizer has a name for. A byte outside the table is not
/// one of these: [`ModDestination::from_raw`] answers `None` for it, and the byte stays
/// in the program either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum ModDestination {
    /// Off.
    Off,
    /// LFO1 Rate.
    Lfo1Rate,
    /// LFO1 Delay.
    Lfo1Delay,
    /// LFO1 Slew.
    Lfo1Slew,
    /// LFO1 Shape.
    Lfo1Shape,
    /// LFO2 Rate.
    Lfo2Rate,
    /// LFO2 Delay.
    Lfo2Delay,
    /// LFO2 Slew.
    Lfo2Slew,
    /// LFO2 Shape.
    Lfo2Shape,
    /// OSC1+2 Pit.
    Osc12Pit,
    /// OSC1+2 Fine.
    Osc12Fine,
    /// OSC1 Pitch.
    Osc1Pitch,
    /// OSC1 Fine.
    Osc1Fine,
    /// OSC2 Pitch.
    Osc2Pitch,
    /// OSC2 Fine.
    Osc2Fine,
    /// OSC1 PM Dep.
    Osc1PmDep,
    /// PWM Depth.
    PwmDepth,
    /// `TMod` Depth.
    TmodDepth,
    /// OSC2 PM Dep.
    Osc2PmDep,
    /// Porta Time.
    PortaTime,
    /// VCF Freq.
    VcfFreq,
    /// VCF Res.
    VcfRes,
    /// VCF Env.
    VcfEnv,
    /// VCF LFO.
    VcfLfo,
    /// Env Rates.
    EnvRates,
    /// All Attack.
    AllAttack,
    /// All Decay.
    AllDecay,
    /// All Sus.
    AllSus,
    /// All Rel.
    AllRel,
    /// Env1 Rates.
    Env1Rates,
    /// Env2 Rates.
    Env2Rates,
    /// Env3 Rates.
    Env3Rates,
    /// `Env1CurveS`.
    Env1CurveS,
    /// `Env2CurveS`.
    Env2CurveS,
    /// `Env3CurveS`.
    Env3CurveS,
    /// Env1 Attack.
    Env1Attack,
    /// Env1 Decay.
    Env1Decay,
    /// Env1 Sus.
    Env1Sus,
    /// Env1 Rel.
    Env1Rel,
    /// Env1 `AtCur`.
    Env1AtCur,
    /// Env1 `DcyCur`.
    Env1DcyCur,
    /// Env1 `SuSCur`.
    Env1SuScur,
    /// Env1 `RelCur`.
    Env1RelCur,
    /// Env2 Attack.
    Env2Attack,
    /// Env2 Decay.
    Env2Decay,
    /// Env2 Sus.
    Env2Sus,
    /// Env2 Rel.
    Env2Rel,
    /// Env2 `AtCur`.
    Env2AtCur,
    /// Env2 `DcyCur`.
    Env2DcyCur,
    /// Env2 `SuSCur`.
    Env2SuScur,
    /// Env2 `RelCur`.
    Env2RelCur,
    /// Env3 Attack.
    Env3Attack,
    /// Env3 Decay.
    Env3Decay,
    /// Env3 Sus.
    Env3Sus,
    /// Env3 Rel.
    Env3Rel,
    /// Env3 `AtCur`.
    Env3AtCur,
    /// Env3 `DcyCur`.
    Env3DcyCur,
    /// Env3 `SuSCur`.
    Env3SuScur,
    /// Env3 `RelCur`.
    Env3RelCur,
    /// VCA All.
    VcaAll,
    /// VCA Active.
    VcaActive,
    /// VCA `EnvDep`.
    VcaEnvDep,
    /// Pan Spread.
    PanSpread,
    /// VCA Pan.
    VcaPan,
    /// OSC2 Lvl.
    Osc2Lvl,
    /// Noise Lvl.
    NoiseLvl,
    /// HP Freq.
    HpFreq,
    /// Uni Detune.
    UniDetune,
    /// OSC Drift.
    OscDrift,
    /// Param Drift.
    ParamDrift,
    /// Drift Rate.
    DriftRate,
    /// Arp Gate.
    ArpGate,
    /// Seq Slew.
    SeqSlew,
    /// Mod 1 Dep.
    Mod1Dep,
    /// Mod 2 Dep.
    Mod2Dep,
    /// Mod 3 Dep.
    Mod3Dep,
    /// Mod 4 Dep.
    Mod4Dep,
    /// Mod 5 Dep.
    Mod5Dep,
    /// Mod 6 Dep.
    Mod6Dep,
    /// Mod 7 Dep.
    Mod7Dep,
    /// Mod 8 Dep.
    Mod8Dep,
    /// Fx 1 Param 1.
    Fx1Param1,
    /// Fx 1 Param 2.
    Fx1Param2,
    /// Fx 1 Param 3.
    Fx1Param3,
    /// Fx 1 Param 4.
    Fx1Param4,
    /// Fx 1 Param 5.
    Fx1Param5,
    /// Fx 1 Param 6.
    Fx1Param6,
    /// Fx 1 Param 7.
    Fx1Param7,
    /// Fx 1 Param 8.
    Fx1Param8,
    /// Fx 1 Param 9.
    Fx1Param9,
    /// Fx 1 Param 10.
    Fx1Param10,
    /// Fx 1 Param 11.
    Fx1Param11,
    /// Fx 1 Param 12.
    Fx1Param12,
    /// Fx 2 Param 1.
    Fx2Param1,
    /// Fx 2 Param 2.
    Fx2Param2,
    /// Fx 2 Param 3.
    Fx2Param3,
    /// Fx 2 Param 4.
    Fx2Param4,
    /// Fx 2 Param 5.
    Fx2Param5,
    /// Fx 2 Param 6.
    Fx2Param6,
    /// Fx 2 Param 7.
    Fx2Param7,
    /// Fx 2 Param 8.
    Fx2Param8,
    /// Fx 2 Param 9.
    Fx2Param9,
    /// Fx 2 Param 10.
    Fx2Param10,
    /// Fx 2 Param 11.
    Fx2Param11,
    /// Fx 2 Param 12.
    Fx2Param12,
    /// Fx 3 Param 1.
    Fx3Param1,
    /// Fx 3 Param 2.
    Fx3Param2,
    /// Fx 3 Param 3.
    Fx3Param3,
    /// Fx 3 Param 4.
    Fx3Param4,
    /// Fx 3 Param 5.
    Fx3Param5,
    /// Fx 3 Param 6.
    Fx3Param6,
    /// Fx 3 Param 7.
    Fx3Param7,
    /// Fx 3 Param 8.
    Fx3Param8,
    /// Fx 3 Param 9.
    Fx3Param9,
    /// Fx 3 Param 10.
    Fx3Param10,
    /// Fx 3 Param 11.
    Fx3Param11,
    /// Fx 3 Param 12.
    Fx3Param12,
    /// Fx 4 Param 1.
    Fx4Param1,
    /// Fx 4 Param 2.
    Fx4Param2,
    /// Fx 4 Param 3.
    Fx4Param3,
    /// Fx 4 Param 4.
    Fx4Param4,
    /// Fx 4 Param 5.
    Fx4Param5,
    /// Fx 4 Param 6.
    Fx4Param6,
    /// Fx 4 Param 7.
    Fx4Param7,
    /// Fx 4 Param 8.
    Fx4Param8,
    /// Fx 4 Param 9.
    Fx4Param9,
    /// Fx 4 Param 10.
    Fx4Param10,
    /// Fx 4 Param 11.
    Fx4Param11,
    /// Fx 4 Param 12.
    Fx4Param12,
    /// Fx 1 Level.
    Fx1Level,
    /// Fx 2 Level.
    Fx2Level,
    /// Fx 3 Level.
    Fx3Level,
    /// Fx 4 Level.
    Fx4Level,
}

impl ModDestination {
    /// The value table this type lists.
    pub const TABLE: TableId = TableId::ModDestination;

    /// Every value, in the order the newest firmware numbers them.
    pub const ALL: &'static [Self] = &[
        Self::Off,
        Self::Lfo1Rate,
        Self::Lfo1Delay,
        Self::Lfo1Slew,
        Self::Lfo1Shape,
        Self::Lfo2Rate,
        Self::Lfo2Delay,
        Self::Lfo2Slew,
        Self::Lfo2Shape,
        Self::Osc12Pit,
        Self::Osc12Fine,
        Self::Osc1Pitch,
        Self::Osc1Fine,
        Self::Osc2Pitch,
        Self::Osc2Fine,
        Self::Osc1PmDep,
        Self::PwmDepth,
        Self::TmodDepth,
        Self::Osc2PmDep,
        Self::PortaTime,
        Self::VcfFreq,
        Self::VcfRes,
        Self::VcfEnv,
        Self::VcfLfo,
        Self::EnvRates,
        Self::AllAttack,
        Self::AllDecay,
        Self::AllSus,
        Self::AllRel,
        Self::Env1Rates,
        Self::Env2Rates,
        Self::Env3Rates,
        Self::Env1CurveS,
        Self::Env2CurveS,
        Self::Env3CurveS,
        Self::Env1Attack,
        Self::Env1Decay,
        Self::Env1Sus,
        Self::Env1Rel,
        Self::Env1AtCur,
        Self::Env1DcyCur,
        Self::Env1SuScur,
        Self::Env1RelCur,
        Self::Env2Attack,
        Self::Env2Decay,
        Self::Env2Sus,
        Self::Env2Rel,
        Self::Env2AtCur,
        Self::Env2DcyCur,
        Self::Env2SuScur,
        Self::Env2RelCur,
        Self::Env3Attack,
        Self::Env3Decay,
        Self::Env3Sus,
        Self::Env3Rel,
        Self::Env3AtCur,
        Self::Env3DcyCur,
        Self::Env3SuScur,
        Self::Env3RelCur,
        Self::VcaAll,
        Self::VcaActive,
        Self::VcaEnvDep,
        Self::PanSpread,
        Self::VcaPan,
        Self::Osc2Lvl,
        Self::NoiseLvl,
        Self::HpFreq,
        Self::UniDetune,
        Self::OscDrift,
        Self::ParamDrift,
        Self::DriftRate,
        Self::ArpGate,
        Self::SeqSlew,
        Self::Mod1Dep,
        Self::Mod2Dep,
        Self::Mod3Dep,
        Self::Mod4Dep,
        Self::Mod5Dep,
        Self::Mod6Dep,
        Self::Mod7Dep,
        Self::Mod8Dep,
        Self::Fx1Param1,
        Self::Fx1Param2,
        Self::Fx1Param3,
        Self::Fx1Param4,
        Self::Fx1Param5,
        Self::Fx1Param6,
        Self::Fx1Param7,
        Self::Fx1Param8,
        Self::Fx1Param9,
        Self::Fx1Param10,
        Self::Fx1Param11,
        Self::Fx1Param12,
        Self::Fx2Param1,
        Self::Fx2Param2,
        Self::Fx2Param3,
        Self::Fx2Param4,
        Self::Fx2Param5,
        Self::Fx2Param6,
        Self::Fx2Param7,
        Self::Fx2Param8,
        Self::Fx2Param9,
        Self::Fx2Param10,
        Self::Fx2Param11,
        Self::Fx2Param12,
        Self::Fx3Param1,
        Self::Fx3Param2,
        Self::Fx3Param3,
        Self::Fx3Param4,
        Self::Fx3Param5,
        Self::Fx3Param6,
        Self::Fx3Param7,
        Self::Fx3Param8,
        Self::Fx3Param9,
        Self::Fx3Param10,
        Self::Fx3Param11,
        Self::Fx3Param12,
        Self::Fx4Param1,
        Self::Fx4Param2,
        Self::Fx4Param3,
        Self::Fx4Param4,
        Self::Fx4Param5,
        Self::Fx4Param6,
        Self::Fx4Param7,
        Self::Fx4Param8,
        Self::Fx4Param9,
        Self::Fx4Param10,
        Self::Fx4Param11,
        Self::Fx4Param12,
        Self::Fx1Level,
        Self::Fx2Level,
        Self::Fx3Level,
        Self::Fx4Level,
    ];

    /// Returns the value this byte selects on [`DEFAULT_FIRMWARE`].
    ///
    /// `None` when the table does not list the byte.
    #[must_use]
    pub const fn from_raw(raw: u8) -> Option<Self> {
        Self::from_raw_for(raw, DEFAULT_FIRMWARE)
    }

    /// Returns the value this byte selects on the firmware a device inquiry
    /// reported.
    ///
    /// Worth reaching for whenever the byte came from a stored program: nothing
    /// in a dump says which firmware wrote it, and this table was renumbered.
    #[must_use]
    pub const fn from_raw_for(raw: u8, firmware: Version) -> Option<Self> {
        if firmware.at_least(1, 1) {
            match raw {
                0 => Some(Self::Off),
                1 => Some(Self::Lfo1Rate),
                2 => Some(Self::Lfo1Delay),
                3 => Some(Self::Lfo1Slew),
                4 => Some(Self::Lfo1Shape),
                5 => Some(Self::Lfo2Rate),
                6 => Some(Self::Lfo2Delay),
                7 => Some(Self::Lfo2Slew),
                8 => Some(Self::Lfo2Shape),
                9 => Some(Self::Osc12Pit),
                10 => Some(Self::Osc12Fine),
                11 => Some(Self::Osc1Pitch),
                12 => Some(Self::Osc1Fine),
                13 => Some(Self::Osc2Pitch),
                14 => Some(Self::Osc2Fine),
                15 => Some(Self::Osc1PmDep),
                16 => Some(Self::PwmDepth),
                17 => Some(Self::TmodDepth),
                18 => Some(Self::Osc2PmDep),
                19 => Some(Self::PortaTime),
                20 => Some(Self::VcfFreq),
                21 => Some(Self::VcfRes),
                22 => Some(Self::VcfEnv),
                23 => Some(Self::VcfLfo),
                24 => Some(Self::EnvRates),
                25 => Some(Self::AllAttack),
                26 => Some(Self::AllDecay),
                27 => Some(Self::AllSus),
                28 => Some(Self::AllRel),
                29 => Some(Self::Env1Rates),
                30 => Some(Self::Env2Rates),
                31 => Some(Self::Env3Rates),
                32 => Some(Self::Env1CurveS),
                33 => Some(Self::Env2CurveS),
                34 => Some(Self::Env3CurveS),
                35 => Some(Self::Env1Attack),
                36 => Some(Self::Env1Decay),
                37 => Some(Self::Env1Sus),
                38 => Some(Self::Env1Rel),
                39 => Some(Self::Env1AtCur),
                40 => Some(Self::Env1DcyCur),
                41 => Some(Self::Env1SuScur),
                42 => Some(Self::Env1RelCur),
                43 => Some(Self::Env2Attack),
                44 => Some(Self::Env2Decay),
                45 => Some(Self::Env2Sus),
                46 => Some(Self::Env2Rel),
                47 => Some(Self::Env2AtCur),
                48 => Some(Self::Env2DcyCur),
                49 => Some(Self::Env2SuScur),
                50 => Some(Self::Env2RelCur),
                51 => Some(Self::Env3Attack),
                52 => Some(Self::Env3Decay),
                53 => Some(Self::Env3Sus),
                54 => Some(Self::Env3Rel),
                55 => Some(Self::Env3AtCur),
                56 => Some(Self::Env3DcyCur),
                57 => Some(Self::Env3SuScur),
                58 => Some(Self::Env3RelCur),
                59 => Some(Self::VcaAll),
                60 => Some(Self::VcaActive),
                61 => Some(Self::VcaEnvDep),
                62 => Some(Self::PanSpread),
                63 => Some(Self::VcaPan),
                64 => Some(Self::Osc2Lvl),
                65 => Some(Self::NoiseLvl),
                66 => Some(Self::HpFreq),
                67 => Some(Self::UniDetune),
                68 => Some(Self::OscDrift),
                69 => Some(Self::ParamDrift),
                70 => Some(Self::DriftRate),
                71 => Some(Self::ArpGate),
                72 => Some(Self::SeqSlew),
                73 => Some(Self::Mod1Dep),
                74 => Some(Self::Mod2Dep),
                75 => Some(Self::Mod3Dep),
                76 => Some(Self::Mod4Dep),
                77 => Some(Self::Mod5Dep),
                78 => Some(Self::Mod6Dep),
                79 => Some(Self::Mod7Dep),
                80 => Some(Self::Mod8Dep),
                81 => Some(Self::Fx1Param1),
                82 => Some(Self::Fx1Param2),
                83 => Some(Self::Fx1Param3),
                84 => Some(Self::Fx1Param4),
                85 => Some(Self::Fx1Param5),
                86 => Some(Self::Fx1Param6),
                87 => Some(Self::Fx1Param7),
                88 => Some(Self::Fx1Param8),
                89 => Some(Self::Fx1Param9),
                90 => Some(Self::Fx1Param10),
                91 => Some(Self::Fx1Param11),
                92 => Some(Self::Fx1Param12),
                93 => Some(Self::Fx2Param1),
                94 => Some(Self::Fx2Param2),
                95 => Some(Self::Fx2Param3),
                96 => Some(Self::Fx2Param4),
                97 => Some(Self::Fx2Param5),
                98 => Some(Self::Fx2Param6),
                99 => Some(Self::Fx2Param7),
                100 => Some(Self::Fx2Param8),
                101 => Some(Self::Fx2Param9),
                102 => Some(Self::Fx2Param10),
                103 => Some(Self::Fx2Param11),
                104 => Some(Self::Fx2Param12),
                105 => Some(Self::Fx3Param1),
                106 => Some(Self::Fx3Param2),
                107 => Some(Self::Fx3Param3),
                108 => Some(Self::Fx3Param4),
                109 => Some(Self::Fx3Param5),
                110 => Some(Self::Fx3Param6),
                111 => Some(Self::Fx3Param7),
                112 => Some(Self::Fx3Param8),
                113 => Some(Self::Fx3Param9),
                114 => Some(Self::Fx3Param10),
                115 => Some(Self::Fx3Param11),
                116 => Some(Self::Fx3Param12),
                117 => Some(Self::Fx4Param1),
                118 => Some(Self::Fx4Param2),
                119 => Some(Self::Fx4Param3),
                120 => Some(Self::Fx4Param4),
                121 => Some(Self::Fx4Param5),
                122 => Some(Self::Fx4Param6),
                123 => Some(Self::Fx4Param7),
                124 => Some(Self::Fx4Param8),
                125 => Some(Self::Fx4Param9),
                126 => Some(Self::Fx4Param10),
                127 => Some(Self::Fx4Param11),
                128 => Some(Self::Fx4Param12),
                129 => Some(Self::Fx1Level),
                130 => Some(Self::Fx2Level),
                131 => Some(Self::Fx3Level),
                132 => Some(Self::Fx4Level),
                _ => None,
            }
        } else {
            match raw {
                0 => Some(Self::Off),
                1 => Some(Self::Lfo1Rate),
                2 => Some(Self::Lfo1Delay),
                3 => Some(Self::Lfo1Slew),
                4 => Some(Self::Lfo1Shape),
                5 => Some(Self::Lfo2Rate),
                6 => Some(Self::Lfo2Delay),
                7 => Some(Self::Lfo2Slew),
                8 => Some(Self::Lfo2Shape),
                9 => Some(Self::Osc12Pit),
                10 => Some(Self::Osc1Pitch),
                11 => Some(Self::Osc2Pitch),
                12 => Some(Self::Osc1PmDep),
                13 => Some(Self::PwmDepth),
                14 => Some(Self::TmodDepth),
                15 => Some(Self::Osc2PmDep),
                16 => Some(Self::PortaTime),
                17 => Some(Self::VcfFreq),
                18 => Some(Self::VcfRes),
                19 => Some(Self::VcfEnv),
                20 => Some(Self::VcfLfo),
                21 => Some(Self::EnvRates),
                22 => Some(Self::AllAttack),
                23 => Some(Self::AllDecay),
                24 => Some(Self::AllSus),
                25 => Some(Self::AllRel),
                26 => Some(Self::Env1Rates),
                27 => Some(Self::Env2Rates),
                28 => Some(Self::Env3Rates),
                29 => Some(Self::Env1CurveS),
                30 => Some(Self::Env2CurveS),
                31 => Some(Self::Env3CurveS),
                32 => Some(Self::Env1Attack),
                33 => Some(Self::Env1Decay),
                34 => Some(Self::Env1Sus),
                35 => Some(Self::Env1Rel),
                36 => Some(Self::Env1AtCur),
                37 => Some(Self::Env1DcyCur),
                38 => Some(Self::Env1SuScur),
                39 => Some(Self::Env1RelCur),
                40 => Some(Self::Env2Attack),
                41 => Some(Self::Env2Decay),
                42 => Some(Self::Env2Sus),
                43 => Some(Self::Env2Rel),
                44 => Some(Self::Env2AtCur),
                45 => Some(Self::Env2DcyCur),
                46 => Some(Self::Env2SuScur),
                47 => Some(Self::Env2RelCur),
                48 => Some(Self::Env3Attack),
                49 => Some(Self::Env3Decay),
                50 => Some(Self::Env3Sus),
                51 => Some(Self::Env3Rel),
                52 => Some(Self::Env3AtCur),
                53 => Some(Self::Env3DcyCur),
                54 => Some(Self::Env3SuScur),
                55 => Some(Self::Env3RelCur),
                56 => Some(Self::VcaAll),
                57 => Some(Self::VcaActive),
                58 => Some(Self::VcaEnvDep),
                59 => Some(Self::PanSpread),
                60 => Some(Self::VcaPan),
                61 => Some(Self::Osc2Lvl),
                62 => Some(Self::NoiseLvl),
                63 => Some(Self::HpFreq),
                64 => Some(Self::UniDetune),
                65 => Some(Self::OscDrift),
                66 => Some(Self::ParamDrift),
                67 => Some(Self::DriftRate),
                68 => Some(Self::ArpGate),
                69 => Some(Self::SeqSlew),
                70 => Some(Self::Mod1Dep),
                71 => Some(Self::Mod2Dep),
                72 => Some(Self::Mod3Dep),
                73 => Some(Self::Mod4Dep),
                74 => Some(Self::Mod5Dep),
                75 => Some(Self::Mod6Dep),
                76 => Some(Self::Mod7Dep),
                77 => Some(Self::Mod8Dep),
                78 => Some(Self::Fx1Param1),
                79 => Some(Self::Fx1Param2),
                80 => Some(Self::Fx1Param3),
                81 => Some(Self::Fx1Param4),
                82 => Some(Self::Fx1Param5),
                83 => Some(Self::Fx1Param6),
                84 => Some(Self::Fx1Param7),
                85 => Some(Self::Fx1Param8),
                86 => Some(Self::Fx1Param9),
                87 => Some(Self::Fx1Param10),
                88 => Some(Self::Fx1Param11),
                89 => Some(Self::Fx1Param12),
                90 => Some(Self::Fx2Param1),
                91 => Some(Self::Fx2Param2),
                92 => Some(Self::Fx2Param3),
                93 => Some(Self::Fx2Param4),
                94 => Some(Self::Fx2Param5),
                95 => Some(Self::Fx2Param6),
                96 => Some(Self::Fx2Param7),
                97 => Some(Self::Fx2Param8),
                98 => Some(Self::Fx2Param9),
                99 => Some(Self::Fx2Param10),
                100 => Some(Self::Fx2Param11),
                101 => Some(Self::Fx2Param12),
                102 => Some(Self::Fx3Param1),
                103 => Some(Self::Fx3Param2),
                104 => Some(Self::Fx3Param3),
                105 => Some(Self::Fx3Param4),
                106 => Some(Self::Fx3Param5),
                107 => Some(Self::Fx3Param6),
                108 => Some(Self::Fx3Param7),
                109 => Some(Self::Fx3Param8),
                110 => Some(Self::Fx3Param9),
                111 => Some(Self::Fx3Param10),
                112 => Some(Self::Fx3Param11),
                113 => Some(Self::Fx3Param12),
                114 => Some(Self::Fx4Param1),
                115 => Some(Self::Fx4Param2),
                116 => Some(Self::Fx4Param3),
                117 => Some(Self::Fx4Param4),
                118 => Some(Self::Fx4Param5),
                119 => Some(Self::Fx4Param6),
                120 => Some(Self::Fx4Param7),
                121 => Some(Self::Fx4Param8),
                122 => Some(Self::Fx4Param9),
                123 => Some(Self::Fx4Param10),
                124 => Some(Self::Fx4Param11),
                125 => Some(Self::Fx4Param12),
                126 => Some(Self::Fx1Level),
                127 => Some(Self::Fx2Level),
                128 => Some(Self::Fx3Level),
                129 => Some(Self::Fx4Level),
                _ => None,
            }
        }
    }

    /// Returns the byte that selects this value on [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
            Self::Off => 0,
            Self::Lfo1Rate => 1,
            Self::Lfo1Delay => 2,
            Self::Lfo1Slew => 3,
            Self::Lfo1Shape => 4,
            Self::Lfo2Rate => 5,
            Self::Lfo2Delay => 6,
            Self::Lfo2Slew => 7,
            Self::Lfo2Shape => 8,
            Self::Osc12Pit => 9,
            Self::Osc12Fine => 10,
            Self::Osc1Pitch => 11,
            Self::Osc1Fine => 12,
            Self::Osc2Pitch => 13,
            Self::Osc2Fine => 14,
            Self::Osc1PmDep => 15,
            Self::PwmDepth => 16,
            Self::TmodDepth => 17,
            Self::Osc2PmDep => 18,
            Self::PortaTime => 19,
            Self::VcfFreq => 20,
            Self::VcfRes => 21,
            Self::VcfEnv => 22,
            Self::VcfLfo => 23,
            Self::EnvRates => 24,
            Self::AllAttack => 25,
            Self::AllDecay => 26,
            Self::AllSus => 27,
            Self::AllRel => 28,
            Self::Env1Rates => 29,
            Self::Env2Rates => 30,
            Self::Env3Rates => 31,
            Self::Env1CurveS => 32,
            Self::Env2CurveS => 33,
            Self::Env3CurveS => 34,
            Self::Env1Attack => 35,
            Self::Env1Decay => 36,
            Self::Env1Sus => 37,
            Self::Env1Rel => 38,
            Self::Env1AtCur => 39,
            Self::Env1DcyCur => 40,
            Self::Env1SuScur => 41,
            Self::Env1RelCur => 42,
            Self::Env2Attack => 43,
            Self::Env2Decay => 44,
            Self::Env2Sus => 45,
            Self::Env2Rel => 46,
            Self::Env2AtCur => 47,
            Self::Env2DcyCur => 48,
            Self::Env2SuScur => 49,
            Self::Env2RelCur => 50,
            Self::Env3Attack => 51,
            Self::Env3Decay => 52,
            Self::Env3Sus => 53,
            Self::Env3Rel => 54,
            Self::Env3AtCur => 55,
            Self::Env3DcyCur => 56,
            Self::Env3SuScur => 57,
            Self::Env3RelCur => 58,
            Self::VcaAll => 59,
            Self::VcaActive => 60,
            Self::VcaEnvDep => 61,
            Self::PanSpread => 62,
            Self::VcaPan => 63,
            Self::Osc2Lvl => 64,
            Self::NoiseLvl => 65,
            Self::HpFreq => 66,
            Self::UniDetune => 67,
            Self::OscDrift => 68,
            Self::ParamDrift => 69,
            Self::DriftRate => 70,
            Self::ArpGate => 71,
            Self::SeqSlew => 72,
            Self::Mod1Dep => 73,
            Self::Mod2Dep => 74,
            Self::Mod3Dep => 75,
            Self::Mod4Dep => 76,
            Self::Mod5Dep => 77,
            Self::Mod6Dep => 78,
            Self::Mod7Dep => 79,
            Self::Mod8Dep => 80,
            Self::Fx1Param1 => 81,
            Self::Fx1Param2 => 82,
            Self::Fx1Param3 => 83,
            Self::Fx1Param4 => 84,
            Self::Fx1Param5 => 85,
            Self::Fx1Param6 => 86,
            Self::Fx1Param7 => 87,
            Self::Fx1Param8 => 88,
            Self::Fx1Param9 => 89,
            Self::Fx1Param10 => 90,
            Self::Fx1Param11 => 91,
            Self::Fx1Param12 => 92,
            Self::Fx2Param1 => 93,
            Self::Fx2Param2 => 94,
            Self::Fx2Param3 => 95,
            Self::Fx2Param4 => 96,
            Self::Fx2Param5 => 97,
            Self::Fx2Param6 => 98,
            Self::Fx2Param7 => 99,
            Self::Fx2Param8 => 100,
            Self::Fx2Param9 => 101,
            Self::Fx2Param10 => 102,
            Self::Fx2Param11 => 103,
            Self::Fx2Param12 => 104,
            Self::Fx3Param1 => 105,
            Self::Fx3Param2 => 106,
            Self::Fx3Param3 => 107,
            Self::Fx3Param4 => 108,
            Self::Fx3Param5 => 109,
            Self::Fx3Param6 => 110,
            Self::Fx3Param7 => 111,
            Self::Fx3Param8 => 112,
            Self::Fx3Param9 => 113,
            Self::Fx3Param10 => 114,
            Self::Fx3Param11 => 115,
            Self::Fx3Param12 => 116,
            Self::Fx4Param1 => 117,
            Self::Fx4Param2 => 118,
            Self::Fx4Param3 => 119,
            Self::Fx4Param4 => 120,
            Self::Fx4Param5 => 121,
            Self::Fx4Param6 => 122,
            Self::Fx4Param7 => 123,
            Self::Fx4Param8 => 124,
            Self::Fx4Param9 => 125,
            Self::Fx4Param10 => 126,
            Self::Fx4Param11 => 127,
            Self::Fx4Param12 => 128,
            Self::Fx1Level => 129,
            Self::Fx2Level => 130,
            Self::Fx3Level => 131,
            Self::Fx4Level => 132,
        }
    }

    /// Returns the byte that selects this value on the firmware a device inquiry
    /// reported.
    ///
    /// `None` for a value that firmware does not have.
    #[must_use]
    pub const fn raw_for(self, firmware: Version) -> Option<u8> {
        if firmware.at_least(1, 1) {
            match self {
                Self::Off => Some(0),
                Self::Lfo1Rate => Some(1),
                Self::Lfo1Delay => Some(2),
                Self::Lfo1Slew => Some(3),
                Self::Lfo1Shape => Some(4),
                Self::Lfo2Rate => Some(5),
                Self::Lfo2Delay => Some(6),
                Self::Lfo2Slew => Some(7),
                Self::Lfo2Shape => Some(8),
                Self::Osc12Pit => Some(9),
                Self::Osc12Fine => Some(10),
                Self::Osc1Pitch => Some(11),
                Self::Osc1Fine => Some(12),
                Self::Osc2Pitch => Some(13),
                Self::Osc2Fine => Some(14),
                Self::Osc1PmDep => Some(15),
                Self::PwmDepth => Some(16),
                Self::TmodDepth => Some(17),
                Self::Osc2PmDep => Some(18),
                Self::PortaTime => Some(19),
                Self::VcfFreq => Some(20),
                Self::VcfRes => Some(21),
                Self::VcfEnv => Some(22),
                Self::VcfLfo => Some(23),
                Self::EnvRates => Some(24),
                Self::AllAttack => Some(25),
                Self::AllDecay => Some(26),
                Self::AllSus => Some(27),
                Self::AllRel => Some(28),
                Self::Env1Rates => Some(29),
                Self::Env2Rates => Some(30),
                Self::Env3Rates => Some(31),
                Self::Env1CurveS => Some(32),
                Self::Env2CurveS => Some(33),
                Self::Env3CurveS => Some(34),
                Self::Env1Attack => Some(35),
                Self::Env1Decay => Some(36),
                Self::Env1Sus => Some(37),
                Self::Env1Rel => Some(38),
                Self::Env1AtCur => Some(39),
                Self::Env1DcyCur => Some(40),
                Self::Env1SuScur => Some(41),
                Self::Env1RelCur => Some(42),
                Self::Env2Attack => Some(43),
                Self::Env2Decay => Some(44),
                Self::Env2Sus => Some(45),
                Self::Env2Rel => Some(46),
                Self::Env2AtCur => Some(47),
                Self::Env2DcyCur => Some(48),
                Self::Env2SuScur => Some(49),
                Self::Env2RelCur => Some(50),
                Self::Env3Attack => Some(51),
                Self::Env3Decay => Some(52),
                Self::Env3Sus => Some(53),
                Self::Env3Rel => Some(54),
                Self::Env3AtCur => Some(55),
                Self::Env3DcyCur => Some(56),
                Self::Env3SuScur => Some(57),
                Self::Env3RelCur => Some(58),
                Self::VcaAll => Some(59),
                Self::VcaActive => Some(60),
                Self::VcaEnvDep => Some(61),
                Self::PanSpread => Some(62),
                Self::VcaPan => Some(63),
                Self::Osc2Lvl => Some(64),
                Self::NoiseLvl => Some(65),
                Self::HpFreq => Some(66),
                Self::UniDetune => Some(67),
                Self::OscDrift => Some(68),
                Self::ParamDrift => Some(69),
                Self::DriftRate => Some(70),
                Self::ArpGate => Some(71),
                Self::SeqSlew => Some(72),
                Self::Mod1Dep => Some(73),
                Self::Mod2Dep => Some(74),
                Self::Mod3Dep => Some(75),
                Self::Mod4Dep => Some(76),
                Self::Mod5Dep => Some(77),
                Self::Mod6Dep => Some(78),
                Self::Mod7Dep => Some(79),
                Self::Mod8Dep => Some(80),
                Self::Fx1Param1 => Some(81),
                Self::Fx1Param2 => Some(82),
                Self::Fx1Param3 => Some(83),
                Self::Fx1Param4 => Some(84),
                Self::Fx1Param5 => Some(85),
                Self::Fx1Param6 => Some(86),
                Self::Fx1Param7 => Some(87),
                Self::Fx1Param8 => Some(88),
                Self::Fx1Param9 => Some(89),
                Self::Fx1Param10 => Some(90),
                Self::Fx1Param11 => Some(91),
                Self::Fx1Param12 => Some(92),
                Self::Fx2Param1 => Some(93),
                Self::Fx2Param2 => Some(94),
                Self::Fx2Param3 => Some(95),
                Self::Fx2Param4 => Some(96),
                Self::Fx2Param5 => Some(97),
                Self::Fx2Param6 => Some(98),
                Self::Fx2Param7 => Some(99),
                Self::Fx2Param8 => Some(100),
                Self::Fx2Param9 => Some(101),
                Self::Fx2Param10 => Some(102),
                Self::Fx2Param11 => Some(103),
                Self::Fx2Param12 => Some(104),
                Self::Fx3Param1 => Some(105),
                Self::Fx3Param2 => Some(106),
                Self::Fx3Param3 => Some(107),
                Self::Fx3Param4 => Some(108),
                Self::Fx3Param5 => Some(109),
                Self::Fx3Param6 => Some(110),
                Self::Fx3Param7 => Some(111),
                Self::Fx3Param8 => Some(112),
                Self::Fx3Param9 => Some(113),
                Self::Fx3Param10 => Some(114),
                Self::Fx3Param11 => Some(115),
                Self::Fx3Param12 => Some(116),
                Self::Fx4Param1 => Some(117),
                Self::Fx4Param2 => Some(118),
                Self::Fx4Param3 => Some(119),
                Self::Fx4Param4 => Some(120),
                Self::Fx4Param5 => Some(121),
                Self::Fx4Param6 => Some(122),
                Self::Fx4Param7 => Some(123),
                Self::Fx4Param8 => Some(124),
                Self::Fx4Param9 => Some(125),
                Self::Fx4Param10 => Some(126),
                Self::Fx4Param11 => Some(127),
                Self::Fx4Param12 => Some(128),
                Self::Fx1Level => Some(129),
                Self::Fx2Level => Some(130),
                Self::Fx3Level => Some(131),
                Self::Fx4Level => Some(132),
            }
        } else {
            match self {
                Self::Off => Some(0),
                Self::Lfo1Rate => Some(1),
                Self::Lfo1Delay => Some(2),
                Self::Lfo1Slew => Some(3),
                Self::Lfo1Shape => Some(4),
                Self::Lfo2Rate => Some(5),
                Self::Lfo2Delay => Some(6),
                Self::Lfo2Slew => Some(7),
                Self::Lfo2Shape => Some(8),
                Self::Osc12Pit => Some(9),
                Self::Osc12Fine => None,
                Self::Osc1Pitch => Some(10),
                Self::Osc1Fine => None,
                Self::Osc2Pitch => Some(11),
                Self::Osc2Fine => None,
                Self::Osc1PmDep => Some(12),
                Self::PwmDepth => Some(13),
                Self::TmodDepth => Some(14),
                Self::Osc2PmDep => Some(15),
                Self::PortaTime => Some(16),
                Self::VcfFreq => Some(17),
                Self::VcfRes => Some(18),
                Self::VcfEnv => Some(19),
                Self::VcfLfo => Some(20),
                Self::EnvRates => Some(21),
                Self::AllAttack => Some(22),
                Self::AllDecay => Some(23),
                Self::AllSus => Some(24),
                Self::AllRel => Some(25),
                Self::Env1Rates => Some(26),
                Self::Env2Rates => Some(27),
                Self::Env3Rates => Some(28),
                Self::Env1CurveS => Some(29),
                Self::Env2CurveS => Some(30),
                Self::Env3CurveS => Some(31),
                Self::Env1Attack => Some(32),
                Self::Env1Decay => Some(33),
                Self::Env1Sus => Some(34),
                Self::Env1Rel => Some(35),
                Self::Env1AtCur => Some(36),
                Self::Env1DcyCur => Some(37),
                Self::Env1SuScur => Some(38),
                Self::Env1RelCur => Some(39),
                Self::Env2Attack => Some(40),
                Self::Env2Decay => Some(41),
                Self::Env2Sus => Some(42),
                Self::Env2Rel => Some(43),
                Self::Env2AtCur => Some(44),
                Self::Env2DcyCur => Some(45),
                Self::Env2SuScur => Some(46),
                Self::Env2RelCur => Some(47),
                Self::Env3Attack => Some(48),
                Self::Env3Decay => Some(49),
                Self::Env3Sus => Some(50),
                Self::Env3Rel => Some(51),
                Self::Env3AtCur => Some(52),
                Self::Env3DcyCur => Some(53),
                Self::Env3SuScur => Some(54),
                Self::Env3RelCur => Some(55),
                Self::VcaAll => Some(56),
                Self::VcaActive => Some(57),
                Self::VcaEnvDep => Some(58),
                Self::PanSpread => Some(59),
                Self::VcaPan => Some(60),
                Self::Osc2Lvl => Some(61),
                Self::NoiseLvl => Some(62),
                Self::HpFreq => Some(63),
                Self::UniDetune => Some(64),
                Self::OscDrift => Some(65),
                Self::ParamDrift => Some(66),
                Self::DriftRate => Some(67),
                Self::ArpGate => Some(68),
                Self::SeqSlew => Some(69),
                Self::Mod1Dep => Some(70),
                Self::Mod2Dep => Some(71),
                Self::Mod3Dep => Some(72),
                Self::Mod4Dep => Some(73),
                Self::Mod5Dep => Some(74),
                Self::Mod6Dep => Some(75),
                Self::Mod7Dep => Some(76),
                Self::Mod8Dep => Some(77),
                Self::Fx1Param1 => Some(78),
                Self::Fx1Param2 => Some(79),
                Self::Fx1Param3 => Some(80),
                Self::Fx1Param4 => Some(81),
                Self::Fx1Param5 => Some(82),
                Self::Fx1Param6 => Some(83),
                Self::Fx1Param7 => Some(84),
                Self::Fx1Param8 => Some(85),
                Self::Fx1Param9 => Some(86),
                Self::Fx1Param10 => Some(87),
                Self::Fx1Param11 => Some(88),
                Self::Fx1Param12 => Some(89),
                Self::Fx2Param1 => Some(90),
                Self::Fx2Param2 => Some(91),
                Self::Fx2Param3 => Some(92),
                Self::Fx2Param4 => Some(93),
                Self::Fx2Param5 => Some(94),
                Self::Fx2Param6 => Some(95),
                Self::Fx2Param7 => Some(96),
                Self::Fx2Param8 => Some(97),
                Self::Fx2Param9 => Some(98),
                Self::Fx2Param10 => Some(99),
                Self::Fx2Param11 => Some(100),
                Self::Fx2Param12 => Some(101),
                Self::Fx3Param1 => Some(102),
                Self::Fx3Param2 => Some(103),
                Self::Fx3Param3 => Some(104),
                Self::Fx3Param4 => Some(105),
                Self::Fx3Param5 => Some(106),
                Self::Fx3Param6 => Some(107),
                Self::Fx3Param7 => Some(108),
                Self::Fx3Param8 => Some(109),
                Self::Fx3Param9 => Some(110),
                Self::Fx3Param10 => Some(111),
                Self::Fx3Param11 => Some(112),
                Self::Fx3Param12 => Some(113),
                Self::Fx4Param1 => Some(114),
                Self::Fx4Param2 => Some(115),
                Self::Fx4Param3 => Some(116),
                Self::Fx4Param4 => Some(117),
                Self::Fx4Param5 => Some(118),
                Self::Fx4Param6 => Some(119),
                Self::Fx4Param7 => Some(120),
                Self::Fx4Param8 => Some(121),
                Self::Fx4Param9 => Some(122),
                Self::Fx4Param10 => Some(123),
                Self::Fx4Param11 => Some(124),
                Self::Fx4Param12 => Some(125),
                Self::Fx1Level => Some(126),
                Self::Fx2Level => Some(127),
                Self::Fx3Level => Some(128),
                Self::Fx4Level => Some(129),
            }
        }
    }

    /// Returns the name the synthesizer's display writes, as of
    /// [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub fn name(self) -> &'static str {
        self.name_for(DEFAULT_FIRMWARE).unwrap_or_default()
    }

    /// Returns the name the synthesizer's display writes on the firmware a
    /// device inquiry reported.
    ///
    /// `None` for a value that firmware does not have. The names live in the
    /// value table the parameter layer already carries; nothing here holds a
    /// second copy of them.
    #[must_use]
    pub fn name_for(self, firmware: Version) -> Option<&'static str> {
        let raw = self.raw_for(firmware)?;
        Self::TABLE.table_for(firmware).name_of(u16::from(raw))
    }
}

impl fmt::Display for ModDestination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Modulation Matrix Source.
///
/// Firmware renumbered this table rather than extending it, so a variant is what
/// the value means and not the byte it travels as. [`ModSource::raw_for`] is that
/// byte on a given firmware, and [`ModSource::raw`] is it on the newest.
///
/// Every value the synthesizer has a name for. A byte outside the table is not
/// one of these: [`ModSource::from_raw`] answers `None` for it, and the byte stays
/// in the program either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum ModSource {
    /// Off.
    Off,
    /// Pitch Bend.
    PitchBend,
    /// Mod Wheel.
    ModWheel,
    /// Foot Ctrl.
    FootCtrl,
    /// `BreathCtrl`.
    BreathCtrl,
    /// Pressure.
    Pressure,
    /// Expression.
    Expression,
    /// LFO1.
    Lfo1,
    /// LFO2.
    Lfo2,
    /// Env 1.
    Env1,
    /// Env 2.
    Env2,
    /// Env 3.
    Env3,
    /// Note Num.
    NoteNum,
    /// Note Vel.
    NoteVel,
    /// Note Off Vel.
    NoteOffVel,
    /// Ctrl Seq.
    CtrlSeq,
    /// LFO1 `(Uni)`.
    Lfo1Uni,
    /// LFO2 `(Uni)`.
    Lfo2Uni,
    /// LFO1 `(Fade)`.
    Lfo1Fade,
    /// LFO2 `(Fade)`.
    Lfo2Fade,
    /// Voice Num.
    VoiceNum,
    /// Uni Voice.
    UniVoice,
    /// CC X (115).
    CcX,
    /// CC Y (116).
    CcY,
    /// CC Z (117).
    CcZ,
}

impl ModSource {
    /// The value table this type lists.
    pub const TABLE: TableId = TableId::ModSource;

    /// Every value, in the order the newest firmware numbers them.
    pub const ALL: &'static [Self] = &[
        Self::Off,
        Self::PitchBend,
        Self::ModWheel,
        Self::FootCtrl,
        Self::BreathCtrl,
        Self::Pressure,
        Self::Expression,
        Self::Lfo1,
        Self::Lfo2,
        Self::Env1,
        Self::Env2,
        Self::Env3,
        Self::NoteNum,
        Self::NoteVel,
        Self::NoteOffVel,
        Self::CtrlSeq,
        Self::Lfo1Uni,
        Self::Lfo2Uni,
        Self::Lfo1Fade,
        Self::Lfo2Fade,
        Self::VoiceNum,
        Self::UniVoice,
        Self::CcX,
        Self::CcY,
        Self::CcZ,
    ];

    /// Returns the value this byte selects on [`DEFAULT_FIRMWARE`].
    ///
    /// `None` when the table does not list the byte.
    #[must_use]
    pub const fn from_raw(raw: u8) -> Option<Self> {
        Self::from_raw_for(raw, DEFAULT_FIRMWARE)
    }

    /// Returns the value this byte selects on the firmware a device inquiry
    /// reported.
    ///
    /// Worth reaching for whenever the byte came from a stored program: nothing
    /// in a dump says which firmware wrote it, and this table was renumbered.
    #[must_use]
    pub const fn from_raw_for(raw: u8, firmware: Version) -> Option<Self> {
        if firmware.at_least(1, 1) {
            match raw {
                0 => Some(Self::Off),
                1 => Some(Self::PitchBend),
                2 => Some(Self::ModWheel),
                3 => Some(Self::FootCtrl),
                4 => Some(Self::BreathCtrl),
                5 => Some(Self::Pressure),
                6 => Some(Self::Expression),
                7 => Some(Self::Lfo1),
                8 => Some(Self::Lfo2),
                9 => Some(Self::Env1),
                10 => Some(Self::Env2),
                11 => Some(Self::Env3),
                12 => Some(Self::NoteNum),
                13 => Some(Self::NoteVel),
                14 => Some(Self::NoteOffVel),
                15 => Some(Self::CtrlSeq),
                16 => Some(Self::Lfo1Uni),
                17 => Some(Self::Lfo2Uni),
                18 => Some(Self::Lfo1Fade),
                19 => Some(Self::Lfo2Fade),
                20 => Some(Self::VoiceNum),
                21 => Some(Self::UniVoice),
                22 => Some(Self::CcX),
                23 => Some(Self::CcY),
                24 => Some(Self::CcZ),
                _ => None,
            }
        } else {
            match raw {
                0 => Some(Self::Off),
                1 => Some(Self::PitchBend),
                2 => Some(Self::ModWheel),
                3 => Some(Self::FootCtrl),
                4 => Some(Self::BreathCtrl),
                5 => Some(Self::Pressure),
                6 => Some(Self::Lfo1),
                7 => Some(Self::Lfo2),
                8 => Some(Self::Env1),
                9 => Some(Self::Env2),
                10 => Some(Self::Env3),
                11 => Some(Self::NoteNum),
                12 => Some(Self::NoteVel),
                13 => Some(Self::CtrlSeq),
                14 => Some(Self::Lfo1Uni),
                15 => Some(Self::Lfo2Uni),
                16 => Some(Self::Lfo1Fade),
                17 => Some(Self::Lfo2Fade),
                18 => Some(Self::NoteOffVel),
                19 => Some(Self::VoiceNum),
                20 => Some(Self::CcX),
                21 => Some(Self::CcY),
                22 => Some(Self::CcZ),
                _ => None,
            }
        }
    }

    /// Returns the byte that selects this value on [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
            Self::Off => 0,
            Self::PitchBend => 1,
            Self::ModWheel => 2,
            Self::FootCtrl => 3,
            Self::BreathCtrl => 4,
            Self::Pressure => 5,
            Self::Expression => 6,
            Self::Lfo1 => 7,
            Self::Lfo2 => 8,
            Self::Env1 => 9,
            Self::Env2 => 10,
            Self::Env3 => 11,
            Self::NoteNum => 12,
            Self::NoteVel => 13,
            Self::NoteOffVel => 14,
            Self::CtrlSeq => 15,
            Self::Lfo1Uni => 16,
            Self::Lfo2Uni => 17,
            Self::Lfo1Fade => 18,
            Self::Lfo2Fade => 19,
            Self::VoiceNum => 20,
            Self::UniVoice => 21,
            Self::CcX => 22,
            Self::CcY => 23,
            Self::CcZ => 24,
        }
    }

    /// Returns the byte that selects this value on the firmware a device inquiry
    /// reported.
    ///
    /// `None` for a value that firmware does not have.
    #[must_use]
    pub const fn raw_for(self, firmware: Version) -> Option<u8> {
        if firmware.at_least(1, 1) {
            match self {
                Self::Off => Some(0),
                Self::PitchBend => Some(1),
                Self::ModWheel => Some(2),
                Self::FootCtrl => Some(3),
                Self::BreathCtrl => Some(4),
                Self::Pressure => Some(5),
                Self::Expression => Some(6),
                Self::Lfo1 => Some(7),
                Self::Lfo2 => Some(8),
                Self::Env1 => Some(9),
                Self::Env2 => Some(10),
                Self::Env3 => Some(11),
                Self::NoteNum => Some(12),
                Self::NoteVel => Some(13),
                Self::NoteOffVel => Some(14),
                Self::CtrlSeq => Some(15),
                Self::Lfo1Uni => Some(16),
                Self::Lfo2Uni => Some(17),
                Self::Lfo1Fade => Some(18),
                Self::Lfo2Fade => Some(19),
                Self::VoiceNum => Some(20),
                Self::UniVoice => Some(21),
                Self::CcX => Some(22),
                Self::CcY => Some(23),
                Self::CcZ => Some(24),
            }
        } else {
            match self {
                Self::Off => Some(0),
                Self::PitchBend => Some(1),
                Self::ModWheel => Some(2),
                Self::FootCtrl => Some(3),
                Self::BreathCtrl => Some(4),
                Self::Pressure => Some(5),
                Self::Expression => None,
                Self::Lfo1 => Some(6),
                Self::Lfo2 => Some(7),
                Self::Env1 => Some(8),
                Self::Env2 => Some(9),
                Self::Env3 => Some(10),
                Self::NoteNum => Some(11),
                Self::NoteVel => Some(12),
                Self::NoteOffVel => Some(18),
                Self::CtrlSeq => Some(13),
                Self::Lfo1Uni => Some(14),
                Self::Lfo2Uni => Some(15),
                Self::Lfo1Fade => Some(16),
                Self::Lfo2Fade => Some(17),
                Self::VoiceNum => Some(19),
                Self::UniVoice => None,
                Self::CcX => Some(20),
                Self::CcY => Some(21),
                Self::CcZ => Some(22),
            }
        }
    }

    /// Returns the name the synthesizer's display writes, as of
    /// [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub fn name(self) -> &'static str {
        self.name_for(DEFAULT_FIRMWARE).unwrap_or_default()
    }

    /// Returns the name the synthesizer's display writes on the firmware a
    /// device inquiry reported.
    ///
    /// `None` for a value that firmware does not have. The names live in the
    /// value table the parameter layer already carries; nothing here holds a
    /// second copy of them.
    #[must_use]
    pub fn name_for(self, firmware: Version) -> Option<&'static str> {
        let raw = self.raw_for(firmware)?;
        Self::TABLE.table_for(firmware).name_of(u16::from(raw))
    }
}

impl fmt::Display for ModSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// OSC 1 Pitch Mod Mode.
///
/// Every value the synthesizer has a name for. A byte outside the table is not
/// one of these: [`Osc1PitchModMode::from_raw`] answers `None` for it, and the byte stays
/// in the program either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Osc1PitchModMode {
    /// OSC 1 + 2.
    Osc12,
    /// OSC 1 only.
    Osc1Only,
}

impl Osc1PitchModMode {
    /// The value table this type lists.
    pub const TABLE: TableId = TableId::Osc1PitchModMode;

    /// Every value, in the order the newest firmware numbers them.
    pub const ALL: &'static [Self] = &[Self::Osc12, Self::Osc1Only];

    /// Returns the value this byte selects on [`DEFAULT_FIRMWARE`].
    ///
    /// `None` when the table does not list the byte.
    #[must_use]
    pub const fn from_raw(raw: u8) -> Option<Self> {
        Self::from_raw_for(raw, DEFAULT_FIRMWARE)
    }

    /// Returns the value this byte selects on the firmware a device inquiry
    /// reported.
    ///
    /// Every firmware numbers this table the same way, so the version makes no
    /// difference here; it is taken so that every value type reads alike.
    #[must_use]
    pub const fn from_raw_for(raw: u8, _firmware: Version) -> Option<Self> {
        match raw {
            0 => Some(Self::Osc12),
            1 => Some(Self::Osc1Only),
            _ => None,
        }
    }

    /// Returns the byte that selects this value on [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
            Self::Osc12 => 0,
            Self::Osc1Only => 1,
        }
    }

    /// Returns the byte that selects this value on the firmware a device inquiry
    /// reported.
    ///
    /// Always `Some` for this table, which every firmware numbers the same way.
    #[must_use]
    pub const fn raw_for(self, _firmware: Version) -> Option<u8> {
        Some(self.raw())
    }

    /// Returns the name the synthesizer's display writes, as of
    /// [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub fn name(self) -> &'static str {
        self.name_for(DEFAULT_FIRMWARE).unwrap_or_default()
    }

    /// Returns the name the synthesizer's display writes on the firmware a
    /// device inquiry reported.
    ///
    /// `None` for a value that firmware does not have. The names live in the
    /// value table the parameter layer already carries; nothing here holds a
    /// second copy of them.
    #[must_use]
    pub fn name_for(self, firmware: Version) -> Option<&'static str> {
        let raw = self.raw_for(firmware)?;
        Self::TABLE.table_for(firmware).name_of(u16::from(raw))
    }
}

impl fmt::Display for Osc1PitchModMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Oscillator Range.
///
/// Every value the synthesizer has a name for. A byte outside the table is not
/// one of these: [`OscRange::from_raw`] answers `None` for it, and the byte stays
/// in the program either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum OscRange {
    /// 16'.
    SixteenFoot,
    /// 8'.
    EightFoot,
    /// 4'.
    FourFoot,
}

impl OscRange {
    /// The value table this type lists.
    pub const TABLE: TableId = TableId::OscRange;

    /// Every value, in the order the newest firmware numbers them.
    pub const ALL: &'static [Self] = &[Self::SixteenFoot, Self::EightFoot, Self::FourFoot];

    /// Returns the value this byte selects on [`DEFAULT_FIRMWARE`].
    ///
    /// `None` when the table does not list the byte.
    #[must_use]
    pub const fn from_raw(raw: u8) -> Option<Self> {
        Self::from_raw_for(raw, DEFAULT_FIRMWARE)
    }

    /// Returns the value this byte selects on the firmware a device inquiry
    /// reported.
    ///
    /// Every firmware numbers this table the same way, so the version makes no
    /// difference here; it is taken so that every value type reads alike.
    #[must_use]
    pub const fn from_raw_for(raw: u8, _firmware: Version) -> Option<Self> {
        match raw {
            0 => Some(Self::SixteenFoot),
            1 => Some(Self::EightFoot),
            2 => Some(Self::FourFoot),
            _ => None,
        }
    }

    /// Returns the byte that selects this value on [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
            Self::SixteenFoot => 0,
            Self::EightFoot => 1,
            Self::FourFoot => 2,
        }
    }

    /// Returns the byte that selects this value on the firmware a device inquiry
    /// reported.
    ///
    /// Always `Some` for this table, which every firmware numbers the same way.
    #[must_use]
    pub const fn raw_for(self, _firmware: Version) -> Option<u8> {
        Some(self.raw())
    }

    /// Returns the name the synthesizer's display writes, as of
    /// [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub fn name(self) -> &'static str {
        self.name_for(DEFAULT_FIRMWARE).unwrap_or_default()
    }

    /// Returns the name the synthesizer's display writes on the firmware a
    /// device inquiry reported.
    ///
    /// `None` for a value that firmware does not have. The names live in the
    /// value table the parameter layer already carries; nothing here holds a
    /// second copy of them.
    #[must_use]
    pub fn name_for(self, firmware: Version) -> Option<&'static str> {
        let raw = self.raw_for(firmware)?;
        Self::TABLE.table_for(firmware).name_of(u16::from(raw))
    }
}

impl fmt::Display for OscRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Oscillator Pitch Mod Source.
///
/// Every value the synthesizer has a name for. A byte outside the table is not
/// one of these: [`PitchModSource::from_raw`] answers `None` for it, and the byte stays
/// in the program either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum PitchModSource {
    /// LFO 1.
    Lfo1,
    /// LFO 2.
    Lfo2,
    /// VCA Env.
    VcaEnv,
    /// VCF Env.
    VcfEnv,
    /// Mod Env.
    ModEnv,
    /// LFO 1 Unipolar.
    Lfo1Unipolar,
    /// LFO 2 Unipolar.
    Lfo2Unipolar,
}

impl PitchModSource {
    /// The value table this type lists.
    pub const TABLE: TableId = TableId::PitchModSource;

    /// Every value, in the order the newest firmware numbers them.
    pub const ALL: &'static [Self] = &[
        Self::Lfo1,
        Self::Lfo2,
        Self::VcaEnv,
        Self::VcfEnv,
        Self::ModEnv,
        Self::Lfo1Unipolar,
        Self::Lfo2Unipolar,
    ];

    /// Returns the value this byte selects on [`DEFAULT_FIRMWARE`].
    ///
    /// `None` when the table does not list the byte.
    #[must_use]
    pub const fn from_raw(raw: u8) -> Option<Self> {
        Self::from_raw_for(raw, DEFAULT_FIRMWARE)
    }

    /// Returns the value this byte selects on the firmware a device inquiry
    /// reported.
    ///
    /// Every firmware numbers this table the same way, so the version makes no
    /// difference here; it is taken so that every value type reads alike.
    #[must_use]
    pub const fn from_raw_for(raw: u8, _firmware: Version) -> Option<Self> {
        match raw {
            0 => Some(Self::Lfo1),
            1 => Some(Self::Lfo2),
            2 => Some(Self::VcaEnv),
            3 => Some(Self::VcfEnv),
            4 => Some(Self::ModEnv),
            5 => Some(Self::Lfo1Unipolar),
            6 => Some(Self::Lfo2Unipolar),
            _ => None,
        }
    }

    /// Returns the byte that selects this value on [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
            Self::Lfo1 => 0,
            Self::Lfo2 => 1,
            Self::VcaEnv => 2,
            Self::VcfEnv => 3,
            Self::ModEnv => 4,
            Self::Lfo1Unipolar => 5,
            Self::Lfo2Unipolar => 6,
        }
    }

    /// Returns the byte that selects this value on the firmware a device inquiry
    /// reported.
    ///
    /// Always `Some` for this table, which every firmware numbers the same way.
    #[must_use]
    pub const fn raw_for(self, _firmware: Version) -> Option<u8> {
        Some(self.raw())
    }

    /// Returns the name the synthesizer's display writes, as of
    /// [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub fn name(self) -> &'static str {
        self.name_for(DEFAULT_FIRMWARE).unwrap_or_default()
    }

    /// Returns the name the synthesizer's display writes on the firmware a
    /// device inquiry reported.
    ///
    /// `None` for a value that firmware does not have. The names live in the
    /// value table the parameter layer already carries; nothing here holds a
    /// second copy of them.
    #[must_use]
    pub fn name_for(self, firmware: Version) -> Option<&'static str> {
        let raw = self.raw_for(firmware)?;
        Self::TABLE.table_for(firmware).name_of(u16::from(raw))
    }
}

impl fmt::Display for PitchModSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Polyphony Mode.
///
/// Every value the synthesizer has a name for. A byte outside the table is not
/// one of these: [`PolyphonyMode::from_raw`] answers `None` for it, and the byte stays
/// in the program either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum PolyphonyMode {
    /// Poly.
    Poly,
    /// Unison 2.
    Unison2,
    /// Unison 3.
    Unison3,
    /// Unison 4.
    Unison4,
    /// Unison 6.
    Unison6,
    /// Unison 12.
    Unison12,
    /// Mono.
    Mono,
    /// Mono 2.
    Mono2,
    /// Mono 3.
    Mono3,
    /// Mono 4.
    Mono4,
    /// Mono 6.
    Mono6,
    /// Poly 6.
    Poly6,
    /// Poly 8.
    Poly8,
}

impl PolyphonyMode {
    /// The value table this type lists.
    pub const TABLE: TableId = TableId::PolyphonyMode;

    /// Every value, in the order the newest firmware numbers them.
    pub const ALL: &'static [Self] = &[
        Self::Poly,
        Self::Unison2,
        Self::Unison3,
        Self::Unison4,
        Self::Unison6,
        Self::Unison12,
        Self::Mono,
        Self::Mono2,
        Self::Mono3,
        Self::Mono4,
        Self::Mono6,
        Self::Poly6,
        Self::Poly8,
    ];

    /// Returns the value this byte selects on [`DEFAULT_FIRMWARE`].
    ///
    /// `None` when the table does not list the byte.
    #[must_use]
    pub const fn from_raw(raw: u8) -> Option<Self> {
        Self::from_raw_for(raw, DEFAULT_FIRMWARE)
    }

    /// Returns the value this byte selects on the firmware a device inquiry
    /// reported.
    ///
    /// Every firmware numbers this table the same way, so the version makes no
    /// difference here; it is taken so that every value type reads alike.
    #[must_use]
    pub const fn from_raw_for(raw: u8, _firmware: Version) -> Option<Self> {
        match raw {
            0 => Some(Self::Poly),
            1 => Some(Self::Unison2),
            2 => Some(Self::Unison3),
            3 => Some(Self::Unison4),
            4 => Some(Self::Unison6),
            5 => Some(Self::Unison12),
            6 => Some(Self::Mono),
            7 => Some(Self::Mono2),
            8 => Some(Self::Mono3),
            9 => Some(Self::Mono4),
            10 => Some(Self::Mono6),
            11 => Some(Self::Poly6),
            12 => Some(Self::Poly8),
            _ => None,
        }
    }

    /// Returns the byte that selects this value on [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
            Self::Poly => 0,
            Self::Unison2 => 1,
            Self::Unison3 => 2,
            Self::Unison4 => 3,
            Self::Unison6 => 4,
            Self::Unison12 => 5,
            Self::Mono => 6,
            Self::Mono2 => 7,
            Self::Mono3 => 8,
            Self::Mono4 => 9,
            Self::Mono6 => 10,
            Self::Poly6 => 11,
            Self::Poly8 => 12,
        }
    }

    /// Returns the byte that selects this value on the firmware a device inquiry
    /// reported.
    ///
    /// Always `Some` for this table, which every firmware numbers the same way.
    #[must_use]
    pub const fn raw_for(self, _firmware: Version) -> Option<u8> {
        Some(self.raw())
    }

    /// Returns the name the synthesizer's display writes, as of
    /// [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub fn name(self) -> &'static str {
        self.name_for(DEFAULT_FIRMWARE).unwrap_or_default()
    }

    /// Returns the name the synthesizer's display writes on the firmware a
    /// device inquiry reported.
    ///
    /// `None` for a value that firmware does not have. The names live in the
    /// value table the parameter layer already carries; nothing here holds a
    /// second copy of them.
    #[must_use]
    pub fn name_for(self, firmware: Version) -> Option<&'static str> {
        let raw = self.raw_for(firmware)?;
        Self::TABLE.table_for(firmware).name_of(u16::from(raw))
    }
}

impl fmt::Display for PolyphonyMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Portamento Mode.
///
/// Every value the synthesizer has a name for. A byte outside the table is not
/// one of these: [`PortamentoMode::from_raw`] answers `None` for it, and the byte stays
/// in the program either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum PortamentoMode {
    /// Normal.
    Normal,
    /// Fingered.
    Fingered,
    /// Fixed Rate.
    FixedRate,
    /// Fixed Rate Fingered.
    FixedRateFingered,
    /// Exponential.
    Exponential,
    /// Exponential Fingered.
    ExponentialFingered,
    /// Fixed +2.
    FixedPlus2,
    /// Fixed -2.
    FixedMinus2,
    /// Fixed +5.
    FixedPlus5,
    /// Fixed -5.
    FixedMinus5,
    /// Fixed +12.
    FixedPlus12,
    /// Fixed -12.
    FixedMinus12,
    /// Fixed +24.
    FixedPlus24,
    /// Fixed -24.
    FixedMinus24,
}

impl PortamentoMode {
    /// The value table this type lists.
    pub const TABLE: TableId = TableId::PortamentoMode;

    /// Every value, in the order the newest firmware numbers them.
    pub const ALL: &'static [Self] = &[
        Self::Normal,
        Self::Fingered,
        Self::FixedRate,
        Self::FixedRateFingered,
        Self::Exponential,
        Self::ExponentialFingered,
        Self::FixedPlus2,
        Self::FixedMinus2,
        Self::FixedPlus5,
        Self::FixedMinus5,
        Self::FixedPlus12,
        Self::FixedMinus12,
        Self::FixedPlus24,
        Self::FixedMinus24,
    ];

    /// Returns the value this byte selects on [`DEFAULT_FIRMWARE`].
    ///
    /// `None` when the table does not list the byte.
    #[must_use]
    pub const fn from_raw(raw: u8) -> Option<Self> {
        Self::from_raw_for(raw, DEFAULT_FIRMWARE)
    }

    /// Returns the value this byte selects on the firmware a device inquiry
    /// reported.
    ///
    /// Every firmware numbers this table the same way, so the version makes no
    /// difference here; it is taken so that every value type reads alike.
    #[must_use]
    pub const fn from_raw_for(raw: u8, _firmware: Version) -> Option<Self> {
        match raw {
            0 => Some(Self::Normal),
            1 => Some(Self::Fingered),
            2 => Some(Self::FixedRate),
            3 => Some(Self::FixedRateFingered),
            4 => Some(Self::Exponential),
            5 => Some(Self::ExponentialFingered),
            6 => Some(Self::FixedPlus2),
            7 => Some(Self::FixedMinus2),
            8 => Some(Self::FixedPlus5),
            9 => Some(Self::FixedMinus5),
            10 => Some(Self::FixedPlus12),
            11 => Some(Self::FixedMinus12),
            12 => Some(Self::FixedPlus24),
            13 => Some(Self::FixedMinus24),
            _ => None,
        }
    }

    /// Returns the byte that selects this value on [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
            Self::Normal => 0,
            Self::Fingered => 1,
            Self::FixedRate => 2,
            Self::FixedRateFingered => 3,
            Self::Exponential => 4,
            Self::ExponentialFingered => 5,
            Self::FixedPlus2 => 6,
            Self::FixedMinus2 => 7,
            Self::FixedPlus5 => 8,
            Self::FixedMinus5 => 9,
            Self::FixedPlus12 => 10,
            Self::FixedMinus12 => 11,
            Self::FixedPlus24 => 12,
            Self::FixedMinus24 => 13,
        }
    }

    /// Returns the byte that selects this value on the firmware a device inquiry
    /// reported.
    ///
    /// Always `Some` for this table, which every firmware numbers the same way.
    #[must_use]
    pub const fn raw_for(self, _firmware: Version) -> Option<u8> {
        Some(self.raw())
    }

    /// Returns the name the synthesizer's display writes, as of
    /// [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub fn name(self) -> &'static str {
        self.name_for(DEFAULT_FIRMWARE).unwrap_or_default()
    }

    /// Returns the name the synthesizer's display writes on the firmware a
    /// device inquiry reported.
    ///
    /// `None` for a value that firmware does not have. The names live in the
    /// value table the parameter layer already carries; nothing here holds a
    /// second copy of them.
    #[must_use]
    pub fn name_for(self, firmware: Version) -> Option<&'static str> {
        let raw = self.raw_for(firmware)?;
        Self::TABLE.table_for(firmware).name_of(u16::from(raw))
    }
}

impl fmt::Display for PortamentoMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Program Category.
///
/// Every value the synthesizer has a name for. A byte outside the table is not
/// one of these: [`ProgramCategory::from_raw`] answers `None` for it, and the byte stays
/// in the program either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum ProgramCategory {
    /// None.
    None,
    /// Bass.
    Bass,
    /// Pad.
    Pad,
    /// Lead.
    Lead,
    /// Mono.
    Mono,
    /// Poly.
    Poly,
    /// Stab.
    Stab,
    /// SFX.
    Sfx,
    /// Arp.
    Arp,
    /// Seq.
    Seq,
    /// Perc.
    Perc,
    /// Ambient.
    Ambient,
    /// Modular.
    Modular,
    /// User-1.
    User1,
    /// User-2.
    User2,
    /// User-3.
    User3,
    /// User-4.
    User4,
}

impl ProgramCategory {
    /// The value table this type lists.
    pub const TABLE: TableId = TableId::ProgramCategory;

    /// Every value, in the order the newest firmware numbers them.
    pub const ALL: &'static [Self] = &[
        Self::None,
        Self::Bass,
        Self::Pad,
        Self::Lead,
        Self::Mono,
        Self::Poly,
        Self::Stab,
        Self::Sfx,
        Self::Arp,
        Self::Seq,
        Self::Perc,
        Self::Ambient,
        Self::Modular,
        Self::User1,
        Self::User2,
        Self::User3,
        Self::User4,
    ];

    /// Returns the value this byte selects on [`DEFAULT_FIRMWARE`].
    ///
    /// `None` when the table does not list the byte.
    #[must_use]
    pub const fn from_raw(raw: u8) -> Option<Self> {
        Self::from_raw_for(raw, DEFAULT_FIRMWARE)
    }

    /// Returns the value this byte selects on the firmware a device inquiry
    /// reported.
    ///
    /// Every firmware numbers this table the same way, so the version makes no
    /// difference here; it is taken so that every value type reads alike.
    #[must_use]
    pub const fn from_raw_for(raw: u8, _firmware: Version) -> Option<Self> {
        match raw {
            0 => Some(Self::None),
            1 => Some(Self::Bass),
            2 => Some(Self::Pad),
            3 => Some(Self::Lead),
            4 => Some(Self::Mono),
            5 => Some(Self::Poly),
            6 => Some(Self::Stab),
            7 => Some(Self::Sfx),
            8 => Some(Self::Arp),
            9 => Some(Self::Seq),
            10 => Some(Self::Perc),
            11 => Some(Self::Ambient),
            12 => Some(Self::Modular),
            13 => Some(Self::User1),
            14 => Some(Self::User2),
            15 => Some(Self::User3),
            16 => Some(Self::User4),
            _ => None,
        }
    }

    /// Returns the byte that selects this value on [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Bass => 1,
            Self::Pad => 2,
            Self::Lead => 3,
            Self::Mono => 4,
            Self::Poly => 5,
            Self::Stab => 6,
            Self::Sfx => 7,
            Self::Arp => 8,
            Self::Seq => 9,
            Self::Perc => 10,
            Self::Ambient => 11,
            Self::Modular => 12,
            Self::User1 => 13,
            Self::User2 => 14,
            Self::User3 => 15,
            Self::User4 => 16,
        }
    }

    /// Returns the byte that selects this value on the firmware a device inquiry
    /// reported.
    ///
    /// Always `Some` for this table, which every firmware numbers the same way.
    #[must_use]
    pub const fn raw_for(self, _firmware: Version) -> Option<u8> {
        Some(self.raw())
    }

    /// Returns the name the synthesizer's display writes, as of
    /// [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub fn name(self) -> &'static str {
        self.name_for(DEFAULT_FIRMWARE).unwrap_or_default()
    }

    /// Returns the name the synthesizer's display writes on the firmware a
    /// device inquiry reported.
    ///
    /// `None` for a value that firmware does not have. The names live in the
    /// value table the parameter layer already carries; nothing here holds a
    /// second copy of them.
    #[must_use]
    pub fn name_for(self, firmware: Version) -> Option<&'static str> {
        let raw = self.raw_for(firmware)?;
        Self::TABLE.table_for(firmware).name_of(u16::from(raw))
    }
}

impl fmt::Display for ProgramCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// OSC 1 PWM Source.
///
/// Every value the synthesizer has a name for. A byte outside the table is not
/// one of these: [`PwmSource::from_raw`] answers `None` for it, and the byte stays
/// in the program either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum PwmSource {
    /// Manual.
    Manual,
    /// LFO 1.
    Lfo1,
    /// LFO 2.
    Lfo2,
    /// VCA Env.
    VcaEnv,
    /// VCF Env.
    VcfEnv,
    /// Mod Env.
    ModEnv,
}

impl PwmSource {
    /// The value table this type lists.
    pub const TABLE: TableId = TableId::PwmSource;

    /// Every value, in the order the newest firmware numbers them.
    pub const ALL: &'static [Self] = &[
        Self::Manual,
        Self::Lfo1,
        Self::Lfo2,
        Self::VcaEnv,
        Self::VcfEnv,
        Self::ModEnv,
    ];

    /// Returns the value this byte selects on [`DEFAULT_FIRMWARE`].
    ///
    /// `None` when the table does not list the byte.
    #[must_use]
    pub const fn from_raw(raw: u8) -> Option<Self> {
        Self::from_raw_for(raw, DEFAULT_FIRMWARE)
    }

    /// Returns the value this byte selects on the firmware a device inquiry
    /// reported.
    ///
    /// Every firmware numbers this table the same way, so the version makes no
    /// difference here; it is taken so that every value type reads alike.
    #[must_use]
    pub const fn from_raw_for(raw: u8, _firmware: Version) -> Option<Self> {
        match raw {
            0 => Some(Self::Manual),
            1 => Some(Self::Lfo1),
            2 => Some(Self::Lfo2),
            3 => Some(Self::VcaEnv),
            4 => Some(Self::VcfEnv),
            5 => Some(Self::ModEnv),
            _ => None,
        }
    }

    /// Returns the byte that selects this value on [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
            Self::Manual => 0,
            Self::Lfo1 => 1,
            Self::Lfo2 => 2,
            Self::VcaEnv => 3,
            Self::VcfEnv => 4,
            Self::ModEnv => 5,
        }
    }

    /// Returns the byte that selects this value on the firmware a device inquiry
    /// reported.
    ///
    /// Always `Some` for this table, which every firmware numbers the same way.
    #[must_use]
    pub const fn raw_for(self, _firmware: Version) -> Option<u8> {
        Some(self.raw())
    }

    /// Returns the name the synthesizer's display writes, as of
    /// [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub fn name(self) -> &'static str {
        self.name_for(DEFAULT_FIRMWARE).unwrap_or_default()
    }

    /// Returns the name the synthesizer's display writes on the firmware a
    /// device inquiry reported.
    ///
    /// `None` for a value that firmware does not have. The names live in the
    /// value table the parameter layer already carries; nothing here holds a
    /// second copy of them.
    #[must_use]
    pub fn name_for(self, firmware: Version) -> Option<&'static str> {
        let raw = self.raw_for(firmware)?;
        Self::TABLE.table_for(firmware).name_of(u16::from(raw))
    }
}

impl fmt::Display for PwmSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Control Sequencer Key Sync and Loop.
///
/// Every value the synthesizer has a name for. A byte outside the table is not
/// one of these: [`SequencerSync::from_raw`] answers `None` for it, and the byte stays
/// in the program either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum SequencerSync {
    /// Loop on.
    LoopOn,
    /// Key sync on.
    KeySyncOn,
    /// Loop and key sync on.
    LoopAndKeySyncOn,
}

impl SequencerSync {
    /// The value table this type lists.
    pub const TABLE: TableId = TableId::SequencerSync;

    /// Every value, in the order the newest firmware numbers them.
    pub const ALL: &'static [Self] = &[Self::LoopOn, Self::KeySyncOn, Self::LoopAndKeySyncOn];

    /// Returns the value this byte selects on [`DEFAULT_FIRMWARE`].
    ///
    /// `None` when the table does not list the byte.
    #[must_use]
    pub const fn from_raw(raw: u8) -> Option<Self> {
        Self::from_raw_for(raw, DEFAULT_FIRMWARE)
    }

    /// Returns the value this byte selects on the firmware a device inquiry
    /// reported.
    ///
    /// Every firmware numbers this table the same way, so the version makes no
    /// difference here; it is taken so that every value type reads alike.
    #[must_use]
    pub const fn from_raw_for(raw: u8, _firmware: Version) -> Option<Self> {
        match raw {
            0 => Some(Self::LoopOn),
            1 => Some(Self::KeySyncOn),
            2 => Some(Self::LoopAndKeySyncOn),
            _ => None,
        }
    }

    /// Returns the byte that selects this value on [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
            Self::LoopOn => 0,
            Self::KeySyncOn => 1,
            Self::LoopAndKeySyncOn => 2,
        }
    }

    /// Returns the byte that selects this value on the firmware a device inquiry
    /// reported.
    ///
    /// Always `Some` for this table, which every firmware numbers the same way.
    #[must_use]
    pub const fn raw_for(self, _firmware: Version) -> Option<u8> {
        Some(self.raw())
    }

    /// Returns the name the synthesizer's display writes, as of
    /// [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub fn name(self) -> &'static str {
        self.name_for(DEFAULT_FIRMWARE).unwrap_or_default()
    }

    /// Returns the name the synthesizer's display writes on the firmware a
    /// device inquiry reported.
    ///
    /// `None` for a value that firmware does not have. The names live in the
    /// value table the parameter layer already carries; nothing here holds a
    /// second copy of them.
    #[must_use]
    pub fn name_for(self, firmware: Version) -> Option<&'static str> {
        let raw = self.raw_for(firmware)?;
        Self::TABLE.table_for(firmware).name_of(u16::from(raw))
    }
}

impl fmt::Display for SequencerSync {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// OSC 2 Tone Mod Source.
///
/// Every value the synthesizer has a name for. A byte outside the table is not
/// one of these: [`ToneModSource::from_raw`] answers `None` for it, and the byte stays
/// in the program either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum ToneModSource {
    /// Manual.
    Manual,
    /// LFO 1.
    Lfo1,
    /// LFO 2.
    Lfo2,
    /// VCA Env.
    VcaEnv,
    /// VCF Env.
    VcfEnv,
    /// Mod Env.
    ModEnv,
}

impl ToneModSource {
    /// The value table this type lists.
    pub const TABLE: TableId = TableId::ToneModSource;

    /// Every value, in the order the newest firmware numbers them.
    pub const ALL: &'static [Self] = &[
        Self::Manual,
        Self::Lfo1,
        Self::Lfo2,
        Self::VcaEnv,
        Self::VcfEnv,
        Self::ModEnv,
    ];

    /// Returns the value this byte selects on [`DEFAULT_FIRMWARE`].
    ///
    /// `None` when the table does not list the byte.
    #[must_use]
    pub const fn from_raw(raw: u8) -> Option<Self> {
        Self::from_raw_for(raw, DEFAULT_FIRMWARE)
    }

    /// Returns the value this byte selects on the firmware a device inquiry
    /// reported.
    ///
    /// Every firmware numbers this table the same way, so the version makes no
    /// difference here; it is taken so that every value type reads alike.
    #[must_use]
    pub const fn from_raw_for(raw: u8, _firmware: Version) -> Option<Self> {
        match raw {
            0 => Some(Self::Manual),
            1 => Some(Self::Lfo1),
            2 => Some(Self::Lfo2),
            3 => Some(Self::VcaEnv),
            4 => Some(Self::VcfEnv),
            5 => Some(Self::ModEnv),
            _ => None,
        }
    }

    /// Returns the byte that selects this value on [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
            Self::Manual => 0,
            Self::Lfo1 => 1,
            Self::Lfo2 => 2,
            Self::VcaEnv => 3,
            Self::VcfEnv => 4,
            Self::ModEnv => 5,
        }
    }

    /// Returns the byte that selects this value on the firmware a device inquiry
    /// reported.
    ///
    /// Always `Some` for this table, which every firmware numbers the same way.
    #[must_use]
    pub const fn raw_for(self, _firmware: Version) -> Option<u8> {
        Some(self.raw())
    }

    /// Returns the name the synthesizer's display writes, as of
    /// [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub fn name(self) -> &'static str {
        self.name_for(DEFAULT_FIRMWARE).unwrap_or_default()
    }

    /// Returns the name the synthesizer's display writes on the firmware a
    /// device inquiry reported.
    ///
    /// `None` for a value that firmware does not have. The names live in the
    /// value table the parameter layer already carries; nothing here holds a
    /// second copy of them.
    #[must_use]
    pub fn name_for(self, firmware: Version) -> Option<&'static str> {
        let raw = self.raw_for(firmware)?;
        Self::TABLE.table_for(firmware).name_of(u16::from(raw))
    }
}

impl fmt::Display for ToneModSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// VCF Envelope Polarity.
///
/// Every value the synthesizer has a name for. A byte outside the table is not
/// one of these: [`VcfEnvelopePolarity::from_raw`] answers `None` for it, and the byte stays
/// in the program either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum VcfEnvelopePolarity {
    /// Negative.
    Negative,
    /// Positive.
    Positive,
}

impl VcfEnvelopePolarity {
    /// The value table this type lists.
    pub const TABLE: TableId = TableId::VcfEnvelopePolarity;

    /// Every value, in the order the newest firmware numbers them.
    pub const ALL: &'static [Self] = &[Self::Negative, Self::Positive];

    /// Returns the value this byte selects on [`DEFAULT_FIRMWARE`].
    ///
    /// `None` when the table does not list the byte.
    #[must_use]
    pub const fn from_raw(raw: u8) -> Option<Self> {
        Self::from_raw_for(raw, DEFAULT_FIRMWARE)
    }

    /// Returns the value this byte selects on the firmware a device inquiry
    /// reported.
    ///
    /// Every firmware numbers this table the same way, so the version makes no
    /// difference here; it is taken so that every value type reads alike.
    #[must_use]
    pub const fn from_raw_for(raw: u8, _firmware: Version) -> Option<Self> {
        match raw {
            0 => Some(Self::Negative),
            1 => Some(Self::Positive),
            _ => None,
        }
    }

    /// Returns the byte that selects this value on [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
            Self::Negative => 0,
            Self::Positive => 1,
        }
    }

    /// Returns the byte that selects this value on the firmware a device inquiry
    /// reported.
    ///
    /// Always `Some` for this table, which every firmware numbers the same way.
    #[must_use]
    pub const fn raw_for(self, _firmware: Version) -> Option<u8> {
        Some(self.raw())
    }

    /// Returns the name the synthesizer's display writes, as of
    /// [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub fn name(self) -> &'static str {
        self.name_for(DEFAULT_FIRMWARE).unwrap_or_default()
    }

    /// Returns the name the synthesizer's display writes on the firmware a
    /// device inquiry reported.
    ///
    /// `None` for a value that firmware does not have. The names live in the
    /// value table the parameter layer already carries; nothing here holds a
    /// second copy of them.
    #[must_use]
    pub fn name_for(self, firmware: Version) -> Option<&'static str> {
        let raw = self.raw_for(firmware)?;
        Self::TABLE.table_for(firmware).name_of(u16::from(raw))
    }
}

impl fmt::Display for VcfEnvelopePolarity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// VCF LFO Select.
///
/// Every value the synthesizer has a name for. A byte outside the table is not
/// one of these: [`VcfLfoSelect::from_raw`] answers `None` for it, and the byte stays
/// in the program either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum VcfLfoSelect {
    /// LFO 1.
    Lfo1,
    /// LFO 2.
    Lfo2,
}

impl VcfLfoSelect {
    /// The value table this type lists.
    pub const TABLE: TableId = TableId::VcfLfoSelect;

    /// Every value, in the order the newest firmware numbers them.
    pub const ALL: &'static [Self] = &[Self::Lfo1, Self::Lfo2];

    /// Returns the value this byte selects on [`DEFAULT_FIRMWARE`].
    ///
    /// `None` when the table does not list the byte.
    #[must_use]
    pub const fn from_raw(raw: u8) -> Option<Self> {
        Self::from_raw_for(raw, DEFAULT_FIRMWARE)
    }

    /// Returns the value this byte selects on the firmware a device inquiry
    /// reported.
    ///
    /// Every firmware numbers this table the same way, so the version makes no
    /// difference here; it is taken so that every value type reads alike.
    #[must_use]
    pub const fn from_raw_for(raw: u8, _firmware: Version) -> Option<Self> {
        match raw {
            0 => Some(Self::Lfo1),
            1 => Some(Self::Lfo2),
            _ => None,
        }
    }

    /// Returns the byte that selects this value on [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
            Self::Lfo1 => 0,
            Self::Lfo2 => 1,
        }
    }

    /// Returns the byte that selects this value on the firmware a device inquiry
    /// reported.
    ///
    /// Always `Some` for this table, which every firmware numbers the same way.
    #[must_use]
    pub const fn raw_for(self, _firmware: Version) -> Option<u8> {
        Some(self.raw())
    }

    /// Returns the name the synthesizer's display writes, as of
    /// [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub fn name(self) -> &'static str {
        self.name_for(DEFAULT_FIRMWARE).unwrap_or_default()
    }

    /// Returns the name the synthesizer's display writes on the firmware a
    /// device inquiry reported.
    ///
    /// `None` for a value that firmware does not have. The names live in the
    /// value table the parameter layer already carries; nothing here holds a
    /// second copy of them.
    #[must_use]
    pub fn name_for(self, firmware: Version) -> Option<&'static str> {
        let raw = self.raw_for(firmware)?;
        Self::TABLE.table_for(firmware).name_of(u16::from(raw))
    }
}

impl fmt::Display for VcfLfoSelect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// VCF Pole Mode.
///
/// Every value the synthesizer has a name for. A byte outside the table is not
/// one of these: [`VcfPoleMode::from_raw`] answers `None` for it, and the byte stays
/// in the program either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum VcfPoleMode {
    /// 4 Pole.
    FourPole,
    /// 2 Pole.
    TwoPole,
}

impl VcfPoleMode {
    /// The value table this type lists.
    pub const TABLE: TableId = TableId::VcfPoleMode;

    /// Every value, in the order the newest firmware numbers them.
    pub const ALL: &'static [Self] = &[Self::FourPole, Self::TwoPole];

    /// Returns the value this byte selects on [`DEFAULT_FIRMWARE`].
    ///
    /// `None` when the table does not list the byte.
    #[must_use]
    pub const fn from_raw(raw: u8) -> Option<Self> {
        Self::from_raw_for(raw, DEFAULT_FIRMWARE)
    }

    /// Returns the value this byte selects on the firmware a device inquiry
    /// reported.
    ///
    /// Every firmware numbers this table the same way, so the version makes no
    /// difference here; it is taken so that every value type reads alike.
    #[must_use]
    pub const fn from_raw_for(raw: u8, _firmware: Version) -> Option<Self> {
        match raw {
            0 => Some(Self::FourPole),
            1 => Some(Self::TwoPole),
            _ => None,
        }
    }

    /// Returns the byte that selects this value on [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
            Self::FourPole => 0,
            Self::TwoPole => 1,
        }
    }

    /// Returns the byte that selects this value on the firmware a device inquiry
    /// reported.
    ///
    /// Always `Some` for this table, which every firmware numbers the same way.
    #[must_use]
    pub const fn raw_for(self, _firmware: Version) -> Option<u8> {
        Some(self.raw())
    }

    /// Returns the name the synthesizer's display writes, as of
    /// [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub fn name(self) -> &'static str {
        self.name_for(DEFAULT_FIRMWARE).unwrap_or_default()
    }

    /// Returns the name the synthesizer's display writes on the firmware a
    /// device inquiry reported.
    ///
    /// `None` for a value that firmware does not have. The names live in the
    /// value table the parameter layer already carries; nothing here holds a
    /// second copy of them.
    #[must_use]
    pub fn name_for(self, firmware: Version) -> Option<&'static str> {
        let raw = self.raw_for(firmware)?;
        Self::TABLE.table_for(firmware).name_of(u16::from(raw))
    }
}

impl fmt::Display for VcfPoleMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Voice Priority Mode.
///
/// Every value the synthesizer has a name for. A byte outside the table is not
/// one of these: [`VoicePriority::from_raw`] answers `None` for it, and the byte stays
/// in the program either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum VoicePriority {
    /// Lowest.
    Lowest,
    /// Highest.
    Highest,
    /// Last.
    Last,
}

impl VoicePriority {
    /// The value table this type lists.
    pub const TABLE: TableId = TableId::VoicePriority;

    /// Every value, in the order the newest firmware numbers them.
    pub const ALL: &'static [Self] = &[Self::Lowest, Self::Highest, Self::Last];

    /// Returns the value this byte selects on [`DEFAULT_FIRMWARE`].
    ///
    /// `None` when the table does not list the byte.
    #[must_use]
    pub const fn from_raw(raw: u8) -> Option<Self> {
        Self::from_raw_for(raw, DEFAULT_FIRMWARE)
    }

    /// Returns the value this byte selects on the firmware a device inquiry
    /// reported.
    ///
    /// Every firmware numbers this table the same way, so the version makes no
    /// difference here; it is taken so that every value type reads alike.
    #[must_use]
    pub const fn from_raw_for(raw: u8, _firmware: Version) -> Option<Self> {
        match raw {
            0 => Some(Self::Lowest),
            1 => Some(Self::Highest),
            2 => Some(Self::Last),
            _ => None,
        }
    }

    /// Returns the byte that selects this value on [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub const fn raw(self) -> u8 {
        match self {
            Self::Lowest => 0,
            Self::Highest => 1,
            Self::Last => 2,
        }
    }

    /// Returns the byte that selects this value on the firmware a device inquiry
    /// reported.
    ///
    /// Always `Some` for this table, which every firmware numbers the same way.
    #[must_use]
    pub const fn raw_for(self, _firmware: Version) -> Option<u8> {
        Some(self.raw())
    }

    /// Returns the name the synthesizer's display writes, as of
    /// [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub fn name(self) -> &'static str {
        self.name_for(DEFAULT_FIRMWARE).unwrap_or_default()
    }

    /// Returns the name the synthesizer's display writes on the firmware a
    /// device inquiry reported.
    ///
    /// `None` for a value that firmware does not have. The names live in the
    /// value table the parameter layer already carries; nothing here holds a
    /// second copy of them.
    #[must_use]
    pub fn name_for(self, firmware: Version) -> Option<&'static str> {
        let raw = self.raw_for(firmware)?;
        Self::TABLE.table_for(firmware).name_of(u16::from(raw))
    }
}

impl fmt::Display for VoicePriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// The parameters, one pair of accessors each.
///
/// A getter reads the byte the program holds and a setter writes it. A setter
/// that takes a raw byte clamps it to what the parameter accepts, since a `u8`
/// reaches values a parameter does not; [`Program::set`] is the checked way in.
///
/// The parameters that spell the program's name are not here. They are one
/// string, and [`Program::name`] is how a string is read.
impl Program {
    /// LFO 1 Rate.
    ///
    /// Offset 0, 0 to 255.
    #[must_use]
    pub fn lfo1_rate(&self) -> u8 {
        self.get(ParamId::Lfo1Rate)
    }

    /// Sets LFO 1 Rate, clamped to 0 to 255.
    pub fn set_lfo1_rate(&mut self, value: u8) {
        self.set_clamped(ParamId::Lfo1Rate, value);
    }

    /// LFO 1 Delay / Fade.
    ///
    /// Offset 1, 0 to 255.
    #[must_use]
    pub fn lfo1_delay_fade(&self) -> u8 {
        self.get(ParamId::Lfo1DelayFade)
    }

    /// Sets LFO 1 Delay / Fade, clamped to 0 to 255.
    pub fn set_lfo1_delay_fade(&mut self, value: u8) {
        self.set_clamped(ParamId::Lfo1DelayFade, value);
    }

    /// LFO 1 Shape.
    ///
    /// Offset 2. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn lfo1_shape(&self) -> Option<LfoShape> {
        LfoShape::from_raw(self.get(ParamId::Lfo1Shape))
    }

    /// Sets LFO 1 Shape.
    pub fn set_lfo1_shape(&mut self, value: LfoShape) {
        self.set_clamped(ParamId::Lfo1Shape, value.raw());
    }

    /// LFO 1 Key Sync.
    ///
    /// Offset 3. Off is zero and on is one.
    #[must_use]
    pub fn lfo1_key_sync(&self) -> bool {
        self.get(ParamId::Lfo1KeySync) != 0
    }

    /// Sets LFO 1 Key Sync.
    pub fn set_lfo1_key_sync(&mut self, value: bool) {
        self.set_clamped(ParamId::Lfo1KeySync, u8::from(value));
    }

    /// LFO 1 Arp Sync.
    ///
    /// Offset 4. Off is zero and on is one.
    #[must_use]
    pub fn lfo1_arp_sync(&self) -> bool {
        self.get(ParamId::Lfo1ArpSync) != 0
    }

    /// Sets LFO 1 Arp Sync.
    pub fn set_lfo1_arp_sync(&mut self, value: bool) {
        self.set_clamped(ParamId::Lfo1ArpSync, u8::from(value));
    }

    /// LFO 1 Mono Mode.
    ///
    /// Offset 5, 0 to 255.
    #[must_use]
    pub fn lfo1_mono_mode(&self) -> u8 {
        self.get(ParamId::Lfo1MonoMode)
    }

    /// Sets LFO 1 Mono Mode, clamped to 0 to 255.
    pub fn set_lfo1_mono_mode(&mut self, value: u8) {
        self.set_clamped(ParamId::Lfo1MonoMode, value);
    }

    /// LFO 1 Slew Rate.
    ///
    /// Offset 6, 0 to 255.
    #[must_use]
    pub fn lfo1_slew_rate(&self) -> u8 {
        self.get(ParamId::Lfo1SlewRate)
    }

    /// Sets LFO 1 Slew Rate, clamped to 0 to 255.
    pub fn set_lfo1_slew_rate(&mut self, value: u8) {
        self.set_clamped(ParamId::Lfo1SlewRate, value);
    }

    /// LFO 2 Rate.
    ///
    /// Offset 7, 0 to 255.
    #[must_use]
    pub fn lfo2_rate(&self) -> u8 {
        self.get(ParamId::Lfo2Rate)
    }

    /// Sets LFO 2 Rate, clamped to 0 to 255.
    pub fn set_lfo2_rate(&mut self, value: u8) {
        self.set_clamped(ParamId::Lfo2Rate, value);
    }

    /// LFO 2 Delay / Fade.
    ///
    /// Offset 8, 0 to 255.
    #[must_use]
    pub fn lfo2_delay_fade(&self) -> u8 {
        self.get(ParamId::Lfo2DelayFade)
    }

    /// Sets LFO 2 Delay / Fade, clamped to 0 to 255.
    pub fn set_lfo2_delay_fade(&mut self, value: u8) {
        self.set_clamped(ParamId::Lfo2DelayFade, value);
    }

    /// LFO 2 Shape.
    ///
    /// Offset 9. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn lfo2_shape(&self) -> Option<LfoShape> {
        LfoShape::from_raw(self.get(ParamId::Lfo2Shape))
    }

    /// Sets LFO 2 Shape.
    pub fn set_lfo2_shape(&mut self, value: LfoShape) {
        self.set_clamped(ParamId::Lfo2Shape, value.raw());
    }

    /// LFO 2 Key Sync.
    ///
    /// Offset 10. Off is zero and on is one.
    #[must_use]
    pub fn lfo2_key_sync(&self) -> bool {
        self.get(ParamId::Lfo2KeySync) != 0
    }

    /// Sets LFO 2 Key Sync.
    pub fn set_lfo2_key_sync(&mut self, value: bool) {
        self.set_clamped(ParamId::Lfo2KeySync, u8::from(value));
    }

    /// LFO 2 Arp Sync.
    ///
    /// Offset 11. Off is zero and on is one.
    #[must_use]
    pub fn lfo2_arp_sync(&self) -> bool {
        self.get(ParamId::Lfo2ArpSync) != 0
    }

    /// Sets LFO 2 Arp Sync.
    pub fn set_lfo2_arp_sync(&mut self, value: bool) {
        self.set_clamped(ParamId::Lfo2ArpSync, u8::from(value));
    }

    /// LFO 2 Mono Mode.
    ///
    /// Offset 12, 0 to 255.
    #[must_use]
    pub fn lfo2_mono_mode(&self) -> u8 {
        self.get(ParamId::Lfo2MonoMode)
    }

    /// Sets LFO 2 Mono Mode, clamped to 0 to 255.
    pub fn set_lfo2_mono_mode(&mut self, value: u8) {
        self.set_clamped(ParamId::Lfo2MonoMode, value);
    }

    /// LFO 2 Slew Rate.
    ///
    /// Offset 13, 0 to 255.
    #[must_use]
    pub fn lfo2_slew_rate(&self) -> u8 {
        self.get(ParamId::Lfo2SlewRate)
    }

    /// Sets LFO 2 Slew Rate, clamped to 0 to 255.
    pub fn set_lfo2_slew_rate(&mut self, value: u8) {
        self.set_clamped(ParamId::Lfo2SlewRate, value);
    }

    /// OSC 1 Range.
    ///
    /// Offset 14. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn osc1_range(&self) -> Option<OscRange> {
        OscRange::from_raw(self.get(ParamId::Osc1Range))
    }

    /// Sets OSC 1 Range.
    pub fn set_osc1_range(&mut self, value: OscRange) {
        self.set_clamped(ParamId::Osc1Range, value.raw());
    }

    /// OSC 2 Range.
    ///
    /// Offset 15. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn osc2_range(&self) -> Option<OscRange> {
        OscRange::from_raw(self.get(ParamId::Osc2Range))
    }

    /// Sets OSC 2 Range.
    pub fn set_osc2_range(&mut self, value: OscRange) {
        self.set_clamped(ParamId::Osc2Range, value.raw());
    }

    /// OSC 1 PWM Source.
    ///
    /// Offset 16. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn osc1_pwm_source(&self) -> Option<PwmSource> {
        PwmSource::from_raw(self.get(ParamId::Osc1PwmSource))
    }

    /// Sets OSC 1 PWM Source.
    pub fn set_osc1_pwm_source(&mut self, value: PwmSource) {
        self.set_clamped(ParamId::Osc1PwmSource, value.raw());
    }

    /// OSC 2 Tone Mod Source.
    ///
    /// Offset 17. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn osc2_tone_mod_source(&self) -> Option<ToneModSource> {
        ToneModSource::from_raw(self.get(ParamId::Osc2ToneModSource))
    }

    /// Sets OSC 2 Tone Mod Source.
    pub fn set_osc2_tone_mod_source(&mut self, value: ToneModSource) {
        self.set_clamped(ParamId::Osc2ToneModSource, value.raw());
    }

    /// OSC 1 Pulse Enable.
    ///
    /// Offset 18. Off is zero and on is one.
    #[must_use]
    pub fn osc1_pulse_enable(&self) -> bool {
        self.get(ParamId::Osc1PulseEnable) != 0
    }

    /// Sets OSC 1 Pulse Enable.
    pub fn set_osc1_pulse_enable(&mut self, value: bool) {
        self.set_clamped(ParamId::Osc1PulseEnable, u8::from(value));
    }

    /// OSC 1 Saw Enable.
    ///
    /// Offset 19. Off is zero and on is one.
    #[must_use]
    pub fn osc1_saw_enable(&self) -> bool {
        self.get(ParamId::Osc1SawEnable) != 0
    }

    /// Sets OSC 1 Saw Enable.
    pub fn set_osc1_saw_enable(&mut self, value: bool) {
        self.set_clamped(ParamId::Osc1SawEnable, u8::from(value));
    }

    /// OSC Sync Enable.
    ///
    /// Offset 20. Off is zero and on is one.
    #[must_use]
    pub fn osc_sync_enable(&self) -> bool {
        self.get(ParamId::OscSyncEnable) != 0
    }

    /// Sets OSC Sync Enable.
    pub fn set_osc_sync_enable(&mut self, value: bool) {
        self.set_clamped(ParamId::OscSyncEnable, u8::from(value));
    }

    /// OSC 1 Pitch Mod Depth.
    ///
    /// Offset 21, 0 to 255.
    #[must_use]
    pub fn osc1_pitch_mod_depth(&self) -> u8 {
        self.get(ParamId::Osc1PitchModDepth)
    }

    /// Sets OSC 1 Pitch Mod Depth, clamped to 0 to 255.
    pub fn set_osc1_pitch_mod_depth(&mut self, value: u8) {
        self.set_clamped(ParamId::Osc1PitchModDepth, value);
    }

    /// OSC 1 Pitch Mod Select.
    ///
    /// Offset 22. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn osc1_pitch_mod_select(&self) -> Option<PitchModSource> {
        PitchModSource::from_raw(self.get(ParamId::Osc1PitchModSelect))
    }

    /// Sets OSC 1 Pitch Mod Select.
    pub fn set_osc1_pitch_mod_select(&mut self, value: PitchModSource) {
        self.set_clamped(ParamId::Osc1PitchModSelect, value.raw());
    }

    /// OSC 1 Aftertouch > Pitch Mod Depth.
    ///
    /// Offset 23, 0 to 255.
    #[must_use]
    pub fn osc1_aftertouch_to_pitch_mod_depth(&self) -> u8 {
        self.get(ParamId::Osc1AftertouchToPitchModDepth)
    }

    /// Sets OSC 1 Aftertouch > Pitch Mod Depth, clamped to 0 to 255.
    pub fn set_osc1_aftertouch_to_pitch_mod_depth(&mut self, value: u8) {
        self.set_clamped(ParamId::Osc1AftertouchToPitchModDepth, value);
    }

    /// OSC 1 Mod Wheel > Pitch Mod Depth.
    ///
    /// Offset 24, 0 to 255.
    #[must_use]
    pub fn osc1_mod_wheel_to_pitch_mod_depth(&self) -> u8 {
        self.get(ParamId::Osc1ModWheelToPitchModDepth)
    }

    /// Sets OSC 1 Mod Wheel > Pitch Mod Depth, clamped to 0 to 255.
    pub fn set_osc1_mod_wheel_to_pitch_mod_depth(&mut self, value: u8) {
        self.set_clamped(ParamId::Osc1ModWheelToPitchModDepth, value);
    }

    /// OSC 1 PWM Depth.
    ///
    /// Offset 25, 0 to 255.
    #[must_use]
    pub fn osc1_pwm_depth(&self) -> u8 {
        self.get(ParamId::Osc1PwmDepth)
    }

    /// Sets OSC 1 PWM Depth, clamped to 0 to 255.
    pub fn set_osc1_pwm_depth(&mut self, value: u8) {
        self.set_clamped(ParamId::Osc1PwmDepth, value);
    }

    /// OSC 2 Level.
    ///
    /// Offset 26, 0 to 255.
    #[must_use]
    pub fn osc2_level(&self) -> u8 {
        self.get(ParamId::Osc2Level)
    }

    /// Sets OSC 2 Level, clamped to 0 to 255.
    pub fn set_osc2_level(&mut self, value: u8) {
        self.set_clamped(ParamId::Osc2Level, value);
    }

    /// OSC 2 Pitch.
    ///
    /// Offset 27, 0 to 255.
    #[must_use]
    pub fn osc2_pitch(&self) -> u8 {
        self.get(ParamId::Osc2Pitch)
    }

    /// Sets OSC 2 Pitch, clamped to 0 to 255.
    pub fn set_osc2_pitch(&mut self, value: u8) {
        self.set_clamped(ParamId::Osc2Pitch, value);
    }

    /// OSC 2 Tone Mod Depth.
    ///
    /// Offset 28, 0 to 255.
    #[must_use]
    pub fn osc2_tone_mod_depth(&self) -> u8 {
        self.get(ParamId::Osc2ToneModDepth)
    }

    /// Sets OSC 2 Tone Mod Depth, clamped to 0 to 255.
    pub fn set_osc2_tone_mod_depth(&mut self, value: u8) {
        self.set_clamped(ParamId::Osc2ToneModDepth, value);
    }

    /// OSC 2 Pitch Mod Depth.
    ///
    /// Offset 29, 0 to 255.
    #[must_use]
    pub fn osc2_pitch_mod_depth(&self) -> u8 {
        self.get(ParamId::Osc2PitchModDepth)
    }

    /// Sets OSC 2 Pitch Mod Depth, clamped to 0 to 255.
    pub fn set_osc2_pitch_mod_depth(&mut self, value: u8) {
        self.set_clamped(ParamId::Osc2PitchModDepth, value);
    }

    /// OSC 2 Aftertouch > Pitch Mod Depth.
    ///
    /// Offset 30, 0 to 255.
    #[must_use]
    pub fn osc2_aftertouch_to_pitch_mod_depth(&self) -> u8 {
        self.get(ParamId::Osc2AftertouchToPitchModDepth)
    }

    /// Sets OSC 2 Aftertouch > Pitch Mod Depth, clamped to 0 to 255.
    pub fn set_osc2_aftertouch_to_pitch_mod_depth(&mut self, value: u8) {
        self.set_clamped(ParamId::Osc2AftertouchToPitchModDepth, value);
    }

    /// OSC 2 Mod Wheel > Pitch Mod Depth.
    ///
    /// Offset 31, 0 to 255.
    #[must_use]
    pub fn osc2_mod_wheel_to_pitch_mod_depth(&self) -> u8 {
        self.get(ParamId::Osc2ModWheelToPitchModDepth)
    }

    /// Sets OSC 2 Mod Wheel > Pitch Mod Depth, clamped to 0 to 255.
    pub fn set_osc2_mod_wheel_to_pitch_mod_depth(&mut self, value: u8) {
        self.set_clamped(ParamId::Osc2ModWheelToPitchModDepth, value);
    }

    /// OSC 2 Pitch Mod Select.
    ///
    /// Offset 32. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn osc2_pitch_mod_select(&self) -> Option<PitchModSource> {
        PitchModSource::from_raw(self.get(ParamId::Osc2PitchModSelect))
    }

    /// Sets OSC 2 Pitch Mod Select.
    pub fn set_osc2_pitch_mod_select(&mut self, value: PitchModSource) {
        self.set_clamped(ParamId::Osc2PitchModSelect, value.raw());
    }

    /// Noise Level.
    ///
    /// Offset 33, 0 to 255.
    #[must_use]
    pub fn noise_level(&self) -> u8 {
        self.get(ParamId::NoiseLevel)
    }

    /// Sets Noise Level, clamped to 0 to 255.
    pub fn set_noise_level(&mut self, value: u8) {
        self.set_clamped(ParamId::NoiseLevel, value);
    }

    /// Portamento time.
    ///
    /// Offset 34, 0 to 255.
    #[must_use]
    pub fn portamento_time(&self) -> u8 {
        self.get(ParamId::PortamentoTime)
    }

    /// Sets Portamento time, clamped to 0 to 255.
    pub fn set_portamento_time(&mut self, value: u8) {
        self.set_clamped(ParamId::PortamentoTime, value);
    }

    /// Portamento mode.
    ///
    /// Offset 35. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn portamento_mode(&self) -> Option<PortamentoMode> {
        PortamentoMode::from_raw(self.get(ParamId::PortamentoMode))
    }

    /// Sets Portamento mode.
    pub fn set_portamento_mode(&mut self, value: PortamentoMode) {
        self.set_clamped(ParamId::PortamentoMode, value.raw());
    }

    /// Pitch Bend Up Depth.
    ///
    /// Offset 36, 0 to 48.
    #[must_use]
    pub fn pitch_bend_up_depth(&self) -> u8 {
        self.get(ParamId::PitchBendUpDepth)
    }

    /// Sets Pitch Bend Up Depth, clamped to 0 to 48.
    pub fn set_pitch_bend_up_depth(&mut self, value: u8) {
        self.set_clamped(ParamId::PitchBendUpDepth, value);
    }

    /// Pitch Bend Down Depth.
    ///
    /// Offset 37, 0 to 48.
    #[must_use]
    pub fn pitch_bend_down_depth(&self) -> u8 {
        self.get(ParamId::PitchBendDownDepth)
    }

    /// Sets Pitch Bend Down Depth, clamped to 0 to 48.
    pub fn set_pitch_bend_down_depth(&mut self, value: u8) {
        self.set_clamped(ParamId::PitchBendDownDepth, value);
    }

    /// OSC 1 Pitch Mod Mode.
    ///
    /// Offset 38. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn osc1_pitch_mod_mode(&self) -> Option<Osc1PitchModMode> {
        Osc1PitchModMode::from_raw(self.get(ParamId::Osc1PitchModMode))
    }

    /// Sets OSC 1 Pitch Mod Mode.
    pub fn set_osc1_pitch_mod_mode(&mut self, value: Osc1PitchModMode) {
        self.set_clamped(ParamId::Osc1PitchModMode, value.raw());
    }

    /// VCF Frequency.
    ///
    /// Offset 39, 0 to 255.
    #[must_use]
    pub fn vcf_frequency(&self) -> u8 {
        self.get(ParamId::VcfFrequency)
    }

    /// Sets VCF Frequency, clamped to 0 to 255.
    pub fn set_vcf_frequency(&mut self, value: u8) {
        self.set_clamped(ParamId::VcfFrequency, value);
    }

    /// VCF `HighPass` Frequency.
    ///
    /// Offset 40, 0 to 255.
    #[must_use]
    pub fn vcf_high_pass_frequency(&self) -> u8 {
        self.get(ParamId::VcfHighPassFrequency)
    }

    /// Sets VCF `HighPass` Frequency, clamped to 0 to 255.
    pub fn set_vcf_high_pass_frequency(&mut self, value: u8) {
        self.set_clamped(ParamId::VcfHighPassFrequency, value);
    }

    /// VCF Resonance.
    ///
    /// Offset 41, 0 to 255.
    #[must_use]
    pub fn vcf_resonance(&self) -> u8 {
        self.get(ParamId::VcfResonance)
    }

    /// Sets VCF Resonance, clamped to 0 to 255.
    pub fn set_vcf_resonance(&mut self, value: u8) {
        self.set_clamped(ParamId::VcfResonance, value);
    }

    /// VCF Envelope Depth.
    ///
    /// Offset 42, 0 to 255.
    #[must_use]
    pub fn vcf_envelope_depth(&self) -> u8 {
        self.get(ParamId::VcfEnvelopeDepth)
    }

    /// Sets VCF Envelope Depth, clamped to 0 to 255.
    pub fn set_vcf_envelope_depth(&mut self, value: u8) {
        self.set_clamped(ParamId::VcfEnvelopeDepth, value);
    }

    /// VCF Envelope Velocity Sensitivity.
    ///
    /// Offset 43, 0 to 255.
    #[must_use]
    pub fn vcf_envelope_velocity_sensitivity(&self) -> u8 {
        self.get(ParamId::VcfEnvelopeVelocitySensitivity)
    }

    /// Sets VCF Envelope Velocity Sensitivity, clamped to 0 to 255.
    pub fn set_vcf_envelope_velocity_sensitivity(&mut self, value: u8) {
        self.set_clamped(ParamId::VcfEnvelopeVelocitySensitivity, value);
    }

    /// VCF Pitch Bend to Freq Depth.
    ///
    /// Offset 44, 0 to 255.
    #[must_use]
    pub fn vcf_pitch_bend_to_freq_depth(&self) -> u8 {
        self.get(ParamId::VcfPitchBendToFreqDepth)
    }

    /// Sets VCF Pitch Bend to Freq Depth, clamped to 0 to 255.
    pub fn set_vcf_pitch_bend_to_freq_depth(&mut self, value: u8) {
        self.set_clamped(ParamId::VcfPitchBendToFreqDepth, value);
    }

    /// VCF LFO Depth.
    ///
    /// Offset 45, 0 to 255.
    #[must_use]
    pub fn vcf_lfo_depth(&self) -> u8 {
        self.get(ParamId::VcfLfoDepth)
    }

    /// Sets VCF LFO Depth, clamped to 0 to 255.
    pub fn set_vcf_lfo_depth(&mut self, value: u8) {
        self.set_clamped(ParamId::VcfLfoDepth, value);
    }

    /// VCF LFO Select.
    ///
    /// Offset 46. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn vcf_lfo_select(&self) -> Option<VcfLfoSelect> {
        VcfLfoSelect::from_raw(self.get(ParamId::VcfLfoSelect))
    }

    /// Sets VCF LFO Select.
    pub fn set_vcf_lfo_select(&mut self, value: VcfLfoSelect) {
        self.set_clamped(ParamId::VcfLfoSelect, value.raw());
    }

    /// VCF Aftertouch > LFO Depth.
    ///
    /// Offset 47, 0 to 255.
    #[must_use]
    pub fn vcf_aftertouch_to_lfo_depth(&self) -> u8 {
        self.get(ParamId::VcfAftertouchToLfoDepth)
    }

    /// Sets VCF Aftertouch > LFO Depth, clamped to 0 to 255.
    pub fn set_vcf_aftertouch_to_lfo_depth(&mut self, value: u8) {
        self.set_clamped(ParamId::VcfAftertouchToLfoDepth, value);
    }

    /// VCF Mod Wheel > LFO Depth.
    ///
    /// Offset 48, 0 to 255.
    #[must_use]
    pub fn vcf_mod_wheel_to_lfo_depth(&self) -> u8 {
        self.get(ParamId::VcfModWheelToLfoDepth)
    }

    /// Sets VCF Mod Wheel > LFO Depth, clamped to 0 to 255.
    pub fn set_vcf_mod_wheel_to_lfo_depth(&mut self, value: u8) {
        self.set_clamped(ParamId::VcfModWheelToLfoDepth, value);
    }

    /// VCF Keyboard Tracking.
    ///
    /// Offset 49, 0 to 255.
    #[must_use]
    pub fn vcf_keyboard_tracking(&self) -> u8 {
        self.get(ParamId::VcfKeyboardTracking)
    }

    /// Sets VCF Keyboard Tracking, clamped to 0 to 255.
    pub fn set_vcf_keyboard_tracking(&mut self, value: u8) {
        self.set_clamped(ParamId::VcfKeyboardTracking, value);
    }

    /// VCF Envelope Polarity.
    ///
    /// Offset 50. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn vcf_envelope_polarity(&self) -> Option<VcfEnvelopePolarity> {
        VcfEnvelopePolarity::from_raw(self.get(ParamId::VcfEnvelopePolarity))
    }

    /// Sets VCF Envelope Polarity.
    pub fn set_vcf_envelope_polarity(&mut self, value: VcfEnvelopePolarity) {
        self.set_clamped(ParamId::VcfEnvelopePolarity, value.raw());
    }

    /// VCF 2 Pole Mode.
    ///
    /// Offset 51. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn vcf2_pole_mode(&self) -> Option<VcfPoleMode> {
        VcfPoleMode::from_raw(self.get(ParamId::Vcf2PoleMode))
    }

    /// Sets VCF 2 Pole Mode.
    pub fn set_vcf2_pole_mode(&mut self, value: VcfPoleMode) {
        self.set_clamped(ParamId::Vcf2PoleMode, value.raw());
    }

    /// VCF Bass Boost.
    ///
    /// Offset 52. Off is zero and on is one.
    #[must_use]
    pub fn vcf_bass_boost(&self) -> bool {
        self.get(ParamId::VcfBassBoost) != 0
    }

    /// Sets VCF Bass Boost.
    pub fn set_vcf_bass_boost(&mut self, value: bool) {
        self.set_clamped(ParamId::VcfBassBoost, u8::from(value));
    }

    /// VCA Envelope Attack Time.
    ///
    /// Offset 53, 0 to 255.
    #[must_use]
    pub fn vca_envelope_attack_time(&self) -> u8 {
        self.get(ParamId::VcaEnvelopeAttackTime)
    }

    /// Sets VCA Envelope Attack Time, clamped to 0 to 255.
    pub fn set_vca_envelope_attack_time(&mut self, value: u8) {
        self.set_clamped(ParamId::VcaEnvelopeAttackTime, value);
    }

    /// VCA Envelope Decay Time.
    ///
    /// Offset 54, 0 to 255.
    #[must_use]
    pub fn vca_envelope_decay_time(&self) -> u8 {
        self.get(ParamId::VcaEnvelopeDecayTime)
    }

    /// Sets VCA Envelope Decay Time, clamped to 0 to 255.
    pub fn set_vca_envelope_decay_time(&mut self, value: u8) {
        self.set_clamped(ParamId::VcaEnvelopeDecayTime, value);
    }

    /// VCA Envelope Sustain Level.
    ///
    /// Offset 55, 0 to 255.
    #[must_use]
    pub fn vca_envelope_sustain_level(&self) -> u8 {
        self.get(ParamId::VcaEnvelopeSustainLevel)
    }

    /// Sets VCA Envelope Sustain Level, clamped to 0 to 255.
    pub fn set_vca_envelope_sustain_level(&mut self, value: u8) {
        self.set_clamped(ParamId::VcaEnvelopeSustainLevel, value);
    }

    /// VCA Envelope Release Time.
    ///
    /// Offset 56, 0 to 255.
    #[must_use]
    pub fn vca_envelope_release_time(&self) -> u8 {
        self.get(ParamId::VcaEnvelopeReleaseTime)
    }

    /// Sets VCA Envelope Release Time, clamped to 0 to 255.
    pub fn set_vca_envelope_release_time(&mut self, value: u8) {
        self.set_clamped(ParamId::VcaEnvelopeReleaseTime, value);
    }

    /// VCA Envelope Trigger Mode.
    ///
    /// Offset 57. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn vca_envelope_trigger_mode(&self) -> Option<EnvelopeTrigger> {
        EnvelopeTrigger::from_raw(self.get(ParamId::VcaEnvelopeTriggerMode))
    }

    /// Sets VCA Envelope Trigger Mode.
    pub fn set_vca_envelope_trigger_mode(&mut self, value: EnvelopeTrigger) {
        self.set_clamped(ParamId::VcaEnvelopeTriggerMode, value.raw());
    }

    /// VCA Envelope Attack Curve.
    ///
    /// Offset 58, 0 to 255.
    #[must_use]
    pub fn vca_envelope_attack_curve(&self) -> u8 {
        self.get(ParamId::VcaEnvelopeAttackCurve)
    }

    /// Sets VCA Envelope Attack Curve, clamped to 0 to 255.
    pub fn set_vca_envelope_attack_curve(&mut self, value: u8) {
        self.set_clamped(ParamId::VcaEnvelopeAttackCurve, value);
    }

    /// VCA Envelope Decay Curve.
    ///
    /// Offset 59, 0 to 255.
    #[must_use]
    pub fn vca_envelope_decay_curve(&self) -> u8 {
        self.get(ParamId::VcaEnvelopeDecayCurve)
    }

    /// Sets VCA Envelope Decay Curve, clamped to 0 to 255.
    pub fn set_vca_envelope_decay_curve(&mut self, value: u8) {
        self.set_clamped(ParamId::VcaEnvelopeDecayCurve, value);
    }

    /// VCA Envelope Sustain Curve.
    ///
    /// Offset 60, 0 to 255.
    #[must_use]
    pub fn vca_envelope_sustain_curve(&self) -> u8 {
        self.get(ParamId::VcaEnvelopeSustainCurve)
    }

    /// Sets VCA Envelope Sustain Curve, clamped to 0 to 255.
    pub fn set_vca_envelope_sustain_curve(&mut self, value: u8) {
        self.set_clamped(ParamId::VcaEnvelopeSustainCurve, value);
    }

    /// VCA Envelope Release Curve.
    ///
    /// Offset 61, 0 to 255.
    #[must_use]
    pub fn vca_envelope_release_curve(&self) -> u8 {
        self.get(ParamId::VcaEnvelopeReleaseCurve)
    }

    /// Sets VCA Envelope Release Curve, clamped to 0 to 255.
    pub fn set_vca_envelope_release_curve(&mut self, value: u8) {
        self.set_clamped(ParamId::VcaEnvelopeReleaseCurve, value);
    }

    /// VCF Envelope Attack Time.
    ///
    /// Offset 62, 0 to 255.
    #[must_use]
    pub fn vcf_envelope_attack_time(&self) -> u8 {
        self.get(ParamId::VcfEnvelopeAttackTime)
    }

    /// Sets VCF Envelope Attack Time, clamped to 0 to 255.
    pub fn set_vcf_envelope_attack_time(&mut self, value: u8) {
        self.set_clamped(ParamId::VcfEnvelopeAttackTime, value);
    }

    /// VCF Envelope Decay Time.
    ///
    /// Offset 63, 0 to 255.
    #[must_use]
    pub fn vcf_envelope_decay_time(&self) -> u8 {
        self.get(ParamId::VcfEnvelopeDecayTime)
    }

    /// Sets VCF Envelope Decay Time, clamped to 0 to 255.
    pub fn set_vcf_envelope_decay_time(&mut self, value: u8) {
        self.set_clamped(ParamId::VcfEnvelopeDecayTime, value);
    }

    /// VCF Envelope Sustain Level.
    ///
    /// Offset 64, 0 to 255.
    #[must_use]
    pub fn vcf_envelope_sustain_level(&self) -> u8 {
        self.get(ParamId::VcfEnvelopeSustainLevel)
    }

    /// Sets VCF Envelope Sustain Level, clamped to 0 to 255.
    pub fn set_vcf_envelope_sustain_level(&mut self, value: u8) {
        self.set_clamped(ParamId::VcfEnvelopeSustainLevel, value);
    }

    /// VCF Envelope Release Time.
    ///
    /// Offset 65, 0 to 255.
    #[must_use]
    pub fn vcf_envelope_release_time(&self) -> u8 {
        self.get(ParamId::VcfEnvelopeReleaseTime)
    }

    /// Sets VCF Envelope Release Time, clamped to 0 to 255.
    pub fn set_vcf_envelope_release_time(&mut self, value: u8) {
        self.set_clamped(ParamId::VcfEnvelopeReleaseTime, value);
    }

    /// VCF Envelope Trigger Mode.
    ///
    /// Offset 66. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn vcf_envelope_trigger_mode(&self) -> Option<EnvelopeTrigger> {
        EnvelopeTrigger::from_raw(self.get(ParamId::VcfEnvelopeTriggerMode))
    }

    /// Sets VCF Envelope Trigger Mode.
    pub fn set_vcf_envelope_trigger_mode(&mut self, value: EnvelopeTrigger) {
        self.set_clamped(ParamId::VcfEnvelopeTriggerMode, value.raw());
    }

    /// VCF Envelope Attack Curve.
    ///
    /// Offset 67, 0 to 255.
    #[must_use]
    pub fn vcf_envelope_attack_curve(&self) -> u8 {
        self.get(ParamId::VcfEnvelopeAttackCurve)
    }

    /// Sets VCF Envelope Attack Curve, clamped to 0 to 255.
    pub fn set_vcf_envelope_attack_curve(&mut self, value: u8) {
        self.set_clamped(ParamId::VcfEnvelopeAttackCurve, value);
    }

    /// VCF Envelope Decay Curve.
    ///
    /// Offset 68, 0 to 255.
    #[must_use]
    pub fn vcf_envelope_decay_curve(&self) -> u8 {
        self.get(ParamId::VcfEnvelopeDecayCurve)
    }

    /// Sets VCF Envelope Decay Curve, clamped to 0 to 255.
    pub fn set_vcf_envelope_decay_curve(&mut self, value: u8) {
        self.set_clamped(ParamId::VcfEnvelopeDecayCurve, value);
    }

    /// VCF Envelope Sustain Curve.
    ///
    /// Offset 69, 0 to 255.
    #[must_use]
    pub fn vcf_envelope_sustain_curve(&self) -> u8 {
        self.get(ParamId::VcfEnvelopeSustainCurve)
    }

    /// Sets VCF Envelope Sustain Curve, clamped to 0 to 255.
    pub fn set_vcf_envelope_sustain_curve(&mut self, value: u8) {
        self.set_clamped(ParamId::VcfEnvelopeSustainCurve, value);
    }

    /// VCF Envelope Release Curve.
    ///
    /// Offset 70, 0 to 255.
    #[must_use]
    pub fn vcf_envelope_release_curve(&self) -> u8 {
        self.get(ParamId::VcfEnvelopeReleaseCurve)
    }

    /// Sets VCF Envelope Release Curve, clamped to 0 to 255.
    pub fn set_vcf_envelope_release_curve(&mut self, value: u8) {
        self.set_clamped(ParamId::VcfEnvelopeReleaseCurve, value);
    }

    /// Mod Envelope Attack Time.
    ///
    /// Offset 71, 0 to 255.
    #[must_use]
    pub fn mod_envelope_attack_time(&self) -> u8 {
        self.get(ParamId::ModEnvelopeAttackTime)
    }

    /// Sets Mod Envelope Attack Time, clamped to 0 to 255.
    pub fn set_mod_envelope_attack_time(&mut self, value: u8) {
        self.set_clamped(ParamId::ModEnvelopeAttackTime, value);
    }

    /// Mod Envelope Decay Time.
    ///
    /// Offset 72, 0 to 255.
    #[must_use]
    pub fn mod_envelope_decay_time(&self) -> u8 {
        self.get(ParamId::ModEnvelopeDecayTime)
    }

    /// Sets Mod Envelope Decay Time, clamped to 0 to 255.
    pub fn set_mod_envelope_decay_time(&mut self, value: u8) {
        self.set_clamped(ParamId::ModEnvelopeDecayTime, value);
    }

    /// Mod Envelope Sustain Level.
    ///
    /// Offset 73, 0 to 255.
    #[must_use]
    pub fn mod_envelope_sustain_level(&self) -> u8 {
        self.get(ParamId::ModEnvelopeSustainLevel)
    }

    /// Sets Mod Envelope Sustain Level, clamped to 0 to 255.
    pub fn set_mod_envelope_sustain_level(&mut self, value: u8) {
        self.set_clamped(ParamId::ModEnvelopeSustainLevel, value);
    }

    /// Mod Envelope Release Time.
    ///
    /// Offset 74, 0 to 255.
    #[must_use]
    pub fn mod_envelope_release_time(&self) -> u8 {
        self.get(ParamId::ModEnvelopeReleaseTime)
    }

    /// Sets Mod Envelope Release Time, clamped to 0 to 255.
    pub fn set_mod_envelope_release_time(&mut self, value: u8) {
        self.set_clamped(ParamId::ModEnvelopeReleaseTime, value);
    }

    /// Mod Envelope Trigger Mode.
    ///
    /// Offset 75. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn mod_envelope_trigger_mode(&self) -> Option<EnvelopeTrigger> {
        EnvelopeTrigger::from_raw(self.get(ParamId::ModEnvelopeTriggerMode))
    }

    /// Sets Mod Envelope Trigger Mode.
    pub fn set_mod_envelope_trigger_mode(&mut self, value: EnvelopeTrigger) {
        self.set_clamped(ParamId::ModEnvelopeTriggerMode, value.raw());
    }

    /// Mod Envelope Attack Curve.
    ///
    /// Offset 76, 0 to 255.
    #[must_use]
    pub fn mod_envelope_attack_curve(&self) -> u8 {
        self.get(ParamId::ModEnvelopeAttackCurve)
    }

    /// Sets Mod Envelope Attack Curve, clamped to 0 to 255.
    pub fn set_mod_envelope_attack_curve(&mut self, value: u8) {
        self.set_clamped(ParamId::ModEnvelopeAttackCurve, value);
    }

    /// Mod Envelope Decay Curve.
    ///
    /// Offset 77, 0 to 255.
    #[must_use]
    pub fn mod_envelope_decay_curve(&self) -> u8 {
        self.get(ParamId::ModEnvelopeDecayCurve)
    }

    /// Sets Mod Envelope Decay Curve, clamped to 0 to 255.
    pub fn set_mod_envelope_decay_curve(&mut self, value: u8) {
        self.set_clamped(ParamId::ModEnvelopeDecayCurve, value);
    }

    /// Mod Envelope Sustain Curve.
    ///
    /// Offset 78, 0 to 255.
    #[must_use]
    pub fn mod_envelope_sustain_curve(&self) -> u8 {
        self.get(ParamId::ModEnvelopeSustainCurve)
    }

    /// Sets Mod Envelope Sustain Curve, clamped to 0 to 255.
    pub fn set_mod_envelope_sustain_curve(&mut self, value: u8) {
        self.set_clamped(ParamId::ModEnvelopeSustainCurve, value);
    }

    /// Mod Envelope Release Curve.
    ///
    /// Offset 79, 0 to 255.
    #[must_use]
    pub fn mod_envelope_release_curve(&self) -> u8 {
        self.get(ParamId::ModEnvelopeReleaseCurve)
    }

    /// Sets Mod Envelope Release Curve, clamped to 0 to 255.
    pub fn set_mod_envelope_release_curve(&mut self, value: u8) {
        self.set_clamped(ParamId::ModEnvelopeReleaseCurve, value);
    }

    /// VCA Level.
    ///
    /// Offset 80, 0 to 255.
    #[must_use]
    pub fn vca_level(&self) -> u8 {
        self.get(ParamId::VcaLevel)
    }

    /// Sets VCA Level, clamped to 0 to 255.
    pub fn set_vca_level(&mut self, value: u8) {
        self.set_clamped(ParamId::VcaLevel, value);
    }

    /// VCA Envelope Depth.
    ///
    /// Offset 81, 0 to 255.
    #[must_use]
    pub fn vca_envelope_depth(&self) -> u8 {
        self.get(ParamId::VcaEnvelopeDepth)
    }

    /// Sets VCA Envelope Depth, clamped to 0 to 255.
    pub fn set_vca_envelope_depth(&mut self, value: u8) {
        self.set_clamped(ParamId::VcaEnvelopeDepth, value);
    }

    /// VCA Envelope Velocity Sensitivity.
    ///
    /// Offset 82, 0 to 255.
    #[must_use]
    pub fn vca_envelope_velocity_sensitivity(&self) -> u8 {
        self.get(ParamId::VcaEnvelopeVelocitySensitivity)
    }

    /// Sets VCA Envelope Velocity Sensitivity, clamped to 0 to 255.
    pub fn set_vca_envelope_velocity_sensitivity(&mut self, value: u8) {
        self.set_clamped(ParamId::VcaEnvelopeVelocitySensitivity, value);
    }

    /// VCA Pan Spread.
    ///
    /// Offset 83, 0 to 255.
    #[must_use]
    pub fn vca_pan_spread(&self) -> u8 {
        self.get(ParamId::VcaPanSpread)
    }

    /// Sets VCA Pan Spread, clamped to 0 to 255.
    pub fn set_vca_pan_spread(&mut self, value: u8) {
        self.set_clamped(ParamId::VcaPanSpread, value);
    }

    /// Voice Priority Mode.
    ///
    /// Offset 84. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn voice_priority_mode(&self) -> Option<VoicePriority> {
        VoicePriority::from_raw(self.get(ParamId::VoicePriorityMode))
    }

    /// Sets Voice Priority Mode.
    pub fn set_voice_priority_mode(&mut self, value: VoicePriority) {
        self.set_clamped(ParamId::VoicePriorityMode, value.raw());
    }

    /// Polyphony Mode.
    ///
    /// Offset 85. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn polyphony_mode(&self) -> Option<PolyphonyMode> {
        PolyphonyMode::from_raw(self.get(ParamId::PolyphonyMode))
    }

    /// Sets Polyphony Mode.
    pub fn set_polyphony_mode(&mut self, value: PolyphonyMode) {
        self.set_clamped(ParamId::PolyphonyMode, value.raw());
    }

    /// Envelope Trigger Mode.
    ///
    /// Offset 86. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn envelope_trigger_mode(&self) -> Option<KeyAssignMode> {
        KeyAssignMode::from_raw(self.get(ParamId::EnvelopeTriggerMode))
    }

    /// Sets Envelope Trigger Mode.
    pub fn set_envelope_trigger_mode(&mut self, value: KeyAssignMode) {
        self.set_clamped(ParamId::EnvelopeTriggerMode, value.raw());
    }

    /// Unison Detune.
    ///
    /// Offset 87, 0 to 255.
    #[must_use]
    pub fn unison_detune(&self) -> u8 {
        self.get(ParamId::UnisonDetune)
    }

    /// Sets Unison Detune, clamped to 0 to 255.
    pub fn set_unison_detune(&mut self, value: u8) {
        self.set_clamped(ParamId::UnisonDetune, value);
    }

    /// Voice Drift.
    ///
    /// Offset 88, 0 to 255.
    #[must_use]
    pub fn voice_drift(&self) -> u8 {
        self.get(ParamId::VoiceDrift)
    }

    /// Sets Voice Drift, clamped to 0 to 255.
    pub fn set_voice_drift(&mut self, value: u8) {
        self.set_clamped(ParamId::VoiceDrift, value);
    }

    /// Parameter Drift.
    ///
    /// Offset 89, 0 to 255.
    #[must_use]
    pub fn parameter_drift(&self) -> u8 {
        self.get(ParamId::ParameterDrift)
    }

    /// Sets Parameter Drift, clamped to 0 to 255.
    pub fn set_parameter_drift(&mut self, value: u8) {
        self.set_clamped(ParamId::ParameterDrift, value);
    }

    /// Drift Rate.
    ///
    /// Offset 90, 0 to 255.
    #[must_use]
    pub fn drift_rate(&self) -> u8 {
        self.get(ParamId::DriftRate)
    }

    /// Sets Drift Rate, clamped to 0 to 255.
    pub fn set_drift_rate(&mut self, value: u8) {
        self.set_clamped(ParamId::DriftRate, value);
    }

    /// OSC Portamento Balance.
    ///
    /// Offset 91, 0 to 255.
    #[must_use]
    pub fn osc_portamento_balance(&self) -> u8 {
        self.get(ParamId::OscPortamentoBalance)
    }

    /// Sets OSC Portamento Balance, clamped to 0 to 255.
    pub fn set_osc_portamento_balance(&mut self, value: u8) {
        self.set_clamped(ParamId::OscPortamentoBalance, value);
    }

    /// OSC Key Down Reset.
    ///
    /// Offset 92. Off is zero and on is one.
    #[must_use]
    pub fn osc_key_down_reset(&self) -> bool {
        self.get(ParamId::OscKeyDownReset) != 0
    }

    /// Sets OSC Key Down Reset.
    pub fn set_osc_key_down_reset(&mut self, value: bool) {
        self.set_clamped(ParamId::OscKeyDownReset, u8::from(value));
    }

    /// Mod 1 Source.
    ///
    /// Offset 93. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn mod1_source(&self) -> Option<ModSource> {
        ModSource::from_raw(self.get(ParamId::Mod1Source))
    }

    /// Sets Mod 1 Source.
    pub fn set_mod1_source(&mut self, value: ModSource) {
        self.set_clamped(ParamId::Mod1Source, value.raw());
    }

    /// Mod 1 Destination.
    ///
    /// Offset 94. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn mod1_destination(&self) -> Option<ModDestination> {
        ModDestination::from_raw(self.get(ParamId::Mod1Destination))
    }

    /// Sets Mod 1 Destination.
    pub fn set_mod1_destination(&mut self, value: ModDestination) {
        self.set_clamped(ParamId::Mod1Destination, value.raw());
    }

    /// Mod 1 Depth.
    ///
    /// Offset 95, 0 to 255.
    #[must_use]
    pub fn mod1_depth(&self) -> u8 {
        self.get(ParamId::Mod1Depth)
    }

    /// Sets Mod 1 Depth, clamped to 0 to 255.
    pub fn set_mod1_depth(&mut self, value: u8) {
        self.set_clamped(ParamId::Mod1Depth, value);
    }

    /// Mod 2 Source.
    ///
    /// Offset 96. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn mod2_source(&self) -> Option<ModSource> {
        ModSource::from_raw(self.get(ParamId::Mod2Source))
    }

    /// Sets Mod 2 Source.
    pub fn set_mod2_source(&mut self, value: ModSource) {
        self.set_clamped(ParamId::Mod2Source, value.raw());
    }

    /// Mod 2 Destination.
    ///
    /// Offset 97. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn mod2_destination(&self) -> Option<ModDestination> {
        ModDestination::from_raw(self.get(ParamId::Mod2Destination))
    }

    /// Sets Mod 2 Destination.
    pub fn set_mod2_destination(&mut self, value: ModDestination) {
        self.set_clamped(ParamId::Mod2Destination, value.raw());
    }

    /// Mod 2 Depth.
    ///
    /// Offset 98, 0 to 255.
    #[must_use]
    pub fn mod2_depth(&self) -> u8 {
        self.get(ParamId::Mod2Depth)
    }

    /// Sets Mod 2 Depth, clamped to 0 to 255.
    pub fn set_mod2_depth(&mut self, value: u8) {
        self.set_clamped(ParamId::Mod2Depth, value);
    }

    /// Mod 3 Source.
    ///
    /// Offset 99. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn mod3_source(&self) -> Option<ModSource> {
        ModSource::from_raw(self.get(ParamId::Mod3Source))
    }

    /// Sets Mod 3 Source.
    pub fn set_mod3_source(&mut self, value: ModSource) {
        self.set_clamped(ParamId::Mod3Source, value.raw());
    }

    /// Mod 3 Destination.
    ///
    /// Offset 100. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn mod3_destination(&self) -> Option<ModDestination> {
        ModDestination::from_raw(self.get(ParamId::Mod3Destination))
    }

    /// Sets Mod 3 Destination.
    pub fn set_mod3_destination(&mut self, value: ModDestination) {
        self.set_clamped(ParamId::Mod3Destination, value.raw());
    }

    /// Mod 3 Depth.
    ///
    /// Offset 101, 0 to 255.
    #[must_use]
    pub fn mod3_depth(&self) -> u8 {
        self.get(ParamId::Mod3Depth)
    }

    /// Sets Mod 3 Depth, clamped to 0 to 255.
    pub fn set_mod3_depth(&mut self, value: u8) {
        self.set_clamped(ParamId::Mod3Depth, value);
    }

    /// Mod 4 Source.
    ///
    /// Offset 102. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn mod4_source(&self) -> Option<ModSource> {
        ModSource::from_raw(self.get(ParamId::Mod4Source))
    }

    /// Sets Mod 4 Source.
    pub fn set_mod4_source(&mut self, value: ModSource) {
        self.set_clamped(ParamId::Mod4Source, value.raw());
    }

    /// Mod 4 Destination.
    ///
    /// Offset 103. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn mod4_destination(&self) -> Option<ModDestination> {
        ModDestination::from_raw(self.get(ParamId::Mod4Destination))
    }

    /// Sets Mod 4 Destination.
    pub fn set_mod4_destination(&mut self, value: ModDestination) {
        self.set_clamped(ParamId::Mod4Destination, value.raw());
    }

    /// Mod 4 Depth.
    ///
    /// Offset 104, 0 to 255.
    #[must_use]
    pub fn mod4_depth(&self) -> u8 {
        self.get(ParamId::Mod4Depth)
    }

    /// Sets Mod 4 Depth, clamped to 0 to 255.
    pub fn set_mod4_depth(&mut self, value: u8) {
        self.set_clamped(ParamId::Mod4Depth, value);
    }

    /// Mod 5 Source.
    ///
    /// Offset 105. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn mod5_source(&self) -> Option<ModSource> {
        ModSource::from_raw(self.get(ParamId::Mod5Source))
    }

    /// Sets Mod 5 Source.
    pub fn set_mod5_source(&mut self, value: ModSource) {
        self.set_clamped(ParamId::Mod5Source, value.raw());
    }

    /// Mod 5 Destination.
    ///
    /// Offset 106. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn mod5_destination(&self) -> Option<ModDestination> {
        ModDestination::from_raw(self.get(ParamId::Mod5Destination))
    }

    /// Sets Mod 5 Destination.
    pub fn set_mod5_destination(&mut self, value: ModDestination) {
        self.set_clamped(ParamId::Mod5Destination, value.raw());
    }

    /// Mod 5 Depth.
    ///
    /// Offset 107, 0 to 255.
    #[must_use]
    pub fn mod5_depth(&self) -> u8 {
        self.get(ParamId::Mod5Depth)
    }

    /// Sets Mod 5 Depth, clamped to 0 to 255.
    pub fn set_mod5_depth(&mut self, value: u8) {
        self.set_clamped(ParamId::Mod5Depth, value);
    }

    /// Mod 6 Source.
    ///
    /// Offset 108. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn mod6_source(&self) -> Option<ModSource> {
        ModSource::from_raw(self.get(ParamId::Mod6Source))
    }

    /// Sets Mod 6 Source.
    pub fn set_mod6_source(&mut self, value: ModSource) {
        self.set_clamped(ParamId::Mod6Source, value.raw());
    }

    /// Mod 6 Destination.
    ///
    /// Offset 109. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn mod6_destination(&self) -> Option<ModDestination> {
        ModDestination::from_raw(self.get(ParamId::Mod6Destination))
    }

    /// Sets Mod 6 Destination.
    pub fn set_mod6_destination(&mut self, value: ModDestination) {
        self.set_clamped(ParamId::Mod6Destination, value.raw());
    }

    /// Mod 6 Depth.
    ///
    /// Offset 110, 0 to 255.
    #[must_use]
    pub fn mod6_depth(&self) -> u8 {
        self.get(ParamId::Mod6Depth)
    }

    /// Sets Mod 6 Depth, clamped to 0 to 255.
    pub fn set_mod6_depth(&mut self, value: u8) {
        self.set_clamped(ParamId::Mod6Depth, value);
    }

    /// Mod 7 Source.
    ///
    /// Offset 111. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn mod7_source(&self) -> Option<ModSource> {
        ModSource::from_raw(self.get(ParamId::Mod7Source))
    }

    /// Sets Mod 7 Source.
    pub fn set_mod7_source(&mut self, value: ModSource) {
        self.set_clamped(ParamId::Mod7Source, value.raw());
    }

    /// Mod 7 Destination.
    ///
    /// Offset 112. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn mod7_destination(&self) -> Option<ModDestination> {
        ModDestination::from_raw(self.get(ParamId::Mod7Destination))
    }

    /// Sets Mod 7 Destination.
    pub fn set_mod7_destination(&mut self, value: ModDestination) {
        self.set_clamped(ParamId::Mod7Destination, value.raw());
    }

    /// Mod 7 Depth.
    ///
    /// Offset 113, 0 to 255.
    #[must_use]
    pub fn mod7_depth(&self) -> u8 {
        self.get(ParamId::Mod7Depth)
    }

    /// Sets Mod 7 Depth, clamped to 0 to 255.
    pub fn set_mod7_depth(&mut self, value: u8) {
        self.set_clamped(ParamId::Mod7Depth, value);
    }

    /// Mod 8 Source.
    ///
    /// Offset 114. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn mod8_source(&self) -> Option<ModSource> {
        ModSource::from_raw(self.get(ParamId::Mod8Source))
    }

    /// Sets Mod 8 Source.
    pub fn set_mod8_source(&mut self, value: ModSource) {
        self.set_clamped(ParamId::Mod8Source, value.raw());
    }

    /// Mod 8 Destination.
    ///
    /// Offset 115. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn mod8_destination(&self) -> Option<ModDestination> {
        ModDestination::from_raw(self.get(ParamId::Mod8Destination))
    }

    /// Sets Mod 8 Destination.
    pub fn set_mod8_destination(&mut self, value: ModDestination) {
        self.set_clamped(ParamId::Mod8Destination, value.raw());
    }

    /// Mod 8 Depth.
    ///
    /// Offset 116, 0 to 255.
    #[must_use]
    pub fn mod8_depth(&self) -> u8 {
        self.get(ParamId::Mod8Depth)
    }

    /// Sets Mod 8 Depth, clamped to 0 to 255.
    pub fn set_mod8_depth(&mut self, value: u8) {
        self.set_clamped(ParamId::Mod8Depth, value);
    }

    /// Ctrl Sequencer Enable.
    ///
    /// Offset 117. Off is zero and on is one.
    #[must_use]
    pub fn ctrl_sequencer_enable(&self) -> bool {
        self.get(ParamId::CtrlSequencerEnable) != 0
    }

    /// Sets Ctrl Sequencer Enable.
    pub fn set_ctrl_sequencer_enable(&mut self, value: bool) {
        self.set_clamped(ParamId::CtrlSequencerEnable, u8::from(value));
    }

    /// Ctrl Sequencer Clock Divider.
    ///
    /// Offset 118, 0 to 15.
    #[must_use]
    pub fn ctrl_sequencer_clock_divider(&self) -> u8 {
        self.get(ParamId::CtrlSequencerClockDivider)
    }

    /// Sets Ctrl Sequencer Clock Divider, clamped to 0 to 15.
    pub fn set_ctrl_sequencer_clock_divider(&mut self, value: u8) {
        self.set_clamped(ParamId::CtrlSequencerClockDivider, value);
    }

    /// Sequence Length.
    ///
    /// Offset 119, 0 to 31.
    #[must_use]
    pub fn sequence_length(&self) -> u8 {
        self.get(ParamId::SequenceLength)
    }

    /// Sets Sequence Length, clamped to 0 to 31.
    pub fn set_sequence_length(&mut self, value: u8) {
        self.set_clamped(ParamId::SequenceLength, value);
    }

    /// Sequencer Swing Timing.
    ///
    /// Offset 120, 0 to 255.
    #[must_use]
    pub fn sequencer_swing_timing(&self) -> u8 {
        self.get(ParamId::SequencerSwingTiming)
    }

    /// Sets Sequencer Swing Timing, clamped to 0 to 255.
    pub fn set_sequencer_swing_timing(&mut self, value: u8) {
        self.set_clamped(ParamId::SequencerSwingTiming, value);
    }

    /// Key Sync & Loop.
    ///
    /// Offset 121. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn key_sync_and_loop(&self) -> Option<SequencerSync> {
        SequencerSync::from_raw(self.get(ParamId::KeySyncAndLoop))
    }

    /// Sets Key Sync & Loop.
    pub fn set_key_sync_and_loop(&mut self, value: SequencerSync) {
        self.set_clamped(ParamId::KeySyncAndLoop, value.raw());
    }

    /// Slew Rate.
    ///
    /// Offset 122, 0 to 255.
    #[must_use]
    pub fn slew_rate(&self) -> u8 {
        self.get(ParamId::SlewRate)
    }

    /// Sets Slew Rate, clamped to 0 to 255.
    pub fn set_slew_rate(&mut self, value: u8) {
        self.set_clamped(ParamId::SlewRate, value);
    }

    /// Seq Step Value 1.
    ///
    /// Offset 123, 0 to 255.
    #[must_use]
    pub fn seq_step_value1(&self) -> u8 {
        self.get(ParamId::SeqStepValue1)
    }

    /// Sets Seq Step Value 1, clamped to 0 to 255.
    pub fn set_seq_step_value1(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue1, value);
    }

    /// Seq Step Value 2.
    ///
    /// Offset 124, 0 to 255.
    #[must_use]
    pub fn seq_step_value2(&self) -> u8 {
        self.get(ParamId::SeqStepValue2)
    }

    /// Sets Seq Step Value 2, clamped to 0 to 255.
    pub fn set_seq_step_value2(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue2, value);
    }

    /// Seq Step Value 3.
    ///
    /// Offset 125, 0 to 255.
    #[must_use]
    pub fn seq_step_value3(&self) -> u8 {
        self.get(ParamId::SeqStepValue3)
    }

    /// Sets Seq Step Value 3, clamped to 0 to 255.
    pub fn set_seq_step_value3(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue3, value);
    }

    /// Seq Step Value 4.
    ///
    /// Offset 126, 0 to 255.
    #[must_use]
    pub fn seq_step_value4(&self) -> u8 {
        self.get(ParamId::SeqStepValue4)
    }

    /// Sets Seq Step Value 4, clamped to 0 to 255.
    pub fn set_seq_step_value4(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue4, value);
    }

    /// Seq Step Value 5.
    ///
    /// Offset 127, 0 to 255.
    #[must_use]
    pub fn seq_step_value5(&self) -> u8 {
        self.get(ParamId::SeqStepValue5)
    }

    /// Sets Seq Step Value 5, clamped to 0 to 255.
    pub fn set_seq_step_value5(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue5, value);
    }

    /// Seq Step Value 6.
    ///
    /// Offset 128, 0 to 255.
    #[must_use]
    pub fn seq_step_value6(&self) -> u8 {
        self.get(ParamId::SeqStepValue6)
    }

    /// Sets Seq Step Value 6, clamped to 0 to 255.
    pub fn set_seq_step_value6(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue6, value);
    }

    /// Seq Step Value 7.
    ///
    /// Offset 129, 0 to 255.
    #[must_use]
    pub fn seq_step_value7(&self) -> u8 {
        self.get(ParamId::SeqStepValue7)
    }

    /// Sets Seq Step Value 7, clamped to 0 to 255.
    pub fn set_seq_step_value7(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue7, value);
    }

    /// Seq Step Value 8.
    ///
    /// Offset 130, 0 to 255.
    #[must_use]
    pub fn seq_step_value8(&self) -> u8 {
        self.get(ParamId::SeqStepValue8)
    }

    /// Sets Seq Step Value 8, clamped to 0 to 255.
    pub fn set_seq_step_value8(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue8, value);
    }

    /// Seq Step Value 9.
    ///
    /// Offset 131, 0 to 255.
    #[must_use]
    pub fn seq_step_value9(&self) -> u8 {
        self.get(ParamId::SeqStepValue9)
    }

    /// Sets Seq Step Value 9, clamped to 0 to 255.
    pub fn set_seq_step_value9(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue9, value);
    }

    /// Seq Step Value 10.
    ///
    /// Offset 132, 0 to 255.
    #[must_use]
    pub fn seq_step_value10(&self) -> u8 {
        self.get(ParamId::SeqStepValue10)
    }

    /// Sets Seq Step Value 10, clamped to 0 to 255.
    pub fn set_seq_step_value10(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue10, value);
    }

    /// Seq Step Value 11.
    ///
    /// Offset 133, 0 to 255.
    #[must_use]
    pub fn seq_step_value11(&self) -> u8 {
        self.get(ParamId::SeqStepValue11)
    }

    /// Sets Seq Step Value 11, clamped to 0 to 255.
    pub fn set_seq_step_value11(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue11, value);
    }

    /// Seq Step Value 12.
    ///
    /// Offset 134, 0 to 255.
    #[must_use]
    pub fn seq_step_value12(&self) -> u8 {
        self.get(ParamId::SeqStepValue12)
    }

    /// Sets Seq Step Value 12, clamped to 0 to 255.
    pub fn set_seq_step_value12(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue12, value);
    }

    /// Seq Step Value 13.
    ///
    /// Offset 135, 0 to 255.
    #[must_use]
    pub fn seq_step_value13(&self) -> u8 {
        self.get(ParamId::SeqStepValue13)
    }

    /// Sets Seq Step Value 13, clamped to 0 to 255.
    pub fn set_seq_step_value13(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue13, value);
    }

    /// Seq Step Value 14.
    ///
    /// Offset 136, 0 to 255.
    #[must_use]
    pub fn seq_step_value14(&self) -> u8 {
        self.get(ParamId::SeqStepValue14)
    }

    /// Sets Seq Step Value 14, clamped to 0 to 255.
    pub fn set_seq_step_value14(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue14, value);
    }

    /// Seq Step Value 15.
    ///
    /// Offset 137, 0 to 255.
    #[must_use]
    pub fn seq_step_value15(&self) -> u8 {
        self.get(ParamId::SeqStepValue15)
    }

    /// Sets Seq Step Value 15, clamped to 0 to 255.
    pub fn set_seq_step_value15(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue15, value);
    }

    /// Seq Step Value 16.
    ///
    /// Offset 138, 0 to 255.
    #[must_use]
    pub fn seq_step_value16(&self) -> u8 {
        self.get(ParamId::SeqStepValue16)
    }

    /// Sets Seq Step Value 16, clamped to 0 to 255.
    pub fn set_seq_step_value16(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue16, value);
    }

    /// Seq Step Value 17.
    ///
    /// Offset 139, 0 to 255.
    #[must_use]
    pub fn seq_step_value17(&self) -> u8 {
        self.get(ParamId::SeqStepValue17)
    }

    /// Sets Seq Step Value 17, clamped to 0 to 255.
    pub fn set_seq_step_value17(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue17, value);
    }

    /// Seq Step Value 18.
    ///
    /// Offset 140, 0 to 255.
    #[must_use]
    pub fn seq_step_value18(&self) -> u8 {
        self.get(ParamId::SeqStepValue18)
    }

    /// Sets Seq Step Value 18, clamped to 0 to 255.
    pub fn set_seq_step_value18(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue18, value);
    }

    /// Seq Step Value 19.
    ///
    /// Offset 141, 0 to 255.
    #[must_use]
    pub fn seq_step_value19(&self) -> u8 {
        self.get(ParamId::SeqStepValue19)
    }

    /// Sets Seq Step Value 19, clamped to 0 to 255.
    pub fn set_seq_step_value19(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue19, value);
    }

    /// Seq Step Value 20.
    ///
    /// Offset 142, 0 to 255.
    #[must_use]
    pub fn seq_step_value20(&self) -> u8 {
        self.get(ParamId::SeqStepValue20)
    }

    /// Sets Seq Step Value 20, clamped to 0 to 255.
    pub fn set_seq_step_value20(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue20, value);
    }

    /// Seq Step Value 21.
    ///
    /// Offset 143, 0 to 255.
    #[must_use]
    pub fn seq_step_value21(&self) -> u8 {
        self.get(ParamId::SeqStepValue21)
    }

    /// Sets Seq Step Value 21, clamped to 0 to 255.
    pub fn set_seq_step_value21(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue21, value);
    }

    /// Seq Step Value 22.
    ///
    /// Offset 144, 0 to 255.
    #[must_use]
    pub fn seq_step_value22(&self) -> u8 {
        self.get(ParamId::SeqStepValue22)
    }

    /// Sets Seq Step Value 22, clamped to 0 to 255.
    pub fn set_seq_step_value22(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue22, value);
    }

    /// Seq Step Value 23.
    ///
    /// Offset 145, 0 to 255.
    #[must_use]
    pub fn seq_step_value23(&self) -> u8 {
        self.get(ParamId::SeqStepValue23)
    }

    /// Sets Seq Step Value 23, clamped to 0 to 255.
    pub fn set_seq_step_value23(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue23, value);
    }

    /// Seq Step Value 24.
    ///
    /// Offset 146, 0 to 255.
    #[must_use]
    pub fn seq_step_value24(&self) -> u8 {
        self.get(ParamId::SeqStepValue24)
    }

    /// Sets Seq Step Value 24, clamped to 0 to 255.
    pub fn set_seq_step_value24(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue24, value);
    }

    /// Seq Step Value 25.
    ///
    /// Offset 147, 0 to 255.
    #[must_use]
    pub fn seq_step_value25(&self) -> u8 {
        self.get(ParamId::SeqStepValue25)
    }

    /// Sets Seq Step Value 25, clamped to 0 to 255.
    pub fn set_seq_step_value25(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue25, value);
    }

    /// Seq Step Value 26.
    ///
    /// Offset 148, 0 to 255.
    #[must_use]
    pub fn seq_step_value26(&self) -> u8 {
        self.get(ParamId::SeqStepValue26)
    }

    /// Sets Seq Step Value 26, clamped to 0 to 255.
    pub fn set_seq_step_value26(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue26, value);
    }

    /// Seq Step Value 27.
    ///
    /// Offset 149, 0 to 255.
    #[must_use]
    pub fn seq_step_value27(&self) -> u8 {
        self.get(ParamId::SeqStepValue27)
    }

    /// Sets Seq Step Value 27, clamped to 0 to 255.
    pub fn set_seq_step_value27(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue27, value);
    }

    /// Seq Step Value 28.
    ///
    /// Offset 150, 0 to 255.
    #[must_use]
    pub fn seq_step_value28(&self) -> u8 {
        self.get(ParamId::SeqStepValue28)
    }

    /// Sets Seq Step Value 28, clamped to 0 to 255.
    pub fn set_seq_step_value28(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue28, value);
    }

    /// Seq Step Value 29.
    ///
    /// Offset 151, 0 to 255.
    #[must_use]
    pub fn seq_step_value29(&self) -> u8 {
        self.get(ParamId::SeqStepValue29)
    }

    /// Sets Seq Step Value 29, clamped to 0 to 255.
    pub fn set_seq_step_value29(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue29, value);
    }

    /// Seq Step Value 30.
    ///
    /// Offset 152, 0 to 255.
    #[must_use]
    pub fn seq_step_value30(&self) -> u8 {
        self.get(ParamId::SeqStepValue30)
    }

    /// Sets Seq Step Value 30, clamped to 0 to 255.
    pub fn set_seq_step_value30(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue30, value);
    }

    /// Seq Step Value 31.
    ///
    /// Offset 153, 0 to 255.
    #[must_use]
    pub fn seq_step_value31(&self) -> u8 {
        self.get(ParamId::SeqStepValue31)
    }

    /// Sets Seq Step Value 31, clamped to 0 to 255.
    pub fn set_seq_step_value31(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue31, value);
    }

    /// Seq Step Value 32.
    ///
    /// Offset 154, 0 to 255.
    #[must_use]
    pub fn seq_step_value32(&self) -> u8 {
        self.get(ParamId::SeqStepValue32)
    }

    /// Sets Seq Step Value 32, clamped to 0 to 255.
    pub fn set_seq_step_value32(&mut self, value: u8) {
        self.set_clamped(ParamId::SeqStepValue32, value);
    }

    /// Arp `On/Off`.
    ///
    /// Offset 155. Off is zero and on is one.
    #[must_use]
    pub fn arp_on_off(&self) -> bool {
        self.get(ParamId::ArpOnOff) != 0
    }

    /// Sets Arp `On/Off`.
    pub fn set_arp_on_off(&mut self, value: bool) {
        self.set_clamped(ParamId::ArpOnOff, u8::from(value));
    }

    /// Arp Mode.
    ///
    /// Offset 156. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn arp_mode(&self) -> Option<ArpMode> {
        ArpMode::from_raw(self.get(ParamId::ArpMode))
    }

    /// Sets Arp Mode.
    pub fn set_arp_mode(&mut self, value: ArpMode) {
        self.set_clamped(ParamId::ArpMode, value.raw());
    }

    /// Arp Rate (tempo).
    ///
    /// Offset 157, 0 to 255.
    #[must_use]
    pub fn arp_rate_tempo(&self) -> u8 {
        self.get(ParamId::ArpRateTempo)
    }

    /// Sets Arp Rate (tempo), clamped to 0 to 255.
    pub fn set_arp_rate_tempo(&mut self, value: u8) {
        self.set_clamped(ParamId::ArpRateTempo, value);
    }

    /// Arp Clock.
    ///
    /// Offset 158, 0 to 12.
    #[must_use]
    pub fn arp_clock(&self) -> u8 {
        self.get(ParamId::ArpClock)
    }

    /// Sets Arp Clock, clamped to 0 to 12.
    pub fn set_arp_clock(&mut self, value: u8) {
        self.set_clamped(ParamId::ArpClock, value);
    }

    /// Arp Key Sync.
    ///
    /// Offset 159. Off is zero and on is one.
    #[must_use]
    pub fn arp_key_sync(&self) -> bool {
        self.get(ParamId::ArpKeySync) != 0
    }

    /// Sets Arp Key Sync.
    pub fn set_arp_key_sync(&mut self, value: bool) {
        self.set_clamped(ParamId::ArpKeySync, u8::from(value));
    }

    /// Arp Gate Time.
    ///
    /// Offset 160, 0 to 255.
    #[must_use]
    pub fn arp_gate_time(&self) -> u8 {
        self.get(ParamId::ArpGateTime)
    }

    /// Sets Arp Gate Time, clamped to 0 to 255.
    pub fn set_arp_gate_time(&mut self, value: u8) {
        self.set_clamped(ParamId::ArpGateTime, value);
    }

    /// Arp Hold.
    ///
    /// Offset 161. Off is zero and on is one.
    #[must_use]
    pub fn arp_hold(&self) -> bool {
        self.get(ParamId::ArpHold) != 0
    }

    /// Sets Arp Hold.
    pub fn set_arp_hold(&mut self, value: bool) {
        self.set_clamped(ParamId::ArpHold, u8::from(value));
    }

    /// Arp Pattern.
    ///
    /// Offset 162, 0 to 64.
    #[must_use]
    pub fn arp_pattern(&self) -> u8 {
        self.get(ParamId::ArpPattern)
    }

    /// Sets Arp Pattern, clamped to 0 to 64.
    pub fn set_arp_pattern(&mut self, value: u8) {
        self.set_clamped(ParamId::ArpPattern, value);
    }

    /// Arp Swing.
    ///
    /// Offset 163, 0 to 255.
    #[must_use]
    pub fn arp_swing(&self) -> u8 {
        self.get(ParamId::ArpSwing)
    }

    /// Sets Arp Swing, clamped to 0 to 255.
    pub fn set_arp_swing(&mut self, value: u8) {
        self.set_clamped(ParamId::ArpSwing, value);
    }

    /// Arp Octaves.
    ///
    /// Offset 164, 0 to 5.
    #[must_use]
    pub fn arp_octaves(&self) -> u8 {
        self.get(ParamId::ArpOctaves)
    }

    /// Sets Arp Octaves, clamped to 0 to 5.
    pub fn set_arp_octaves(&mut self, value: u8) {
        self.set_clamped(ParamId::ArpOctaves, value);
    }

    /// FX Routing.
    ///
    /// Offset 165. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn fx_routing(&self) -> Option<FxRouting> {
        FxRouting::from_raw(self.get(ParamId::FxRouting))
    }

    /// Sets FX Routing.
    pub fn set_fx_routing(&mut self, value: FxRouting) {
        self.set_clamped(ParamId::FxRouting, value.raw());
    }

    /// FX 1 Type.
    ///
    /// Offset 166. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn fx1_type(&self) -> Option<FxType> {
        FxType::from_raw(self.get(ParamId::Fx1Type))
    }

    /// Sets FX 1 Type.
    pub fn set_fx1_type(&mut self, value: FxType) {
        self.set_clamped(ParamId::Fx1Type, value.raw());
    }

    /// FX 1 Param 1.
    ///
    /// Offset 167, 0 to 255.
    #[must_use]
    pub fn fx1_param1(&self) -> u8 {
        self.get(ParamId::Fx1Param1)
    }

    /// Sets FX 1 Param 1, clamped to 0 to 255.
    pub fn set_fx1_param1(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx1Param1, value);
    }

    /// FX 1 Param 2.
    ///
    /// Offset 168, 0 to 255.
    #[must_use]
    pub fn fx1_param2(&self) -> u8 {
        self.get(ParamId::Fx1Param2)
    }

    /// Sets FX 1 Param 2, clamped to 0 to 255.
    pub fn set_fx1_param2(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx1Param2, value);
    }

    /// FX 1 Param 3.
    ///
    /// Offset 169, 0 to 255.
    #[must_use]
    pub fn fx1_param3(&self) -> u8 {
        self.get(ParamId::Fx1Param3)
    }

    /// Sets FX 1 Param 3, clamped to 0 to 255.
    pub fn set_fx1_param3(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx1Param3, value);
    }

    /// FX 1 Param 4.
    ///
    /// Offset 170, 0 to 255.
    #[must_use]
    pub fn fx1_param4(&self) -> u8 {
        self.get(ParamId::Fx1Param4)
    }

    /// Sets FX 1 Param 4, clamped to 0 to 255.
    pub fn set_fx1_param4(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx1Param4, value);
    }

    /// FX 1 Param 5.
    ///
    /// Offset 171, 0 to 255.
    #[must_use]
    pub fn fx1_param5(&self) -> u8 {
        self.get(ParamId::Fx1Param5)
    }

    /// Sets FX 1 Param 5, clamped to 0 to 255.
    pub fn set_fx1_param5(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx1Param5, value);
    }

    /// FX 1 Param 6.
    ///
    /// Offset 172, 0 to 255.
    #[must_use]
    pub fn fx1_param6(&self) -> u8 {
        self.get(ParamId::Fx1Param6)
    }

    /// Sets FX 1 Param 6, clamped to 0 to 255.
    pub fn set_fx1_param6(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx1Param6, value);
    }

    /// FX 1 Param 7.
    ///
    /// Offset 173, 0 to 255.
    #[must_use]
    pub fn fx1_param7(&self) -> u8 {
        self.get(ParamId::Fx1Param7)
    }

    /// Sets FX 1 Param 7, clamped to 0 to 255.
    pub fn set_fx1_param7(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx1Param7, value);
    }

    /// FX 1 Param 8.
    ///
    /// Offset 174, 0 to 255.
    #[must_use]
    pub fn fx1_param8(&self) -> u8 {
        self.get(ParamId::Fx1Param8)
    }

    /// Sets FX 1 Param 8, clamped to 0 to 255.
    pub fn set_fx1_param8(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx1Param8, value);
    }

    /// FX 1 Param 9.
    ///
    /// Offset 175, 0 to 255.
    #[must_use]
    pub fn fx1_param9(&self) -> u8 {
        self.get(ParamId::Fx1Param9)
    }

    /// Sets FX 1 Param 9, clamped to 0 to 255.
    pub fn set_fx1_param9(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx1Param9, value);
    }

    /// FX 1 Param 10.
    ///
    /// Offset 176, 0 to 255.
    #[must_use]
    pub fn fx1_param10(&self) -> u8 {
        self.get(ParamId::Fx1Param10)
    }

    /// Sets FX 1 Param 10, clamped to 0 to 255.
    pub fn set_fx1_param10(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx1Param10, value);
    }

    /// FX 1 Param 11.
    ///
    /// Offset 177, 0 to 255.
    #[must_use]
    pub fn fx1_param11(&self) -> u8 {
        self.get(ParamId::Fx1Param11)
    }

    /// Sets FX 1 Param 11, clamped to 0 to 255.
    pub fn set_fx1_param11(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx1Param11, value);
    }

    /// FX 1 Param 12.
    ///
    /// Offset 178, 0 to 255.
    #[must_use]
    pub fn fx1_param12(&self) -> u8 {
        self.get(ParamId::Fx1Param12)
    }

    /// Sets FX 1 Param 12, clamped to 0 to 255.
    pub fn set_fx1_param12(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx1Param12, value);
    }

    /// FX 2 Type.
    ///
    /// Offset 179. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn fx2_type(&self) -> Option<FxType> {
        FxType::from_raw(self.get(ParamId::Fx2Type))
    }

    /// Sets FX 2 Type.
    pub fn set_fx2_type(&mut self, value: FxType) {
        self.set_clamped(ParamId::Fx2Type, value.raw());
    }

    /// FX 2 Param 1.
    ///
    /// Offset 180, 0 to 255.
    #[must_use]
    pub fn fx2_param1(&self) -> u8 {
        self.get(ParamId::Fx2Param1)
    }

    /// Sets FX 2 Param 1, clamped to 0 to 255.
    pub fn set_fx2_param1(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx2Param1, value);
    }

    /// FX 2 Param 2.
    ///
    /// Offset 181, 0 to 255.
    #[must_use]
    pub fn fx2_param2(&self) -> u8 {
        self.get(ParamId::Fx2Param2)
    }

    /// Sets FX 2 Param 2, clamped to 0 to 255.
    pub fn set_fx2_param2(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx2Param2, value);
    }

    /// FX 2 Param 3.
    ///
    /// Offset 182, 0 to 255.
    #[must_use]
    pub fn fx2_param3(&self) -> u8 {
        self.get(ParamId::Fx2Param3)
    }

    /// Sets FX 2 Param 3, clamped to 0 to 255.
    pub fn set_fx2_param3(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx2Param3, value);
    }

    /// FX 2 Param 4.
    ///
    /// Offset 183, 0 to 255.
    #[must_use]
    pub fn fx2_param4(&self) -> u8 {
        self.get(ParamId::Fx2Param4)
    }

    /// Sets FX 2 Param 4, clamped to 0 to 255.
    pub fn set_fx2_param4(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx2Param4, value);
    }

    /// FX 2 Param 5.
    ///
    /// Offset 184, 0 to 255.
    #[must_use]
    pub fn fx2_param5(&self) -> u8 {
        self.get(ParamId::Fx2Param5)
    }

    /// Sets FX 2 Param 5, clamped to 0 to 255.
    pub fn set_fx2_param5(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx2Param5, value);
    }

    /// FX 2 Param 6.
    ///
    /// Offset 185, 0 to 255.
    #[must_use]
    pub fn fx2_param6(&self) -> u8 {
        self.get(ParamId::Fx2Param6)
    }

    /// Sets FX 2 Param 6, clamped to 0 to 255.
    pub fn set_fx2_param6(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx2Param6, value);
    }

    /// FX 2 Param 7.
    ///
    /// Offset 186, 0 to 255.
    #[must_use]
    pub fn fx2_param7(&self) -> u8 {
        self.get(ParamId::Fx2Param7)
    }

    /// Sets FX 2 Param 7, clamped to 0 to 255.
    pub fn set_fx2_param7(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx2Param7, value);
    }

    /// FX 2 Param 8.
    ///
    /// Offset 187, 0 to 255.
    #[must_use]
    pub fn fx2_param8(&self) -> u8 {
        self.get(ParamId::Fx2Param8)
    }

    /// Sets FX 2 Param 8, clamped to 0 to 255.
    pub fn set_fx2_param8(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx2Param8, value);
    }

    /// FX 2 Param 9.
    ///
    /// Offset 188, 0 to 255.
    #[must_use]
    pub fn fx2_param9(&self) -> u8 {
        self.get(ParamId::Fx2Param9)
    }

    /// Sets FX 2 Param 9, clamped to 0 to 255.
    pub fn set_fx2_param9(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx2Param9, value);
    }

    /// FX 2 Param 10.
    ///
    /// Offset 189, 0 to 255.
    #[must_use]
    pub fn fx2_param10(&self) -> u8 {
        self.get(ParamId::Fx2Param10)
    }

    /// Sets FX 2 Param 10, clamped to 0 to 255.
    pub fn set_fx2_param10(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx2Param10, value);
    }

    /// FX 2 Param 11.
    ///
    /// Offset 190, 0 to 255.
    #[must_use]
    pub fn fx2_param11(&self) -> u8 {
        self.get(ParamId::Fx2Param11)
    }

    /// Sets FX 2 Param 11, clamped to 0 to 255.
    pub fn set_fx2_param11(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx2Param11, value);
    }

    /// FX 2 Param 12.
    ///
    /// Offset 191, 0 to 255.
    #[must_use]
    pub fn fx2_param12(&self) -> u8 {
        self.get(ParamId::Fx2Param12)
    }

    /// Sets FX 2 Param 12, clamped to 0 to 255.
    pub fn set_fx2_param12(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx2Param12, value);
    }

    /// FX 3 Type.
    ///
    /// Offset 192. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn fx3_type(&self) -> Option<FxType> {
        FxType::from_raw(self.get(ParamId::Fx3Type))
    }

    /// Sets FX 3 Type.
    pub fn set_fx3_type(&mut self, value: FxType) {
        self.set_clamped(ParamId::Fx3Type, value.raw());
    }

    /// FX 3 Param 1.
    ///
    /// Offset 193, 0 to 255.
    #[must_use]
    pub fn fx3_param1(&self) -> u8 {
        self.get(ParamId::Fx3Param1)
    }

    /// Sets FX 3 Param 1, clamped to 0 to 255.
    pub fn set_fx3_param1(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx3Param1, value);
    }

    /// FX 3 Param 2.
    ///
    /// Offset 194, 0 to 255.
    #[must_use]
    pub fn fx3_param2(&self) -> u8 {
        self.get(ParamId::Fx3Param2)
    }

    /// Sets FX 3 Param 2, clamped to 0 to 255.
    pub fn set_fx3_param2(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx3Param2, value);
    }

    /// FX 3 Param 3.
    ///
    /// Offset 195, 0 to 255.
    #[must_use]
    pub fn fx3_param3(&self) -> u8 {
        self.get(ParamId::Fx3Param3)
    }

    /// Sets FX 3 Param 3, clamped to 0 to 255.
    pub fn set_fx3_param3(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx3Param3, value);
    }

    /// FX 3 Param 4.
    ///
    /// Offset 196, 0 to 255.
    #[must_use]
    pub fn fx3_param4(&self) -> u8 {
        self.get(ParamId::Fx3Param4)
    }

    /// Sets FX 3 Param 4, clamped to 0 to 255.
    pub fn set_fx3_param4(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx3Param4, value);
    }

    /// FX 3 Param 5.
    ///
    /// Offset 197, 0 to 255.
    #[must_use]
    pub fn fx3_param5(&self) -> u8 {
        self.get(ParamId::Fx3Param5)
    }

    /// Sets FX 3 Param 5, clamped to 0 to 255.
    pub fn set_fx3_param5(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx3Param5, value);
    }

    /// FX 3 Param 6.
    ///
    /// Offset 198, 0 to 255.
    #[must_use]
    pub fn fx3_param6(&self) -> u8 {
        self.get(ParamId::Fx3Param6)
    }

    /// Sets FX 3 Param 6, clamped to 0 to 255.
    pub fn set_fx3_param6(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx3Param6, value);
    }

    /// FX 3 Param 7.
    ///
    /// Offset 199, 0 to 255.
    #[must_use]
    pub fn fx3_param7(&self) -> u8 {
        self.get(ParamId::Fx3Param7)
    }

    /// Sets FX 3 Param 7, clamped to 0 to 255.
    pub fn set_fx3_param7(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx3Param7, value);
    }

    /// FX 3 Param 8.
    ///
    /// Offset 200, 0 to 255.
    #[must_use]
    pub fn fx3_param8(&self) -> u8 {
        self.get(ParamId::Fx3Param8)
    }

    /// Sets FX 3 Param 8, clamped to 0 to 255.
    pub fn set_fx3_param8(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx3Param8, value);
    }

    /// FX 3 Param 9.
    ///
    /// Offset 201, 0 to 255.
    #[must_use]
    pub fn fx3_param9(&self) -> u8 {
        self.get(ParamId::Fx3Param9)
    }

    /// Sets FX 3 Param 9, clamped to 0 to 255.
    pub fn set_fx3_param9(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx3Param9, value);
    }

    /// FX 3 Param 10.
    ///
    /// Offset 202, 0 to 255.
    #[must_use]
    pub fn fx3_param10(&self) -> u8 {
        self.get(ParamId::Fx3Param10)
    }

    /// Sets FX 3 Param 10, clamped to 0 to 255.
    pub fn set_fx3_param10(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx3Param10, value);
    }

    /// FX 3 Param 11.
    ///
    /// Offset 203, 0 to 255.
    #[must_use]
    pub fn fx3_param11(&self) -> u8 {
        self.get(ParamId::Fx3Param11)
    }

    /// Sets FX 3 Param 11, clamped to 0 to 255.
    pub fn set_fx3_param11(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx3Param11, value);
    }

    /// FX 3 Param 12.
    ///
    /// Offset 204, 0 to 255.
    #[must_use]
    pub fn fx3_param12(&self) -> u8 {
        self.get(ParamId::Fx3Param12)
    }

    /// Sets FX 3 Param 12, clamped to 0 to 255.
    pub fn set_fx3_param12(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx3Param12, value);
    }

    /// FX 4 Type.
    ///
    /// Offset 205. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn fx4_type(&self) -> Option<FxType> {
        FxType::from_raw(self.get(ParamId::Fx4Type))
    }

    /// Sets FX 4 Type.
    pub fn set_fx4_type(&mut self, value: FxType) {
        self.set_clamped(ParamId::Fx4Type, value.raw());
    }

    /// FX 4 Param 1.
    ///
    /// Offset 206, 0 to 255.
    #[must_use]
    pub fn fx4_param1(&self) -> u8 {
        self.get(ParamId::Fx4Param1)
    }

    /// Sets FX 4 Param 1, clamped to 0 to 255.
    pub fn set_fx4_param1(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx4Param1, value);
    }

    /// FX 4 Param 2.
    ///
    /// Offset 207, 0 to 255.
    #[must_use]
    pub fn fx4_param2(&self) -> u8 {
        self.get(ParamId::Fx4Param2)
    }

    /// Sets FX 4 Param 2, clamped to 0 to 255.
    pub fn set_fx4_param2(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx4Param2, value);
    }

    /// FX 4 Param 3.
    ///
    /// Offset 208, 0 to 255.
    #[must_use]
    pub fn fx4_param3(&self) -> u8 {
        self.get(ParamId::Fx4Param3)
    }

    /// Sets FX 4 Param 3, clamped to 0 to 255.
    pub fn set_fx4_param3(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx4Param3, value);
    }

    /// FX 4 Param 4.
    ///
    /// Offset 209, 0 to 255.
    #[must_use]
    pub fn fx4_param4(&self) -> u8 {
        self.get(ParamId::Fx4Param4)
    }

    /// Sets FX 4 Param 4, clamped to 0 to 255.
    pub fn set_fx4_param4(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx4Param4, value);
    }

    /// FX 4 Param 5.
    ///
    /// Offset 210, 0 to 255.
    #[must_use]
    pub fn fx4_param5(&self) -> u8 {
        self.get(ParamId::Fx4Param5)
    }

    /// Sets FX 4 Param 5, clamped to 0 to 255.
    pub fn set_fx4_param5(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx4Param5, value);
    }

    /// FX 4 Param 6.
    ///
    /// Offset 211, 0 to 255.
    #[must_use]
    pub fn fx4_param6(&self) -> u8 {
        self.get(ParamId::Fx4Param6)
    }

    /// Sets FX 4 Param 6, clamped to 0 to 255.
    pub fn set_fx4_param6(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx4Param6, value);
    }

    /// FX 4 Param 7.
    ///
    /// Offset 212, 0 to 255.
    #[must_use]
    pub fn fx4_param7(&self) -> u8 {
        self.get(ParamId::Fx4Param7)
    }

    /// Sets FX 4 Param 7, clamped to 0 to 255.
    pub fn set_fx4_param7(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx4Param7, value);
    }

    /// FX 4 Param 8.
    ///
    /// Offset 213, 0 to 255.
    #[must_use]
    pub fn fx4_param8(&self) -> u8 {
        self.get(ParamId::Fx4Param8)
    }

    /// Sets FX 4 Param 8, clamped to 0 to 255.
    pub fn set_fx4_param8(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx4Param8, value);
    }

    /// FX 4 Param 9.
    ///
    /// Offset 214, 0 to 255.
    #[must_use]
    pub fn fx4_param9(&self) -> u8 {
        self.get(ParamId::Fx4Param9)
    }

    /// Sets FX 4 Param 9, clamped to 0 to 255.
    pub fn set_fx4_param9(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx4Param9, value);
    }

    /// FX 4 Param 10.
    ///
    /// Offset 215, 0 to 255.
    #[must_use]
    pub fn fx4_param10(&self) -> u8 {
        self.get(ParamId::Fx4Param10)
    }

    /// Sets FX 4 Param 10, clamped to 0 to 255.
    pub fn set_fx4_param10(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx4Param10, value);
    }

    /// FX 4 Param 11.
    ///
    /// Offset 216, 0 to 255.
    #[must_use]
    pub fn fx4_param11(&self) -> u8 {
        self.get(ParamId::Fx4Param11)
    }

    /// Sets FX 4 Param 11, clamped to 0 to 255.
    pub fn set_fx4_param11(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx4Param11, value);
    }

    /// FX 4 Param 12.
    ///
    /// Offset 217, 0 to 255.
    #[must_use]
    pub fn fx4_param12(&self) -> u8 {
        self.get(ParamId::Fx4Param12)
    }

    /// Sets FX 4 Param 12, clamped to 0 to 255.
    pub fn set_fx4_param12(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx4Param12, value);
    }

    /// FX 1 Output Gain.
    ///
    /// Offset 218, 0 to 150.
    #[must_use]
    pub fn fx1_output_gain(&self) -> u8 {
        self.get(ParamId::Fx1OutputGain)
    }

    /// Sets FX 1 Output Gain, clamped to 0 to 150.
    pub fn set_fx1_output_gain(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx1OutputGain, value);
    }

    /// FX 2 Output Gain.
    ///
    /// Offset 219, 0 to 150.
    #[must_use]
    pub fn fx2_output_gain(&self) -> u8 {
        self.get(ParamId::Fx2OutputGain)
    }

    /// Sets FX 2 Output Gain, clamped to 0 to 150.
    pub fn set_fx2_output_gain(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx2OutputGain, value);
    }

    /// FX 3 Output Gain.
    ///
    /// Offset 220, 0 to 150.
    #[must_use]
    pub fn fx3_output_gain(&self) -> u8 {
        self.get(ParamId::Fx3OutputGain)
    }

    /// Sets FX 3 Output Gain, clamped to 0 to 150.
    pub fn set_fx3_output_gain(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx3OutputGain, value);
    }

    /// FX 4 Output Gain.
    ///
    /// Offset 221, 0 to 150.
    #[must_use]
    pub fn fx4_output_gain(&self) -> u8 {
        self.get(ParamId::Fx4OutputGain)
    }

    /// Sets FX 4 Output Gain, clamped to 0 to 150.
    pub fn set_fx4_output_gain(&mut self, value: u8) {
        self.set_clamped(ParamId::Fx4OutputGain, value);
    }

    /// FX Mode.
    ///
    /// Offset 222. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn fx_mode(&self) -> Option<FxMode> {
        FxMode::from_raw(self.get(ParamId::FxMode))
    }

    /// Sets FX Mode.
    pub fn set_fx_mode(&mut self, value: FxMode) {
        self.set_clamped(ParamId::FxMode, value.raw());
    }

    /// Program Category.
    ///
    /// Offset 240. `None` when the byte is one the table does not list,
    /// which is also what a program written by an older firmware reads as where
    /// that firmware numbered this table differently.
    #[must_use]
    pub fn program_category(&self) -> Option<ProgramCategory> {
        ProgramCategory::from_raw(self.get(ParamId::ProgramCategory))
    }

    /// Sets Program Category.
    pub fn set_program_category(&mut self, value: ProgramCategory) {
        self.set_clamped(ParamId::ProgramCategory, value.raw());
    }

    /// Program Transpose.
    ///
    /// Offset 241, 80 to 176.
    #[must_use]
    pub fn program_transpose(&self) -> u8 {
        self.get(ParamId::ProgramTranspose)
    }

    /// Sets Program Transpose, clamped to 80 to 176.
    pub fn set_program_transpose(&mut self, value: u8) {
        self.set_clamped(ParamId::ProgramTranspose, value);
    }
}
