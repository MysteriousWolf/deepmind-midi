//! The effect panels: what an engine's twelve bytes mean, and which algorithm
//! decides.
//!
//! Four engines, twelve raw parameter bytes each, and what those bytes are
//! depends on which of the 35 algorithms the engine is running. The parameter
//! table can only call them `FX 1 Param 3`, which is the truth and is unusable
//! in an editor. This is the table that says `FX 1 Param 3` is `Size` on a Room
//! Reverb and `Depth` on a Phaser.
//!
//! ```
//! use deepmind_midi::effect::{Algorithm, Engine};
//! use deepmind_midi::param::{DEFAULT_FIRMWARE, ParamId};
//!
//! // What the engine is running, read off the byte the program stores.
//! let room = Algorithm::for_value(2, DEFAULT_FIRMWARE).expect("a Room Reverb");
//! assert_eq!(room.full_name, "Room Reverb");
//! assert_eq!(room.category, "Reverb");
//!
//! // What its third slot is, and the parameter that addresses it.
//! let size = room.slot(3).expect("Room Reverb uses all twelve");
//! assert_eq!(size.title, "Size");
//! assert_eq!(size.unit, Some("m"));
//! assert_eq!(Engine::One.slot_parameter(3), Some(ParamId::Fx1Param3));
//! ```
//!
//! # Fewer than twelve
//!
//! [`Algorithm::slots`] is as long as the algorithm has parameters, which is
//! five for TC Deep Reverb and ten for Gated Reverb. A host that drew twelve
//! controls would be drawing seven that do nothing, so the length is the
//! answer to how many to draw.
//!
//! # Firmware decides the numbering
//!
//! Firmware 1.1 added Vintage Pitch and moved Rotary Speaker, so the byte that
//! selects an algorithm is not the same byte on 1.0.
//! [`Algorithm::for_value`] goes through the `FX Type` value table for the
//! firmware a device inquiry reported, and [`Algorithm::value_for`] is the way
//! back. There is deliberately no `value` field: a number that was right on one
//! firmware sitting beside a lookup that takes a version is the bug the whole
//! `_for` convention exists to prevent.
//!
//! # What this does not carry
//!
//! The manual's prose, for the reason [`param`](crate::param) gives: the
//! descriptions are in `docs/effects.md`, addressed by the same algorithm and
//! slot numbers, and keeping them out of the binary is worth more to the
//! targets this crate is meant for than having them here.
//!
//! Nor the grid, the control shapes and the measured panel colours, which are
//! in `spec/layout.toml` and are a host's business once it knows what the slots
//! are. Nor the raw values a selector's names sit at, which the manual does not
//! publish; see [`FxSlot::values`].

mod generated;

pub use generated::{ALGORITHM_COUNT, ENGINE_COUNT, SLOTS_PER_ENGINE};

use generated::{ALGORITHMS, ENGINES};

use core::fmt;

use crate::error::{Error, Result};
use crate::param::{Kind, ParamId, TableId};
use crate::sysex::inquiry::Version;

/// One of the four effect engines.
///
/// Which one a slot belongs to is what turns "slot 3 of a Room Reverb" into a
/// parameter a host can edit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Engine {
    /// The first engine, whose slots are `FX 1 Param 1` through `12`.
    One,
    /// The second engine.
    Two,
    /// The third engine.
    Three,
    /// The fourth engine.
    Four,
}

/// What one engine addresses, which is the same whatever it is running.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct EngineParameters {
    /// The parameter holding which algorithm is loaded.
    pub(crate) algorithm: ParamId,
    /// The parameter holding the engine's output gain.
    pub(crate) gain: ParamId,
    /// The twelve slot parameters, in slot order.
    pub(crate) slots: [ParamId; SLOTS_PER_ENGINE],
}

impl Engine {
    /// Every engine, in the order the program stores them.
    ///
    /// Declared as [`ENGINE_COUNT`] long, which the specification generates, so
    /// a synthesizer with a fifth engine fails to compile here rather than
    /// quietly losing one.
    pub const ALL: [Self; ENGINE_COUNT] = [Self::One, Self::Two, Self::Three, Self::Four];

    /// Returns the engine's zero-based index.
    #[must_use]
    pub const fn index(self) -> u8 {
        match self {
            Self::One => 0,
            Self::Two => 1,
            Self::Three => 2,
            Self::Four => 3,
        }
    }

    /// Returns the engine's number as the display writes it, 1 through 4.
    #[must_use]
    pub const fn number(self) -> u8 {
        self.index() + 1
    }

    /// Returns the engine with this number, 1 through 4.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EngineOutOfRange`] for anything else.
    pub const fn from_number(number: u8) -> Result<Self> {
        Ok(match number {
            1 => Self::One,
            2 => Self::Two,
            3 => Self::Three,
            4 => Self::Four,
            _ => return Err(Error::EngineOutOfRange(number)),
        })
    }

    /// Returns the parameter holding which algorithm this engine is running.
    #[must_use]
    pub fn algorithm_parameter(self) -> ParamId {
        self.table().algorithm
    }

    /// Returns the parameter holding this engine's output gain.
    ///
    /// The manual calls it `Level` where the modulation matrix names it, and
    /// `FX n Output Gain` in the parameter table. What it does to a slot
    /// depends on where the routing puts that slot, which is `spec/routing.toml`
    /// and is not published here.
    #[must_use]
    pub fn gain_parameter(self) -> ParamId {
        self.table().gain
    }

    /// Returns the parameter addressing one of this engine's slots, counting
    /// from 1.
    ///
    /// `None` past [`SLOTS_PER_ENGINE`]. A slot number an algorithm does not
    /// use still has a parameter: the engine holds twelve bytes whatever it is
    /// running.
    #[must_use]
    pub fn slot_parameter(self, slot: u8) -> Option<ParamId> {
        let index = usize::from(slot.checked_sub(1)?);
        self.table().slots.get(index).copied()
    }

    /// Returns every slot parameter this engine addresses, in slot order.
    #[must_use]
    pub fn slot_parameters(self) -> &'static [ParamId; SLOTS_PER_ENGINE] {
        &self.table().slots
    }

    /// Returns the engine whose slots include this parameter, if one does.
    ///
    /// The way back from a parameter a host is drawing to the engine whose
    /// algorithm says what it is. Only the twelve slots of an engine answer;
    /// its type and gain parameters are the engine's own settings rather than
    /// slots.
    #[must_use]
    pub fn of(parameter: ParamId) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|engine| engine.table().slots.contains(&parameter))
    }

    /// Returns what this engine addresses.
    ///
    /// The fallback is unreachable: [`Engine::ALL`] is [`ENGINE_COUNT`] long by
    /// declaration and `ENGINES` is the same length, so the index is always in
    /// range. It costs one line and keeps this layer free of a panic.
    fn table(self) -> &'static EngineParameters {
        ENGINES
            .get(usize::from(self.index()))
            .unwrap_or(&ENGINES[0])
    }
}

impl fmt::Display for Engine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FX {}", self.number())
    }
}

/// One of the algorithms an effect engine can run.
///
/// Reached through [`Algorithm::for_value`], which takes the byte `FX n Type`
/// holds and the firmware that byte is to be read against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
pub struct Algorithm {
    /// Name as the synthesizer's display writes it, such as `RoomRev`.
    ///
    /// The same name the `FX Type` value table gives, which is what joins the
    /// two across a firmware renumbering.
    pub name: &'static str,
    /// Name written out, such as `Room Reverb`.
    pub full_name: &'static str,
    /// The family the manual groups it with: `Reverb`, `Delay`, `Processing`
    /// or `Creative`.
    pub category: &'static str,
    /// The slots it uses, in slot order.
    ///
    /// Shorter than [`SLOTS_PER_ENGINE`] where the algorithm has fewer
    /// parameters, which is how a host learns not to draw controls that do
    /// nothing.
    pub slots: &'static [FxSlot],
}

impl Algorithm {
    /// Every algorithm, in the order the newest firmware numbers them.
    #[must_use]
    pub fn all() -> &'static [Self; ALGORITHM_COUNT] {
        &ALGORITHMS
    }

    /// Returns the algorithm a `FX n Type` value of `value` selects on
    /// `firmware`.
    ///
    /// Goes through the `FX Type` value table rather than indexing this list,
    /// because firmware 1.1 inserted an algorithm rather than appending one:
    /// 33 is Rotary Speaker on 1.0 and Vintage Pitch on 1.1.
    ///
    /// `None` for a byte the firmware's table does not list.
    #[must_use]
    pub fn for_value(value: u8, firmware: Version) -> Option<&'static Self> {
        let name = TableId::FxType
            .table_for(firmware)
            .name_of(u16::from(value))?;
        Self::by_name(name)
    }

    /// Returns the algorithm of this name, as the display writes it.
    #[must_use]
    pub fn by_name(name: &str) -> Option<&'static Self> {
        ALGORITHMS.iter().find(|algorithm| algorithm.name == name)
    }

    /// Returns the `FX n Type` value that selects this algorithm on `firmware`.
    ///
    /// `None` where that firmware does not offer it, which is Vintage Pitch on
    /// 1.0 and is the reason this is not a field.
    #[must_use]
    pub fn value_for(&self, firmware: Version) -> Option<u8> {
        TableId::FxType
            .table_for(firmware)
            .entries
            .iter()
            .find(|entry| entry.name == self.name)
            .and_then(|entry| u8::try_from(entry.value).ok())
    }

    /// Returns one of its slots, counting from 1.
    ///
    /// `None` for a slot this algorithm does not use, which is the same answer
    /// as "draw nothing here".
    #[must_use]
    pub fn slot(&self, slot: u8) -> Option<&'static FxSlot> {
        let index = usize::from(slot.checked_sub(1)?);
        self.slots.get(index)
    }

    /// Returns the slot an engine's `parameter` is, under this algorithm.
    ///
    /// `None` when the parameter is not one of that engine's twelve, and when
    /// it is one the algorithm leaves unused.
    #[must_use]
    pub fn slot_of(&self, engine: Engine, parameter: ParamId) -> Option<&'static FxSlot> {
        let index = engine
            .slot_parameters()
            .iter()
            .position(|slot| *slot == parameter)?;
        self.slots.get(index)
    }
}

impl fmt::Display for Algorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.full_name)
    }
}

/// One slot of an effect engine, as the algorithm running it defines the slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
pub struct FxSlot {
    /// Position within the engine's twelve parameters, counting from 1.
    ///
    /// [`Engine::slot_parameter`] turns it into the parameter that addresses
    /// the byte.
    pub slot: u8,
    /// The short name the synthesizer's display prints, such as `PRE`.
    pub reference: &'static str,
    /// The same name written out, such as `Pre-Delay`.
    ///
    /// Expanded by this project for a panel with room to be readable, not a
    /// spelling the hardware uses; [`reference`](Self::reference) is the
    /// hardware's own.
    pub title: &'static str,
    /// How the byte is read, in the same terms a program parameter uses.
    ///
    /// [`Kind::Switch`] for a slot with two states and [`Kind::Continuous`] for
    /// everything else, so that a host's "what control is this" code is one
    /// path rather than two.
    ///
    /// Never [`Kind::Enumerated`]. A slot that picks from a list picks from
    /// [`values`](Self::values), and the manual prints those names without the
    /// bytes they sit at, so there is no value table to point at and this
    /// library will not invent one.
    pub kind: Kind,
    /// The names the display shows, where it shows names rather than a number.
    ///
    /// In the order the manual prints them, which is not the same as knowing
    /// which byte shows which: the manual gives no raw-to-display mapping for
    /// an effect parameter. So these are a reading and not an address, and a
    /// control that sent one of them would be sending a guess. Empty for a slot
    /// the display shows a number on.
    pub values: &'static [&'static str],
    /// Unit of the displayed value, where the manual gives one.
    pub unit: Option<&'static str>,
    /// Lowest value the synthesizer displays, as the manual prints it.
    ///
    /// A display reading and not a raw byte: every slot is a byte, and the
    /// curve between the two ends is not published. [`None`] for a slot with
    /// names instead of a range.
    pub min: Option<&'static str>,
    /// Highest value the synthesizer displays, as the manual prints it.
    pub max: Option<&'static str>,
    /// Label shared by the slots that belong together, such as one side of a
    /// stereo engine or one band of an equaliser.
    ///
    /// Derived from the parameter names by this project rather than printed by
    /// the manual, so it is a convention for laying a panel out and not a fact
    /// about the hardware.
    pub group: Option<&'static str>,
    /// `true` when the engine acts on modulation reaching this slot.
    ///
    /// Every slot is addressable from the modulation matrix regardless, as
    /// `Fx n Param m`; this says the engine does something with what arrives.
    pub modulatable: bool,
}

impl FxSlot {
    /// Returns whether the display shows a name here rather than a number.
    ///
    /// Which is [`values`](Self::values) not being empty. Worth asking before
    /// drawing a control: the names are what to print, and none of them is
    /// something to send.
    #[must_use]
    pub const fn is_selector(&self) -> bool {
        !self.values.is_empty()
    }
}

impl fmt::Display for FxSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.title)
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use super::{ALGORITHM_COUNT, Algorithm, Engine, SLOTS_PER_ENGINE};
    use crate::param::{DEFAULT_FIRMWARE, Group, Kind, ParamId, TableId};
    use crate::sysex::inquiry::Version;

    /// Firmware 1.0, which numbers the algorithms differently.
    const FIRMWARE_1_0: Version = Version { major: 1, minor: 0 };

    #[test]
    fn an_engine_addresses_the_parameters_the_table_names() {
        assert_eq!(Engine::One.algorithm_parameter(), ParamId::Fx1Type);
        assert_eq!(Engine::One.gain_parameter(), ParamId::Fx1OutputGain);
        assert_eq!(Engine::One.slot_parameter(1), Some(ParamId::Fx1Param1));
        assert_eq!(Engine::Four.slot_parameter(12), Some(ParamId::Fx4Param12));

        // Slots count from one, and there are twelve of them.
        assert_eq!(Engine::One.slot_parameter(0), None);
        assert_eq!(Engine::One.slot_parameter(13), None);
    }

    /// Every slot parameter in the table belongs to exactly one engine, which
    /// is what makes the way back from a parameter unambiguous.
    #[test]
    fn every_effect_slot_parameter_belongs_to_one_engine() {
        let mut seen = 0;
        for parameter in ParamId::ALL.iter().copied() {
            let Some(engine) = Engine::of(parameter) else {
                continue;
            };
            assert_eq!(parameter.group(), Group::Effects);
            assert!(engine.slot_parameters().contains(&parameter));
            assert_eq!(
                Engine::ALL
                    .into_iter()
                    .filter(|other| other.slot_parameters().contains(&parameter))
                    .count(),
                1,
                "{parameter} is in two engines"
            );
            seen += 1;
        }
        assert_eq!(seen, Engine::ALL.len() * SLOTS_PER_ENGINE);

        // An engine's own settings are not slots of it.
        assert_eq!(Engine::of(ParamId::Fx1Type), None);
        assert_eq!(Engine::of(ParamId::Fx1OutputGain), None);
        assert_eq!(Engine::of(ParamId::VcfFrequency), None);
    }

    #[test]
    fn an_engine_number_round_trips() {
        for engine in Engine::ALL {
            assert_eq!(Engine::from_number(engine.number()), Ok(engine));
        }
        assert!(Engine::from_number(0).is_err());
        assert!(Engine::from_number(5).is_err());
    }

    /// The point of the table: `FX 1 Param 3` is `Size` on one algorithm and
    /// something else on another.
    #[test]
    fn one_byte_means_different_things_under_different_algorithms() {
        let room = Algorithm::for_value(2, DEFAULT_FIRMWARE).expect("a Room Reverb");
        let phaser = Algorithm::for_value(30, DEFAULT_FIRMWARE).expect("a Phaser");

        let third = ParamId::Fx1Param3;
        assert_eq!(
            room.slot_of(Engine::One, third).map(|slot| slot.title),
            Some("Size")
        );
        assert_ne!(
            phaser.slot_of(Engine::One, third).map(|slot| slot.title),
            Some("Size")
        );
    }

    /// A short slot list is how a host learns not to draw seven controls that
    /// do nothing.
    #[test]
    fn an_algorithm_with_fewer_parameters_has_fewer_slots() {
        let deep = Algorithm::for_value(0, DEFAULT_FIRMWARE).expect("a TC Deep Reverb");
        assert_eq!(deep.slots.len(), 5);
        assert!(deep.slot(6).is_none());
        // The parameter still exists; the algorithm just does nothing with it.
        assert_eq!(Engine::One.slot_parameter(6), Some(ParamId::Fx1Param6));
        assert_eq!(deep.slot_of(Engine::One, ParamId::Fx1Param6), None);

        for algorithm in Algorithm::all() {
            assert!(!algorithm.slots.is_empty(), "{algorithm} has no slots");
            assert!(algorithm.slots.len() <= SLOTS_PER_ENGINE);
        }
    }

    /// Firmware 1.1 inserted Vintage Pitch rather than appending it, so the
    /// same byte is two different algorithms.
    #[test]
    fn the_firmware_decides_which_algorithm_a_byte_selects() {
        assert_eq!(
            Algorithm::for_value(33, DEFAULT_FIRMWARE).map(|a| a.name),
            Some("Vintage Pitch")
        );
        assert_eq!(
            Algorithm::for_value(33, FIRMWARE_1_0).map(|a| a.name),
            Some("RotarySpkr")
        );
        // The last algorithm of the newer table is past the end of the older.
        assert_eq!(Algorithm::for_value(34, FIRMWARE_1_0), None);
    }

    #[test]
    fn an_algorithm_says_which_byte_selects_it() {
        let rotary = Algorithm::by_name("RotarySpkr").expect("the rotary speaker");
        assert_eq!(rotary.value_for(DEFAULT_FIRMWARE), Some(34));
        assert_eq!(rotary.value_for(FIRMWARE_1_0), Some(33));

        let vintage = Algorithm::by_name("Vintage Pitch").expect("the vintage pitch shifter");
        assert_eq!(vintage.value_for(DEFAULT_FIRMWARE), Some(33));
        assert_eq!(
            vintage.value_for(FIRMWARE_1_0),
            None,
            "firmware 1.0 does not offer it"
        );
    }

    /// The value table and this one are two renderings of the same 35
    /// algorithms, and a byte that names one has to reach the other.
    #[test]
    fn every_algorithm_the_value_table_names_has_a_panel() {
        for firmware in [DEFAULT_FIRMWARE, FIRMWARE_1_0] {
            let table = TableId::FxType.table_for(firmware);
            for entry in table.entries {
                let value = u8::try_from(entry.value).expect("35 algorithms fit in a byte");
                let algorithm = Algorithm::for_value(value, firmware)
                    .unwrap_or_else(|| panic!("{} has no panel on {firmware}", entry.name));
                assert_eq!(algorithm.name, entry.name);
                assert_eq!(algorithm.value_for(firmware), Some(value));
            }
        }
        assert_eq!(Algorithm::all().len(), ALGORITHM_COUNT);
    }

    /// A slot either sweeps between two printed ends or shows one of a printed
    /// set of names. A slot that did neither would be a control with nothing to
    /// draw.
    #[test]
    fn a_slot_has_either_a_range_or_a_set_of_names() {
        for algorithm in Algorithm::all() {
            for slot in algorithm.slots {
                assert!(!slot.reference.is_empty(), "{algorithm} {}", slot.slot);
                assert!(!slot.title.is_empty(), "{algorithm} {}", slot.slot);
                assert_ne!(
                    slot.is_selector(),
                    slot.min.is_some(),
                    "{algorithm} {} is both a range and a set of names, or neither",
                    slot.slot
                );
                // The wire values of a named set are not published, so a slot
                // is never pointed at a value table.
                assert!(matches!(slot.kind, Kind::Continuous | Kind::Switch));
            }
        }
    }

    #[test]
    fn slots_are_numbered_from_one_and_in_order() {
        for algorithm in Algorithm::all() {
            for (index, slot) in algorithm.slots.iter().enumerate() {
                let number = u8::try_from(index + 1).expect("twelve slots fit in a byte");
                assert_eq!(slot.slot, number, "{algorithm}");
                assert_eq!(algorithm.slot(number).map(|s| s.title), Some(slot.title));
            }
        }
    }
}
