//! The effect panels: what an engine's twelve bytes mean, and which algorithm
//! decides.
//!
//! Four engines, twelve raw parameter bytes each. What those bytes mean depends
//! on which of the 35 algorithms the engine is running. The parameter table can
//! only call them `FX 1 Param 3`. This table says `FX 1 Param 3` is `Size` on a
//! Room Reverb and `Depth` on a Phaser.
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
//! [`Algorithm::slots`] is as long as the algorithm has parameters: five for TC
//! Deep Reverb, ten for Gated Reverb. Its length is how many controls to draw.
//!
//! # Firmware decides the numbering
//!
//! Firmware 1.1 added Vintage Pitch and moved Rotary Speaker, so the byte that
//! selects an algorithm is not the same byte on 1.0. [`Algorithm::for_value`]
//! looks it up in the `FX Type` value table for the firmware a device inquiry
//! reported, and [`Algorithm::value_for`] goes the other way. There is no
//! `value` field, because a stored number would be right for one firmware only.
//!
//! # What this does not carry
//!
//! The manual's prose, for the reason [`param`](crate::param) gives. The
//! descriptions are in `docs/effects.md`, under the same algorithm and slot
//! numbers.
//!
//! Nor the raw values a selector's names sit at, which the manual does not
//! publish; see [`FxSlot::values`].
//!
//! # Drawing one
//!
//! [`Algorithm::panel`] is what the effect's own editor panel is made of: the
//! control to draw for a sweep, and the four colours measured off the manual's
//! figure. [`FxSlot::position`] is where the synthesizer's FX page puts a slot
//! on the one [`grid`] every algorithm shares. The grid is given in the
//! display's own pixels with the display's dimensions beside it, so a host can
//! scale it to whatever it draws in.
//!
//! Nothing in the algorithm table points at any of this, so a host that draws
//! its own panels does not carry it.
//!
//! # How the four are wired
//!
//! [`Routing`] is the ten topologies as edge lists: what reaches each engine,
//! which engines are summed on the way out, and which two put engines in a
//! feedback loop. It decides whether `FX 2 Output Gain` reaches the output at
//! all, since the manual defines a slot's level as the level of an effect that
//! is in parallel or last before the output stage. [`Mode`] is the same question
//! one level up: what `Insert`, `Send` and `Bypass` do to the analog and digital
//! paths.

mod generated;

pub use generated::{
    ALGORITHM_COUNT, ENGINE_COUNT, FAMILY_COUNT, MARK_PIXEL_SIDE, MODE_COUNT, ROUTING_COUNT,
    SLOTS_PER_ENGINE,
};

use generated::{ALGORITHMS, ENGINES, FAMILY_NAMES, GRID, MARKS, MODES, PANELS, ROUTINGS};

use core::fmt;

use crate::error::{Error, Result};
use crate::generator::{Generator, MAX_TAPS};
use crate::param::{DEFAULT_FIRMWARE, Kind, ParamId, TableId};
use crate::program::Program;
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
    /// losing one.
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
    /// range. It is there so this layer holds no panic.
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
///
/// # Every value is an effect
///
/// There is no `Off`, `None` or `Thru`, and no engine that is running nothing.
/// Section 7.2.4 of the manual — *"To load an effect into a slot ... select
/// from one of the following effects"* — lists these 35 and stops, and the
/// effects table in section 9.1 gives the same 35. `FX n Type` is declared
/// `0..=34` to match, so there is no value outside the table either.
///
/// What takes effects out of circuit is the `Bypass` [`Mode`], and that is the whole
/// block of four rather than one engine. Three algorithms carry their own
/// bypass in one of their twelve bytes instead, which is
/// [`FxSlot::is_enable`].
///
/// A host looking for the parameter that silences one engine will not find one.
/// `FX n Output Gain` is not it: the manual defines a slot's level as the level
/// of an effect that is in parallel or last before the output stage, so on six
/// of the ten [`Routing`]s an engine at zero gain still feeds the next engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
pub struct Algorithm {
    /// Where this algorithm sits in [`Algorithm::all`], which is what reaches
    /// the panel table beside it.
    ///
    /// Not the `FX n Type` byte, which firmware renumbered; see
    /// [`Algorithm::value_for`] for that.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) index: u8,
    /// Name as the synthesizer's display writes it, such as `RoomRev`.
    ///
    /// The same name the `FX Type` value table gives, which is what joins the
    /// two across a firmware renumbering.
    pub name: &'static str,
    /// Name written out, such as `Room Reverb`.
    pub full_name: &'static str,
    /// The family the manual groups it with: `Reverb`, `Delay`, `Processing`
    /// or `Creative`.
    ///
    /// The manual's own table of contents. [`Algorithm::family`] is what an
    /// effect does to a signal, which is the finer question and the one a mark
    /// is chosen by.
    pub category: &'static str,
    /// What this effect does to a signal.
    ///
    /// [`Algorithm::family`](Self::family) is how it is read.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) family: Family,
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

    /// Returns what this algorithm's own editor panel is made of.
    ///
    /// Measured off the figure the manual prints beside each algorithm's FX
    /// page in section 9.3. A Room Reverb and a Phaser are drawn differently
    /// there, so this says which.
    ///
    /// ```
    /// use deepmind_midi::effect::{Algorithm, Control};
    /// use deepmind_midi::param::DEFAULT_FIRMWARE;
    ///
    /// let room = Algorithm::for_value(2, DEFAULT_FIRMWARE).expect("a Room Reverb");
    /// assert_eq!(room.panel().control(), Control::Fader);
    /// assert_eq!(room.panel().cap().to_rgb(), [0x98, 0x93, 0x88]);
    /// ```
    #[must_use]
    pub fn panel(&self) -> &'static Panel {
        // Unreachable fallback: the index is generated beside the algorithm and
        // the two tables are declared the same length.
        PANELS.get(usize::from(self.index)).unwrap_or(&PANELS[0])
    }

    /// Returns what this effect does to a signal.
    ///
    /// [`category`](Self::category) is the four buckets the manual's table of
    /// contents uses, and `Creative` holds the phaser, the pitch shifter and
    /// the rotary speaker, which do three unrelated things. This is the same
    /// kind of published fact at the resolution a symbol needs.
    ///
    /// ```
    /// use deepmind_midi::effect::{Algorithm, Family};
    ///
    /// let phaser = Algorithm::by_name("Phaser").expect("a Phaser");
    /// assert_eq!(phaser.category, "Creative");
    /// assert_eq!(phaser.family(), Family::Modulation);
    ///
    /// let rotary = Algorithm::by_name("RotarySpkr").expect("a Rotary Speaker");
    /// assert_eq!(rotary.category, "Creative");
    /// assert_eq!(rotary.family(), Family::Rotary);
    /// ```
    #[must_use]
    pub const fn family(&self) -> Family {
        self.family
    }

    /// Returns the mark to draw for this effect.
    ///
    /// Its [`family`](Self::family)'s mark: nine marks across the 35, because
    /// the difference between a Hall Reverb and a Plate Reverb is not something
    /// a symbol carries and a drawing that implied it would be inventing one.
    /// The name is what tells those two apart.
    #[must_use]
    pub fn mark(&self) -> &'static Mark {
        self.family.mark()
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
    /// bytes they sit at, so there is no value table to point at.
    pub kind: Kind,
    /// The names the display shows, where it shows names rather than a number.
    ///
    /// In the order the manual prints them. Which byte shows which name is not
    /// published, so these are a reading rather than an address and none of them
    /// is something to send. Empty for a slot the display shows a number on.
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
    /// `true` when this switch takes the whole effect out of circuit.
    ///
    /// [`FxSlot::is_enable`](Self::is_enable) is how it is read, and says why
    /// there are only three.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) enable: bool,
    /// What this slot does to a signal.
    ///
    /// [`FxSlot::quantity`](Self::quantity) is how it is read.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) quantity: Quantity,
    /// Column of the FX page's grid this slot is drawn in, counting from 0.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) column: u8,
    /// Row of the FX page's grid this slot is drawn in, counting from 0.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) row: u8,
    /// What the manual says this slot does, where it says anything.
    ///
    /// Present only under the `descriptions` feature, so that a build without
    /// it carries neither the prose nor a pointer to it.
    /// [`FxSlot::description`](Self::description) is how it is read, and
    /// answers `None` when the feature is off.
    #[cfg(feature = "descriptions")]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) description: Option<&'static str>,
}

impl FxSlot {
    /// Returns where the synthesizer's own FX page draws this slot.
    ///
    /// On the one [`grid`] every algorithm shares: six columns and two rows,
    /// filled in slot order from the top left. The column and the row are what
    /// `spec/layout.toml` measured; the centre follows from the grid.
    #[must_use]
    pub const fn position(&self) -> Position {
        Position {
            column: self.column,
            row: self.row,
        }
    }

    /// Returns whether the display shows a name here rather than a number.
    ///
    /// Which is [`values`](Self::values) not being empty. Ask before drawing a
    /// control: the names are what to print, not what to send.
    #[must_use]
    pub const fn is_selector(&self) -> bool {
        !self.values.is_empty()
    }

    /// Returns what this slot does to a signal, as against what it is called.
    ///
    /// [`kind`](Self::kind) says what control to draw. This says what picture
    /// the slot belongs in, which is a different question with a different
    /// answer: a `Mix`, a `Feedback` and a `Pre-Delay` are all a byte `0..=255`
    /// and they do three unrelated things to a drawing.
    ///
    /// It is what lets a host group an algorithm's slots without matching on
    /// titles across 35 algorithms.
    ///
    /// ```
    /// use deepmind_midi::effect::{Algorithm, Quantity};
    ///
    /// let delay = Algorithm::by_name("3TapDelay").expect("a 3-Tap Delay");
    ///
    /// // Drawn as a selector, because it picks from ten printed fractions.
    /// let factor = delay.slot(5).expect("a FactorA slot");
    /// assert!(factor.is_selector());
    /// // And it is a time, because what it sets is when the tap lands.
    /// assert_eq!(factor.quantity(), Quantity::Time);
    /// ```
    #[must_use]
    pub const fn quantity(&self) -> Quantity {
        self.quantity
    }

    /// Returns whether this slot switches the whole effect in and out of
    /// circuit.
    ///
    /// True for three slots of the 35 algorithms, and it is the only
    /// slot-level off the instrument has: Stereo Imaging and Chorus D each
    /// spend their first slot on an `ON`, and the Noise Gate its eighth on a
    /// `PWR`. Nothing outside those three takes one engine out on its own; see
    /// [`Algorithm`] for what the instrument does instead.
    ///
    /// A host that would otherwise match on `ON` and `PWR` as strings asks
    /// this. Which way round the switch reads is
    /// [`min`](Self::min) and [`max`](Self::max), and the Noise Gate is the one
    /// that reads `ON` at the bottom and `OFF` at the top.
    ///
    /// Not the Rack Amplifier's `CAB`, which switches its cabinet simulation
    /// and leaves the amplifier running. That is a stage within the effect, so
    /// it is a plain [`Kind::Switch`].
    ///
    /// ```
    /// use deepmind_midi::effect::Algorithm;
    ///
    /// let gate = Algorithm::by_name("NoiseGate").expect("a Noise Gate");
    /// assert!(gate.slot(8).expect("a Power slot").is_enable());
    ///
    /// let amp = Algorithm::by_name("RackAmp").expect("a Rack Amplifier");
    /// assert!(!amp.slot(9).expect("a Cabinet slot").is_enable());
    /// ```
    #[must_use]
    pub const fn is_enable(&self) -> bool {
        self.enable
    }

    /// Returns what this slot does, in the manual's own words.
    ///
    /// [`title`](Self::title) says a slot is called `Damping`; this says what it
    /// damps. Transcribed from section 9.3 of the manual rather than written
    /// here, so it says what the engine's designers say it does. See NOTICE.
    ///
    /// # Behind a feature
    ///
    /// `None` unless the `descriptions` feature is on, and `None` with it on
    /// for a slot the manual describes no further. The signature is the same
    /// either way, so a host writes one code path and an embedded build carries
    /// none of the prose.
    ///
    /// ```
    /// use deepmind_midi::effect::Algorithm;
    /// use deepmind_midi::param::DEFAULT_FIRMWARE;
    ///
    /// let room = Algorithm::for_value(2, DEFAULT_FIRMWARE).expect("a Room Reverb");
    /// let decay = room.slot(2).expect("a Decay slot");
    /// assert_eq!(decay.title, "Decay");
    /// assert_eq!(decay.description().is_some(), cfg!(feature = "descriptions"));
    /// ```
    #[must_use]
    pub const fn description(&self) -> Option<&'static str> {
        #[cfg(feature = "descriptions")]
        {
            self.description
        }
        #[cfg(not(feature = "descriptions"))]
        {
            None
        }
    }
}

impl fmt::Display for FxSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.title)
    }
}

/// Returns the grid the synthesizer draws its own FX page on.
///
/// Six columns and two rows, in the pixels of the 128x64 display the 35
/// screenshots in the manual were measured on. The dimensions come with it, so a
/// host scales a proportion rather than adopting a size.
///
/// The page itself draws every slot as a circle, switches and selectors
/// included, with the slot's short name above it and its value below. What an
/// algorithm's own editor panel draws instead is [`Panel::control`], and the two
/// disagree for five effects: this is where a slot sits, that is what it looks
/// like.
///
/// ```
/// use deepmind_midi::effect::grid;
///
/// assert_eq!(grid().columns(), 6);
/// assert_eq!(grid().rows(), 2);
/// assert!((grid().display_width() - 128.0).abs() < f32::EPSILON);
/// ```
#[must_use]
pub const fn grid() -> &'static Grid {
    &GRID
}

/// The grid the synthesizer's FX page places every algorithm's slots on.
///
/// Reached through [`grid`]. Measured off the manual's own screenshots, all 35
/// of which agree; see [`FxSlot::position`] for where one slot lands on it.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Grid {
    columns: u8,
    rows: u8,
    display_width: f32,
    display_height: f32,
    first_x: f32,
    first_y: f32,
    column_pitch: f32,
    row_pitch: f32,
    control_diameter: f32,
}

impl Grid {
    /// Returns how many slots fit across a row.
    #[must_use]
    pub const fn columns(&self) -> u8 {
        self.columns
    }

    /// Returns how many rows the page holds.
    #[must_use]
    pub const fn rows(&self) -> u8 {
        self.rows
    }

    /// Returns the width of the display these measurements are in.
    #[must_use]
    pub const fn display_width(&self) -> f32 {
        self.display_width
    }

    /// Returns the height of the display these measurements are in.
    #[must_use]
    pub const fn display_height(&self) -> f32 {
        self.display_height
    }

    /// Returns the distance between the centres of two columns.
    #[must_use]
    pub const fn column_pitch(&self) -> f32 {
        self.column_pitch
    }

    /// Returns the distance between the centres of two rows.
    #[must_use]
    pub const fn row_pitch(&self) -> f32 {
        self.row_pitch
    }

    /// Returns the diameter of the circle the page draws for a slot.
    #[must_use]
    pub const fn control_diameter(&self) -> f32 {
        self.control_diameter
    }

    /// Returns the centre of one cell, in the display's own pixels.
    ///
    /// The cell a left-aligned row puts its `column`th slot in, which every row
    /// in the specification is; [`Row::align`] is what says so, and is there for
    /// a host laying a partial row out some other way.
    #[must_use]
    pub fn centre(&self, column: u8, row: u8) -> (f32, f32) {
        (
            self.first_x + f32::from(column) * self.column_pitch,
            self.first_y + f32::from(row) * self.row_pitch,
        )
    }
}

/// Where the FX page draws one slot.
///
/// Reached through [`FxSlot::position`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Position {
    column: u8,
    row: u8,
}

impl Position {
    /// Returns which column of the grid, counting from 0.
    #[must_use]
    pub const fn column(&self) -> u8 {
        self.column
    }

    /// Returns which row of the grid, counting from 0.
    #[must_use]
    pub const fn row(&self) -> u8 {
        self.row
    }

    /// Returns the centre of this position, in the display's own pixels.
    ///
    /// As [`Grid::centre`], which is what it calls.
    #[must_use]
    pub fn centre(&self) -> (f32, f32) {
        GRID.centre(self.column, self.row)
    }
}

/// What an effect does to a signal, as a closed set.
///
/// [`Algorithm::category`] is the manual's own four buckets, chosen for a table
/// of contents: `Creative` holds the phaser, the pitch shifter and the rotary
/// speaker. This is the finer question, and the one [`Algorithm::mark`] is
/// chosen by.
///
/// Where the two disagree is the point of having both. A host that wants to
/// reproduce the manual's grouping reads the category; one that wants to draw a
/// symbol, or to put the four delays together whatever page they are printed
/// on, reads this.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Family {
    /// A decaying tail. Thirteen of the 35, including the three that pair a
    /// reverb with something else.
    Reverb,
    /// Discrete repeats. The six the manual files under `Delay`.
    Delay,
    /// A swept delay: the chorus, the flanger and the phaser.
    Modulation,
    /// A response with a corner in it: the two equalisers and the Mood Filter.
    Filter,
    /// Level against level: the compressor and the noise gate.
    Dynamics,
    /// Harmonics added by driving a stage: the multi-band distortion and the
    /// rack amplifier.
    Distortion,
    /// The stereo field: the Stereo Imaging and Auto-Panning.
    Imaging,
    /// An interval put beside the note: the two pitch shifters.
    Pitch,
    /// A horn going round: the Rotary Speaker, which is its own family because
    /// it is its own thing.
    Rotary,
}

impl Family {
    /// Every family, in the order the specification declares them.
    ///
    /// Declared as [`FAMILY_COUNT`] long, which the specification generates, so
    /// a family added to `spec/marks.toml` fails to compile here rather than
    /// being left without a variant.
    pub const ALL: [Self; FAMILY_COUNT] = [
        Self::Reverb,
        Self::Delay,
        Self::Modulation,
        Self::Filter,
        Self::Dynamics,
        Self::Distortion,
        Self::Imaging,
        Self::Pitch,
        Self::Rotary,
    ];

    /// Returns this family's zero-based index, which is where its mark is.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Reverb => 0,
            Self::Delay => 1,
            Self::Modulation => 2,
            Self::Filter => 3,
            Self::Dynamics => 4,
            Self::Distortion => 5,
            Self::Imaging => 6,
            Self::Pitch => 7,
            Self::Rotary => 8,
        }
    }

    /// Returns the family's name, as the specification writes it.
    #[must_use]
    pub fn name(self) -> &'static str {
        // Unreachable fallback: `ALL` and the generated table are declared the
        // same length and this indexes by the position in `ALL`.
        FAMILY_NAMES.get(self.index()).copied().unwrap_or("Reverb")
    }

    /// Returns the mark this family is drawn with.
    #[must_use]
    pub fn mark(self) -> &'static Mark {
        // Unreachable fallback, for the same reason as [`Family::name`].
        MARKS.get(self.index()).unwrap_or(&MARKS[0])
    }

    /// Returns every algorithm in this family, in `FX Type` order.
    pub fn algorithms(self) -> impl Iterator<Item = &'static Algorithm> {
        ALGORITHMS
            .iter()
            .filter(move |algorithm| algorithm.family == self)
    }
}

impl fmt::Display for Family {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// A mark for an effect, drawn in the host's own materials.
///
/// Reached through [`Algorithm::mark`] or [`Family::mark`]. The data a drawing
/// is generated from, not the drawing: a host cannot theme, rescale, hit-test
/// or animate an SVG it did not lay out, and the same mark is wanted in a dark
/// panel at twelve points and at forty.
///
/// [`strokes`](Self::strokes) is what to draw, in a unit box with the origin at
/// the top left and y increasing downward. The host provides the size, the
/// stroke width and the colour. Nothing here is filled and no path closes.
///
/// ```
/// use deepmind_midi::effect::{Algorithm, Stroke};
///
/// let delay = Algorithm::by_name("3TapDelay").expect("a 3-Tap Delay");
/// let strokes = delay.mark().strokes();
///
/// // A baseline and the taps standing on it, all inside the unit box.
/// assert!(!strokes.is_empty());
/// for stroke in strokes {
///     if let Stroke::Line { points } = stroke {
///         assert!(points.iter().all(|p| (0.0..=1.0).contains(&p.x())));
///     }
/// }
/// ```
///
/// # What it is not
///
/// Not measured off the manual, unlike [`Panel`]'s colours. These are drawn by
/// this project: a reading of what each family does, rather than a
/// reproduction of anything the manufacturer prints. No font, no raster and no
/// licensed symbol set.
///
/// Not a layout. Whether the mark goes beside the name, in the corner of a
/// panel or on a tab is the host's question, the same way the [`grid`] is data
/// and the pixel size is not.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Mark {
    strokes: &'static [Stroke],
    pixels: Pixels,
}

impl Mark {
    /// Returns the strokes that make the mark up, in drawing order.
    #[must_use]
    pub const fn strokes(&self) -> &'static [Stroke] {
        self.strokes
    }

    /// Returns the same mark on a [`MARK_PIXEL_SIDE`] square grid, one bit a
    /// pixel.
    ///
    /// For a display with no room to stroke anything: an LCD row beside an
    /// effect name, or a hardware panel. Above about sixteen pixels a host
    /// wants [`strokes`](Self::strokes) instead, which has no size of its own.
    ///
    /// Drawn by hand rather than reduced from the strokes. At forty-nine pixels
    /// which pixels are lit is the whole of the design, and a one-pixel stroke
    /// put through a rasteriser at this size comes out as a smear with the idea
    /// gone.
    ///
    /// ```
    /// use deepmind_midi::effect::{Algorithm, MARK_PIXEL_SIDE};
    ///
    /// let rotary = Algorithm::by_name("RotarySpkr").expect("a Rotary Speaker");
    /// let pixels = rotary.mark().pixels();
    ///
    /// // A ring with its centre marked, so the middle pixel is lit and the
    /// // ones either side of it are not.
    /// let middle = (MARK_PIXEL_SIDE / 2) as u8;
    /// assert!(pixels.is_lit(middle, middle));
    /// assert!(!pixels.is_lit(middle - 1, middle));
    /// ```
    #[must_use]
    pub const fn pixels(&self) -> &Pixels {
        &self.pixels
    }
}

/// A [`Mark`] drawn on a square one-bit grid, for a display too small to stroke.
///
/// Reached through [`Mark::pixels`]. [`MARK_PIXEL_SIDE`] is how wide and how
/// tall, and the origin is the top left, matching the unit box the strokes are
/// in.
///
/// ```
/// use deepmind_midi::effect::{Algorithm, MARK_PIXEL_SIDE};
///
/// let filter = Algorithm::by_name("MoodFilter").expect("a Mood Filter");
/// let pixels = filter.mark().pixels();
///
/// // Blitting it is a walk over the grid.
/// let mut lit = 0;
/// for y in 0..MARK_PIXEL_SIDE as u8 {
///     for x in 0..MARK_PIXEL_SIDE as u8 {
///         if pixels.is_lit(x, y) {
///             lit += 1;
///         }
///     }
/// }
/// assert!(lit > 0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pixels {
    rows: [u8; MARK_PIXEL_SIDE],
}

impl Pixels {
    /// Builds a grid from its rows, a bit a pixel and bit 0 leftmost.
    #[must_use]
    pub const fn new(rows: [u8; MARK_PIXEL_SIDE]) -> Self {
        Self { rows }
    }

    /// Returns whether the pixel at `x`, `y` is lit, counting from the top
    /// left.
    ///
    /// `false` outside the grid, so a host walking a larger box than the mark
    /// gets blank rather than an answer it has to bounds-check itself.
    #[must_use]
    pub fn is_lit(&self, x: u8, y: u8) -> bool {
        if usize::from(x) >= MARK_PIXEL_SIDE {
            return false;
        }
        self.row(y) & (1 << x) != 0
    }

    /// Returns one row as its low [`MARK_PIXEL_SIDE`] bits, bit 0 leftmost.
    ///
    /// What a host blitting a row at a time wants. Zero past the bottom of the
    /// grid.
    #[must_use]
    pub fn row(&self, y: u8) -> u8 {
        self.rows.get(usize::from(y)).copied().unwrap_or(0)
    }

    /// Returns every row, top to bottom.
    #[must_use]
    pub const fn rows(&self) -> &[u8; MARK_PIXEL_SIDE] {
        &self.rows
    }
}

/// One stroke of a [`Mark`], in a unit box with the origin top left.
///
/// Two kinds, because a rotary speaker wants a circle and everything else wants
/// a polyline, and approximating the circle with one would put the number of
/// segments in this library rather than in the host that knows how big it is
/// drawing.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
pub enum Stroke {
    /// A polyline through two or more points, stroked and not closed.
    Line {
        /// The points, in order.
        points: &'static [Point],
    },
    /// An arc of a circle.
    ///
    /// Only the swept part is inside the box; the circle it is taken from may
    /// reach outside, which is what lets a shallow arc be drawn from a distant
    /// centre.
    Arc {
        /// Centre of the circle it is taken from.
        centre: Point,
        /// Radius of that circle.
        radius: f32,
        /// Where the arc begins, in turns clockwise from three o'clock.
        ///
        /// Clockwise because the box has y increasing downward, so a positive
        /// sweep turns the way a reader expects on screen.
        start: f32,
        /// How far it goes, in turns. A whole turn is a closed circle.
        sweep: f32,
    },
    /// A filled disc.
    ///
    /// The one thing in a mark that is filled rather than stroked, and the only
    /// one whose size is not the host's stroke width. Two marks use it: the
    /// source a reverb's wavefronts leave, and the centre of the rotary ring.
    Dot {
        /// Centre of the disc.
        centre: Point,
        /// Radius of the disc.
        radius: f32,
    },
}

/// A point of a [`Mark`], in a unit box with the origin top left.
///
/// Unitless, so a host multiplies by whatever it is drawing into.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Point {
    x: f32,
    y: f32,
}

impl Point {
    /// Builds a point from its two coordinates.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Returns the distance from the left of the box, 0 to 1.
    #[must_use]
    pub const fn x(self) -> f32 {
        self.x
    }

    /// Returns the distance from the top of the box, 0 to 1.
    #[must_use]
    pub const fn y(self) -> f32 {
        self.y
    }
}

/// What an effect's own editor panel is made of.
///
/// Reached through [`Algorithm::panel`]. Measured off the figure printed beside
/// each algorithm's FX page in section 9.3 of the manual: the panels differ per
/// algorithm, so this is a fact about the effect rather than a house style.
///
/// Twelve bytes of colour and a control shape. No artwork is reproduced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Panel {
    control: Control,
    chassis: Colour,
    face: Colour,
    cap: Colour,
    accent: Colour,
    rows: &'static [Row],
}

impl Panel {
    /// Returns what this panel draws for a parameter that sweeps a range.
    ///
    /// A switch or a selector is a button whatever the panel is made of, which
    /// is [`FxSlot::kind`] and [`FxSlot::is_selector`]; this is the shape the
    /// rest of the slots take.
    #[must_use]
    pub const fn control(&self) -> Control {
        self.control
    }

    /// Returns the case around the controls.
    #[must_use]
    pub const fn chassis(&self) -> Colour {
        self.chassis
    }

    /// Returns the surface the controls sit on.
    #[must_use]
    pub const fn face(&self) -> Colour {
        self.face
    }

    /// Returns the part a finger moves: a knob body or a fader cap.
    #[must_use]
    pub const fn cap(&self) -> Colour {
        self.cap
    }

    /// Returns the most saturated colour covering a visible share of the panel.
    ///
    /// A label colour on a panel that has one, an LED on one that does not, and
    /// [`cap`](Self::cap) again on a panel with neither.
    #[must_use]
    pub const fn accent(&self) -> Colour {
        self.accent
    }

    /// Returns the rows this algorithm fills, in the order they are drawn.
    #[must_use]
    pub const fn rows(&self) -> &'static [Row] {
        self.rows
    }
}

/// One row of an algorithm's slots on the FX page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Row {
    slots: &'static [u8],
    align: Align,
}

impl Row {
    /// Returns the slots on this row, in the order they are drawn, counting
    /// from 1.
    #[must_use]
    pub const fn slots(&self) -> &'static [u8] {
        self.slots
    }

    /// Returns what this row does with the space left over when it is not full.
    #[must_use]
    pub const fn align(&self) -> Align {
        self.align
    }
}

/// What an effect's own editor panel draws for a parameter that sweeps a range.
///
/// 29 of the 35 algorithms use rotary knobs, five use vertical faders on a
/// cream surface with a lit label strip, and the Vintage Room Reverb uses
/// numeric displays with a single knob.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Control {
    /// A rotary knob.
    Knob,
    /// A vertical fader.
    Fader,
    /// A numeric display.
    Display,
}

/// What an effect slot does to a signal, as against what it is called.
///
/// Reached through [`FxSlot::quantity`]. [`Kind`] is what control to draw; this
/// is what picture the slot belongs in. The two are separate because they
/// disagree: a delay's `Factor` is drawn as a selector, because it picks from
/// ten printed fractions, and what it sets is when a tap lands.
///
/// Derived from the parameter rather than stated by the manual, the same way
/// [`FxSlot::title`] and [`FxSlot::group`] are, so it is a convention this
/// project chose. `spec/panels.toml` records how, and names the slots where the
/// manual's own description decided it against what the name suggests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Quantity {
    /// When something happens: a pre-delay, an envelope stage, a tap's place in
    /// a pattern.
    Time,
    /// Where in the spectrum: a cut, a corner, a rate, an interval.
    Frequency,
    /// How loud: an input, an output, a threshold, a band's level.
    Gain,
    /// How much of the output goes back in.
    Feedback,
    /// How much effect there is: a mix, a drive, a density.
    Depth,
    /// Where in the stereo field: a pan, a width, a spread, a phase offset.
    Position,
    /// What the response looks like: a Q, a damping, a size, a wave morph.
    Shape,
    /// In or out.
    Switch,
    /// One of a list of named kinds: a preset, a cabinet, a distortion type.
    ///
    /// Not a quantity at all, which is why it is last. A slot that picks from a
    /// list still has to answer, and `Shape` would have been a lie.
    Selection,
}

impl Quantity {
    /// Every quantity, in the order they are declared.
    pub const ALL: [Self; 9] = [
        Self::Time,
        Self::Frequency,
        Self::Gain,
        Self::Feedback,
        Self::Depth,
        Self::Position,
        Self::Shape,
        Self::Switch,
        Self::Selection,
    ];
}

/// What a row that is not full does with the space left over.
///
/// [`Align::Left`] on every row of every algorithm as measured. The rest of the
/// variants are what a host with a different panel width says instead, which is
/// what the field is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Align {
    /// Packed against the start of the row.
    #[default]
    Left,
    /// Packed against the end of the row.
    Right,
    /// Packed together in the middle.
    Centre,
    /// Spread across the whole row.
    Spread,
}

/// One colour measured off a printed panel.
///
/// Three components rather than a string, so that no host has to parse the same
/// six hex characters. How the specification stores it is this library's
/// business.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Colour {
    red: u8,
    green: u8,
    blue: u8,
}

impl Colour {
    /// Builds a colour from its three components.
    #[must_use]
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    /// Returns the red component.
    #[must_use]
    pub const fn red(self) -> u8 {
        self.red
    }

    /// Returns the green component.
    #[must_use]
    pub const fn green(self) -> u8 {
        self.green
    }

    /// Returns the blue component.
    #[must_use]
    pub const fn blue(self) -> u8 {
        self.blue
    }

    /// Returns the three components, in the order a `#rrggbb` string writes
    /// them.
    #[must_use]
    pub const fn to_rgb(self) -> [u8; 3] {
        [self.red, self.green, self.blue]
    }
}

impl fmt::Display for Colour {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.red, self.green, self.blue)
    }
}

/// One of the ten ways the four engines can be wired.
///
/// The `FX Routing` byte picks one. [`enums`](crate::param::TableId) names them
/// the way the manual does, `Parallel 1/2, parallel 3/4`, which is for a person
/// to read. This is the same ten topologies as edge lists, which is what a host
/// draws a chain from.
///
/// ```
/// use deepmind_midi::effect::{Engine, Routing, Source};
///
/// // M-1 is the four in series, and only the fourth reaches the output.
/// let serial = Routing::for_value(0).expect("ten topologies");
/// assert_eq!(serial.label(), "M-1");
/// assert_eq!(serial.feeds(Engine::One), &[Source::Input]);
/// assert_eq!(serial.feeds(Engine::Two), &[Source::Engine(Engine::One)]);
/// assert_eq!(serial.output(), &[Engine::Four]);
/// assert!(!serial.is_feedback());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Routing {
    value: u8,
    label: &'static str,
    name: &'static str,
    feedback: bool,
    note: Option<&'static str>,
    feeds: [&'static [Source]; ENGINE_COUNT],
    output: &'static [Engine],
}

impl Routing {
    /// Returns the parameter holding which topology is selected.
    #[must_use]
    pub const fn parameter() -> ParamId {
        ParamId::FxRouting
    }

    /// Returns every topology, in the order the `FX Routing` byte numbers them.
    #[must_use]
    pub fn all() -> &'static [Self; ROUTING_COUNT] {
        &ROUTINGS
    }

    /// Returns the topology the `FX Routing` byte selects.
    ///
    /// `None` for a value the parameter does not accept. A lookup rather than an
    /// index, so a firmware that renumbers these has somewhere to say so, the way
    /// the value tables already do.
    #[must_use]
    pub fn for_value(value: u8) -> Option<&'static Self> {
        ROUTINGS.iter().find(|routing| routing.value == value)
    }

    /// Returns the `FX Routing` value that selects this topology.
    #[must_use]
    pub const fn value(&self) -> u8 {
        self.value
    }

    /// Returns the manual's label for it, `M-1` through `M-10`.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        self.label
    }

    /// Returns the name the `FX Routing` value table gives it.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Returns what reaches this engine: the block's own input, or other
    /// engines.
    #[must_use]
    pub fn feeds(&self, engine: Engine) -> &'static [Source] {
        // Unreachable fallback: the table is generated `ENGINE_COUNT` long and
        // an engine's index is inside it by construction.
        self.feeds
            .get(usize::from(engine.index()))
            .copied()
            .unwrap_or(&[])
    }

    /// Returns the engines whose outputs are summed to leave the block.
    ///
    /// What decides whether an engine's `FX n Output Gain` reaches the output:
    /// the manual defines a slot's level as the level of an effect configured
    /// in parallel, or last before the output stage.
    #[must_use]
    pub const fn output(&self) -> &'static [Engine] {
        self.output
    }

    /// Returns whether any engine is in a loop around the main path.
    ///
    /// Two of the ten are. A host drawing one of those without knowing it draws
    /// a wrong picture confidently.
    #[must_use]
    pub const fn is_feedback(&self) -> bool {
        self.feedback
    }

    /// Returns what the specification records about this topology beyond its
    /// graph, which is where the loop taps on the two that have one.
    #[must_use]
    pub const fn note(&self) -> Option<&'static str> {
        self.note
    }
}

impl fmt::Display for Routing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.label, self.name)
    }
}

/// What a signal reaching an engine came from.
///
/// An enum rather than the specification's `0` for the block input, so that no
/// host has to remember which engine number means "not an engine".
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Source {
    /// The FX block's own input.
    Input,
    /// The output of another engine.
    Engine(Engine),
}

/// What one `FX Mode` setting does to the two paths through the instrument.
///
/// The analog path carries the voices straight to the output stage; the digital
/// path carries them through the FX block. `Bypass` is a true bypass, with the
/// DSP out of circuit rather than muted, which is why a host should ask this
/// rather than match on the value table's name.
///
/// ```
/// use deepmind_midi::effect::Mode;
///
/// let bypass = Mode::for_value(2).expect("three modes");
/// assert_eq!(bypass.name(), "Bypass");
/// assert!(bypass.analog_path());
/// assert!(!bypass.digital_path());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Mode {
    value: u8,
    name: &'static str,
    analog_path: bool,
    digital_path: bool,
}

impl Mode {
    /// Returns the parameter holding which mode is selected.
    #[must_use]
    pub const fn parameter() -> ParamId {
        ParamId::FxMode
    }

    /// Returns every mode, in the order the `FX Mode` byte numbers them.
    #[must_use]
    pub fn all() -> &'static [Self; MODE_COUNT] {
        &MODES
    }

    /// Returns the mode the `FX Mode` byte selects.
    ///
    /// `None` for a value the parameter does not accept.
    #[must_use]
    pub fn for_value(value: u8) -> Option<&'static Self> {
        MODES.iter().find(|mode| mode.value == value)
    }

    /// Returns the `FX Mode` value that selects this mode.
    #[must_use]
    pub const fn value(&self) -> u8 {
        self.value
    }

    /// Returns the name the `FX Mode` value table gives it.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Returns whether the voices reach the output stage directly.
    #[must_use]
    pub const fn analog_path(&self) -> bool {
        self.analog_path
    }

    /// Returns whether the voices run through the FX block.
    #[must_use]
    pub const fn digital_path(&self) -> bool {
        self.digital_path
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name)
    }
}

/// The ten fractions a delay's `Factor` slot picks from, in the order the manual
/// prints them.
///
/// The same ten on every `Factor` slot of both tap delays, which
/// `every_factor_slot_offers_the_same_ten_fractions` holds against
/// [`FxSlot::values`] so that this table cannot drift from the specification.
const FACTORS: [(f32, &str); 10] = [
    (0.25, "1/4"),
    (1.0 / 3.0, "1/3"),
    (0.5, "1/2"),
    (2.0 / 3.0, "2/3"),
    (0.75, "3/4"),
    (1.0, "1"),
    (4.0 / 3.0, "4/3"),
    (1.5, "3/2"),
    (2.0, "2"),
    (3.0, "3"),
];

/// Returns what this engine is doing to a signal, where that follows from
/// published parameters.
///
/// [`None`] for most of the 35, and that is the answer rather than a gap. A
/// reverb's impulse response is its designer's and is not published; a library
/// that invented a plausible one would be drawing something that looks like
/// information and is not. The same goes for a compressor's exact knee and a
/// phaser's comb.
///
/// [`Some`] for the 3-Tap and 4-Tap delays. Their panels are literally a time
/// and a gain per tap, and the times are ratios of the master delay that the
/// manual prints as fractions, so the picture follows from the parameters
/// rather than from anybody's DSP. The horizontal is [`Scale::Normalised`](crate::generator::Scale::Normalised), from
/// zero to the furthest tap: the ratios between the taps are published, and the
/// master delay time they are ratios *of* is a byte with no published curve, so
/// there is no axis to label.
///
/// Read on [`DEFAULT_FIRMWARE`], as the program's own `fx1_type` and its
/// siblings are, so an engine running Vintage Pitch on firmware 1.0 is read as
/// the algorithm 1.1 numbers there.
///
/// # Why not the other four delays
///
/// The Stereo Delay and the Tel-Ray have a master time and a feedback rather
/// than enumerated taps, so what they make is an endless train whose count
/// depends on where a host decides the repeats have died away. That is the
/// host's decision about its own picture, not a fact this library holds. The
/// Decimator Delay is the same with a decimator in front of it, and Mod, Delay
/// and Reverb has a reverb across the end of it.
///
/// ```
/// use deepmind_midi::ProtocolVersion;
/// use deepmind_midi::effect::{self, Engine};
/// use deepmind_midi::program::{FxType, Program};
///
/// let mut program = Program::new(ProtocolVersion::V6);
/// program.set_fx1_type(FxType::ThreeTapDelay);
/// program.set_fx1_param2(255);    // the first tap, at full gain
///
/// // The master tap is the furthest out here, so it lands at the end.
/// let response = effect::response(&program, Engine::One).expect("a 3-Tap Delay");
/// assert!(response.at(1.0) > 0.9);
/// assert_eq!(response.at(0.6), 0.0);
///
/// // And a reverb has none, which is an answer rather than a failure.
/// program.set_fx1_type(FxType::RoomRev);
/// assert!(effect::response(&program, Engine::One).is_none());
/// ```
#[must_use]
pub fn response(program: &Program, engine: Engine) -> Option<Generator> {
    let algorithm =
        Algorithm::for_value(program.get(engine.algorithm_parameter()), DEFAULT_FIRMWARE)?;

    // Slot numbers of the gain and the factor for each tap, in tap order. The
    // first tap runs at the master time and so has no factor of its own.
    let layout: &[(u8, Option<u8>)] = match algorithm.name {
        "3TapDelay" => &[(2, None), (6, Some(5)), (9, Some(8))],
        "4TapDelay" => &[(2, None), (6, Some(5)), (8, Some(7)), (10, Some(9))],
        _ => return None,
    };

    let mut taps = [(0.0, 0.0); MAX_TAPS];
    let mut used = 0;
    let mut furthest = 0.0_f32;
    for &(gain, factor) in layout {
        let at = match factor {
            None => 1.0,
            Some(slot) => fraction(program, engine, slot)?,
        };
        let height = engine
            .slot_parameter(gain)
            .map_or(0.0, |parameter| f32::from(program.get(parameter)) / 255.0);
        *taps.get_mut(used)? = (at, height);
        furthest = furthest.max(at);
        used += 1;
    }

    // Normalised to the furthest tap, since the master time the ratios are
    // ratios of has no published curve to put an axis in.
    if furthest > 0.0 {
        for tap in taps.iter_mut().take(used) {
            tap.0 /= furthest;
        }
    }
    Some(crate::generator::taps(taps, used))
}

/// Returns the fraction of the master delay time a `Factor` slot selects.
///
/// # A reading, not a published mapping
///
/// The ten fractions are printed in the manual and are in [`FxSlot::values`].
/// Which byte shows which of them is not, for the reason
/// [`FxSlot::values`] gives, so this divides the byte's range evenly among them.
/// That is the only mapping a `0..=255` byte and ten named options admit, and it
/// is the one thing in this function that is not off the page.
fn fraction(program: &Program, engine: Engine, slot: u8) -> Option<f32> {
    let raw = program.get(engine.slot_parameter(slot)?);
    let index = usize::from(raw) * FACTORS.len() / 256;
    FACTORS.get(index).map(|&(fraction, _)| fraction)
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use super::{
        ALGORITHM_COUNT, Algorithm, Align, Engine, Family, MARK_PIXEL_SIDE, MODE_COUNT, Mode,
        Quantity, ROUTING_COUNT, Routing, SLOTS_PER_ENGINE, Source, Stroke, grid,
    };
    use crate::ids::ProtocolVersion;
    use crate::param::{DEFAULT_FIRMWARE, Group, Kind, ParamId, TableId};
    use crate::program::{FxType, Program};
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

    /// The grid is one measurement shared by all 35, and every slot has to land
    /// on it: a slot off the page would be a control drawn nowhere.
    #[test]
    fn every_slot_is_placed_on_the_page() {
        let grid = grid();
        assert!(grid.columns() > 0 && grid.rows() > 0);
        for algorithm in Algorithm::all() {
            let panel = algorithm.panel();
            let placed: usize = panel.rows().iter().map(|row| row.slots().len()).sum();
            assert_eq!(placed, algorithm.slots.len(), "{algorithm}");
            assert!(
                panel.rows().len() <= usize::from(grid.rows()),
                "{algorithm}"
            );

            for slot in algorithm.slots {
                let at = slot.position();
                assert!(at.column() < grid.columns(), "{algorithm} {}", slot.slot);
                assert!(at.row() < grid.rows(), "{algorithm} {}", slot.slot);

                // The row the panel says holds it is the row the slot claims.
                let row = panel
                    .rows()
                    .get(usize::from(at.row()))
                    .expect("a slot is on a row the panel has");
                assert_eq!(
                    row.slots().get(usize::from(at.column())).copied(),
                    Some(slot.slot),
                    "{algorithm} {} is not where its row puts it",
                    slot.slot
                );

                let (x, y) = at.centre();
                assert!(x > 0.0 && x < grid.display_width(), "{algorithm}");
                assert!(y > 0.0 && y < grid.display_height(), "{algorithm}");
            }
        }
    }

    /// Every algorithm has a panel of its own, which is the point of publishing
    /// them: a Room Reverb and a Phaser are different objects.
    #[test]
    fn each_algorithm_carries_its_own_measured_panel() {
        let mut faders = 0;
        for algorithm in Algorithm::all() {
            let panel = algorithm.panel();
            if panel.control() == super::Control::Fader {
                faders += 1;
            }
            // A measurement, not a placeholder: the four are not all one colour.
            let colours = [panel.chassis(), panel.face(), panel.cap(), panel.accent()];
            assert!(
                colours.iter().any(|colour| *colour != colours[0]),
                "{algorithm} is one flat colour"
            );
            assert!(
                panel.rows().iter().all(|row| row.align() == Align::Left),
                "{algorithm} was measured as left-aligned"
            );
        }
        assert_eq!(faders, 5, "five panels use faders, the manual's own count");

        let room = Algorithm::for_value(2, DEFAULT_FIRMWARE).expect("a Room Reverb");
        let phaser = Algorithm::for_value(30, DEFAULT_FIRMWARE).expect("a Phaser");
        assert_ne!(room.panel().face(), phaser.panel().face());
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn a_colour_prints_as_the_specification_writes_it() {
        let room = Algorithm::for_value(2, DEFAULT_FIRMWARE).expect("a Room Reverb");
        assert_eq!(alloc::format!("{}", room.panel().cap()), "#989388");
        assert_eq!(room.panel().cap().to_rgb(), [0x98, 0x93, 0x88]);
    }

    /// Every engine is reachable from the input and reaches the output, in all
    /// ten topologies. A slot fed by nothing would be a misread diagram.
    #[test]
    fn every_topology_is_wired_end_to_end() {
        assert_eq!(Routing::all().len(), ROUTING_COUNT);
        for routing in Routing::all() {
            assert_eq!(Routing::for_value(routing.value()), Some(routing));
            assert!(!routing.output().is_empty(), "{routing}");
            for engine in Engine::ALL {
                let feeds = routing.feeds(engine);
                assert!(!feeds.is_empty(), "{routing} feeds {engine} from nothing");
                assert!(
                    !feeds.contains(&Source::Engine(engine)),
                    "{routing}: {engine} feeds itself"
                );
            }
            // The input reaches the block through at least one engine.
            assert!(
                Engine::ALL
                    .into_iter()
                    .any(|engine| routing.feeds(engine).contains(&Source::Input)),
                "{routing} takes no input"
            );
        }
        let past = u8::try_from(ROUTING_COUNT).expect("ten topologies fit in a byte");
        assert_eq!(Routing::for_value(past), None);
        assert_eq!(Routing::parameter(), ParamId::FxRouting);
    }

    /// The two loops are declared rather than discovered, and the note says
    /// where each one taps.
    #[test]
    fn the_two_feedback_topologies_say_so() {
        assert!(
            Routing::all()
                .iter()
                .filter(|routing| routing.is_feedback())
                .map(Routing::label)
                .eq(["M-9", "M-10"]),
            "the feedback topologies are M-9 and M-10"
        );
        for routing in Routing::all() {
            assert_eq!(
                routing.is_feedback(),
                routing.note().is_some(),
                "{routing} has a loop and no note, or the other way round"
            );
        }
    }

    /// The topology is what says whether an engine's output gain reaches the
    /// output at all, which is the question the value table's name cannot
    /// answer.
    #[test]
    fn the_topology_says_which_engines_reach_the_output() {
        let serial = Routing::for_value(0).expect("M-1");
        assert_eq!(serial.output(), &[Engine::Four]);

        let parallel = Routing::for_value(3).expect("M-4");
        assert_eq!(parallel.output(), &Engine::ALL);
        for engine in Engine::ALL {
            assert_eq!(parallel.feeds(engine), &[Source::Input]);
        }
    }

    /// A true bypass takes the DSP out of circuit; a host matching on the value
    /// table's name would be re-tabulating an enum it was handed.
    #[test]
    fn every_mode_leaves_a_path_to_the_output() {
        assert_eq!(Mode::all().len(), MODE_COUNT);
        for mode in Mode::all() {
            assert_eq!(Mode::for_value(mode.value()), Some(mode));
            assert!(
                mode.analog_path() || mode.digital_path(),
                "{mode} is silent"
            );
        }
        let bypass = Mode::for_value(2).expect("a bypass");
        assert_eq!(bypass.name(), "Bypass");
        assert!(bypass.analog_path() && !bypass.digital_path());
        let past = u8::try_from(MODE_COUNT).expect("three modes fit in a byte");
        assert_eq!(Mode::for_value(past), None);
        assert_eq!(Mode::parameter(), ParamId::FxMode);
    }

    /// The names the two tables give have to be the same names, or a host
    /// drawing a list from one and a graph from the other draws two things.
    #[test]
    fn the_topologies_and_modes_match_their_value_tables() {
        for routing in Routing::all() {
            assert_eq!(
                ParamId::FxRouting.label(u16::from(routing.value())),
                Some(routing.name()),
            );
        }
        for mode in Mode::all() {
            assert_eq!(
                ParamId::FxMode.label(u16::from(mode.value())),
                Some(mode.name()),
            );
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

    /// Without the feature there is no prose to reach, and the accessor says so
    /// rather than the caller having to ask whether it was compiled in.
    #[cfg(not(feature = "descriptions"))]
    #[test]
    fn a_slot_describes_nothing_without_the_feature() {
        for algorithm in Algorithm::all() {
            for slot in algorithm.slots {
                assert_eq!(slot.description(), None, "{algorithm}");
            }
        }
    }

    /// The manual describes all but a couple of the slots, and the exceptions
    /// are `None` rather than a sentence this project wrote for it.
    #[cfg(feature = "descriptions")]
    #[test]
    fn a_slot_carries_the_manuals_own_words() {
        let room = Algorithm::for_value(2, DEFAULT_FIRMWARE).expect("a Room Reverb");
        assert_eq!(
            room.slot(3).and_then(super::FxSlot::description),
            Some("Controls the perceived size of the space being created by the reverb."),
        );

        let (slots, described) = Algorithm::all().iter().fold((0, 0), |(all, some), a| {
            (
                all + a.slots.len(),
                some + a
                    .slots
                    .iter()
                    .filter(|slot| slot.description().is_some())
                    .count(),
            )
        });
        assert_eq!(slots, 371);
        assert_eq!(described, 369);
    }

    /// Exactly three slots take an effect out of circuit, and they are the only
    /// slot-level off the instrument has. A fourth appearing here means either
    /// the specification grew one or `enable` has been read too widely.
    #[test]
    fn three_slots_switch_their_effect_out_of_circuit() {
        const EXPECTED: [(&str, u8); 3] = [("EdisonEX1", 1), ("NoiseGate", 8), ("Chorus-D", 1)];

        let mut found = [("", 0_u8); EXPECTED.len()];
        let mut count = 0_usize;
        for algorithm in Algorithm::all() {
            for slot in algorithm.slots.iter().filter(|slot| slot.is_enable()) {
                assert!(
                    count < EXPECTED.len(),
                    "a fourth enable: {algorithm} slot {}",
                    slot.slot
                );
                if let Some(entry) = found.get_mut(count) {
                    *entry = (algorithm.name, slot.slot);
                }
                count += 1;
            }
        }
        assert_eq!(count, EXPECTED.len());
        assert_eq!(found, EXPECTED);
    }

    /// An enable is a switch, because an effect is in circuit or it is not.
    /// The Noise Gate is the one that reads `ON` at the bottom of its range.
    #[test]
    fn an_enable_is_a_switch_and_may_read_either_way_round() {
        for algorithm in Algorithm::all() {
            for slot in algorithm.slots.iter().filter(|slot| slot.is_enable()) {
                assert_eq!(slot.kind, Kind::Switch, "{algorithm} {}", slot.title);
            }
        }

        let gate = Algorithm::by_name("NoiseGate").expect("a Noise Gate");
        let power = gate.slot(8).expect("a Power slot");
        assert_eq!((power.min, power.max), (Some("ON"), Some("OFF")));
    }

    /// Every algorithm reaches a mark, and the nine families between them
    /// account for all 35 with none left over.
    #[test]
    fn every_algorithm_has_a_family_and_a_mark() {
        for algorithm in Algorithm::all() {
            assert!(
                !algorithm.mark().strokes().is_empty(),
                "{algorithm} has an empty mark"
            );
            assert_eq!(algorithm.mark(), algorithm.family().mark());
        }

        let counted: usize = Family::ALL
            .into_iter()
            .map(|family| family.algorithms().count())
            .sum();
        assert_eq!(counted, ALGORITHM_COUNT);
    }

    /// `Family::ALL` is hand-written and the marks beside it are generated, so
    /// the two agree only as long as the order does. A family reordered or
    /// renamed in marks.toml fails here rather than handing out the wrong mark.
    #[test]
    fn the_families_are_in_the_order_the_specification_declares() {
        assert_eq!(
            Family::ALL.map(Family::name),
            [
                "Reverb",
                "Delay",
                "Modulation",
                "Filter",
                "Dynamics",
                "Distortion",
                "Imaging",
                "Pitch",
                "Rotary",
            ]
        );
        for (index, family) in Family::ALL.into_iter().enumerate() {
            assert_eq!(family.index(), index);
        }
    }

    /// Sine and cosine of an angle in radians, for the arc checks below.
    ///
    /// The crate's own `math` module takes turns rather than radians; these
    /// wrap it so the test reads the way the geometry does.
    fn sin(radians: f32) -> f32 {
        crate::math::sin_turns(radians / core::f32::consts::TAU)
    }

    fn cos(radians: f32) -> f32 {
        crate::math::cos_turns(radians / core::f32::consts::TAU)
    }

    /// A mark is drawn in a unit box, and a host that scales it by its own size
    /// expects nothing to land outside. The margin is what leaves room for a
    /// stroke width; `spec/marks.toml` states it and this holds it.
    #[test]
    fn a_mark_stays_inside_the_unit_box() {
        const MARGIN: f32 = 0.08;
        let inside = |x: f32, y: f32, family: Family| {
            assert!(
                (MARGIN..=1.0 - MARGIN).contains(&x) && (MARGIN..=1.0 - MARGIN).contains(&y),
                "{family} reaches ({x}, {y})"
            );
        };

        for family in Family::ALL {
            for stroke in family.mark().strokes() {
                match *stroke {
                    Stroke::Line { points } => {
                        assert!(points.len() >= 2, "{family} has a line of one point");
                        for point in points {
                            inside(point.x(), point.y(), family);
                        }
                    }
                    Stroke::Arc {
                        centre,
                        radius,
                        start,
                        sweep,
                    } => {
                        assert!(radius > 0.0, "{family} has an arc of no radius");
                        assert!(sweep.abs() > 0.0, "{family} has an arc that sweeps nothing");
                        // Only the swept part, because the circle an arc is
                        // taken from is allowed to reach outside the box.
                        for turns in [start, start + sweep] {
                            let radians = turns * core::f32::consts::TAU;
                            inside(
                                centre.x() + radius * cos(radians),
                                centre.y() + radius * sin(radians),
                                family,
                            );
                        }
                    }
                    Stroke::Dot { centre, radius } => {
                        assert!(radius > 0.0, "{family} has a dot of no radius");
                        inside(centre.x() - radius, centre.y() - radius, family);
                        inside(centre.x() + radius, centre.y() + radius, family);
                    }
                }
            }
        }
    }

    /// Every family has a pixel grid, it is the declared size, and it draws
    /// something. A mark that lit nothing, or everything, would blit as a blank
    /// or a block and tell a reader nothing.
    #[test]
    fn every_mark_has_a_pixel_grid_that_draws_something() {
        let side = u8::try_from(MARK_PIXEL_SIDE).expect("a small grid");
        for family in Family::ALL {
            let pixels = family.mark().pixels();
            assert_eq!(pixels.rows().len(), MARK_PIXEL_SIDE, "{family}");

            let mut lit = 0;
            for y in 0..side {
                for x in 0..side {
                    if pixels.is_lit(x, y) {
                        lit += 1;
                    }
                }
            }
            assert!(lit > 0, "{family} lights nothing");
            assert!(lit < MARK_PIXEL_SIDE * MARK_PIXEL_SIDE, "{family} is solid");

            // Nothing outside the grid is lit, and no row carries a bit past
            // its own width, which is what a host blitting a row relies on.
            assert!(
                !pixels.is_lit(side, 0),
                "{family} lights a column past the grid"
            );
            assert_eq!(pixels.row(side), 0, "{family} has a row past the grid");
            for y in 0..side {
                assert_eq!(
                    pixels.row(y) >> side,
                    0,
                    "{family} row {y} carries a bit past the grid"
                );
            }
        }
    }

    /// The nine grids are nine different pictures. Two families blitting the
    /// same pixels would be a symbol that cannot do its one job.
    #[test]
    fn no_two_families_blit_the_same_pixels() {
        for (index, family) in Family::ALL.into_iter().enumerate() {
            for other in Family::ALL.into_iter().skip(index + 1) {
                assert_ne!(
                    family.mark().pixels().rows(),
                    other.mark().pixels().rows(),
                    "{family} and {other} draw the same grid"
                );
            }
        }
    }

    /// The four buckets the manual prints are not the nine a symbol needs, and    /// The four buckets the manual prints are not the nine a symbol needs, and
    /// `Creative` is where that shows: it holds three families at once.
    #[test]
    fn a_family_is_finer_than_the_manuals_category() {
        let creative = |family: Family| {
            Algorithm::all()
                .iter()
                .any(|algorithm| algorithm.category == "Creative" && algorithm.family() == family)
        };
        assert!(creative(Family::Modulation));
        assert!(creative(Family::Filter));
        assert!(creative(Family::Pitch));
        assert!(creative(Family::Rotary));
    }

    /// Every slot answers what it does to a signal, and the answer is not just
    /// a rewording of what control to draw. A `Factor` is the case that proves
    /// it: drawn as a selector, and a time.
    #[test]
    fn a_quantity_is_a_different_question_from_a_control_kind() {
        let mut counts = [0_usize; Quantity::ALL.len()];
        for algorithm in Algorithm::all() {
            for slot in algorithm.slots {
                let count = counts
                    .get_mut(index_of(slot.quantity()))
                    .expect("a quantity is in Quantity::ALL");
                *count += 1;
            }
        }
        // Every one of the nine is used, or it should not be in the enum.
        for (quantity, count) in Quantity::ALL.into_iter().zip(counts) {
            assert!(count > 0, "{quantity:?} is in the enum and on no slot");
        }
        assert_eq!(counts.iter().sum::<usize>(), 371);

        let delay = Algorithm::by_name("3TapDelay").expect("a 3-Tap Delay");
        let factor = delay.slot(5).expect("a FactorA slot");
        assert!(factor.is_selector());
        assert_eq!(factor.quantity(), Quantity::Time);

        // And a switch is a switch both ways round, which is the easy case.
        let gate = Algorithm::by_name("NoiseGate").expect("a Noise Gate");
        let power = gate.slot(8).expect("a Power slot");
        assert_eq!(power.kind, Kind::Switch);
        assert_eq!(power.quantity(), Quantity::Switch);
    }

    /// Returns a quantity's position in `Quantity::ALL`.
    fn index_of(quantity: Quantity) -> usize {
        Quantity::ALL
            .into_iter()
            .position(|other| other == quantity)
            .unwrap_or(0)
    }

    /// A unit decides a quantity wherever the manual gives one, so every slot
    /// measured in the same unit answers the same way. A disagreement here is a
    /// hand edit that got out of step with the rest of the column.
    #[test]
    fn a_unit_decides_the_quantity_wherever_there_is_one() {
        for algorithm in Algorithm::all() {
            for slot in algorithm.slots {
                let expected = match slot.unit {
                    Some("ms" | "s") => Some(Quantity::Time),
                    Some("Hz") => Some(Quantity::Frequency),
                    Some("deg") => Some(Quantity::Position),
                    _ => None,
                };
                if let Some(expected) = expected {
                    assert_eq!(
                        slot.quantity(),
                        expected,
                        "{algorithm} {} is in {:?}",
                        slot.title,
                        slot.unit
                    );
                }
            }
        }
    }

    /// The fractions `response` places taps with are the ones the manual prints,
    /// held against the slot's own values so the table cannot drift from the
    /// specification it was copied out of.
    #[test]
    fn every_factor_slot_offers_the_same_ten_fractions() {
        let printed = super::FACTORS.map(|(_, name)| name);
        let mut found = 0;
        for algorithm in Algorithm::all() {
            for slot in algorithm.slots {
                if !slot.title.contains("Factor") {
                    continue;
                }
                assert_eq!(
                    slot.values,
                    printed.as_slice(),
                    "{algorithm} {}",
                    slot.title
                );
                assert_eq!(slot.quantity(), Quantity::Time);
                found += 1;
            }
        }
        // Every algorithm that divides a master delay time offers the same ten,
        // which is what makes one table right for all of them.
        assert_eq!(found, 10);
    }

    /// The two tap delays have a response and the other 33 do not, which is the
    /// answer rather than a gap.
    #[test]
    fn only_the_tap_delays_have_a_published_response() {
        let mut program = Program::new(ProtocolVersion::V6);
        for algorithm in Algorithm::all() {
            let Some(value) = algorithm.value_for(DEFAULT_FIRMWARE) else {
                continue;
            };
            program.set_clamped(Engine::One.algorithm_parameter(), value);
            let response = super::response(&program, Engine::One);
            let expected = matches!(algorithm.name, "3TapDelay" | "4TapDelay");
            assert_eq!(response.is_some(), expected, "{algorithm}");
        }
    }

    /// A tap lands where its factor puts it, against the master tap, and stands
    /// as tall as its gain byte. That is the whole of what a tap delay's
    /// picture is, and all of it comes off the manual's page bar the byte that
    /// picks the fraction.
    #[test]
    fn a_tap_lands_where_its_factor_and_gain_put_it() {
        let mut program = Program::new(ProtocolVersion::V6);
        program.set_fx1_type(FxType::FourTapDelay);
        // Every gain up, so each tap is there to be found.
        for slot in [2, 6, 8, 10] {
            program.set_clamped(Engine::One.slot_parameter(slot).expect("a gain slot"), 255);
        }
        // The last of the ten fractions is 3, the largest, so the fourth tap is
        // the furthest out and the picture is normalised to it.
        program.set_clamped(Engine::One.slot_parameter(9).expect("FactorC"), 255);
        // And the first two at 1/4, the smallest.
        for slot in [5, 7] {
            program.set_clamped(Engine::One.slot_parameter(slot).expect("a factor"), 0);
        }

        let response = super::response(&program, Engine::One).expect("a 4-Tap Delay");
        // Taps at 1, 1/4, 1/4 and 3 master times, over a furthest of 3.
        assert!(
            response.at(1.0) > 0.9,
            "the furthest tap: {}",
            response.at(1.0)
        );
        assert!(response.at(1.0 / 3.0) > 0.9, "the master tap");
        assert!(response.at(0.25 / 3.0) > 0.9, "the two quarter taps");
        // And nothing between them.
        assert!(response.at(0.6) <= 0.0, "{}", response.at(0.6));

        // A tap with its gain at nothing is drawn at nothing.
        program.set_clamped(Engine::One.slot_parameter(2).expect("the master gain"), 0);
        let quiet = super::response(&program, Engine::One).expect("a 4-Tap Delay");
        assert!(quiet.at(1.0 / 3.0) <= 0.0, "{}", quiet.at(1.0 / 3.0));
    }

    /// The `FX Type` table is 35 effects and nothing else, on both firmwares.
    /// This is the absence issue #30 asked to have stated somewhere a test can
    /// keep true: a value that stops naming an effect fails here.
    #[test]
    fn every_fx_type_value_is_an_effect() {
        for firmware in [FIRMWARE_1_0, DEFAULT_FIRMWARE] {
            let table = TableId::FxType.table_for(firmware);
            for (value, entry) in table.entries.iter().enumerate() {
                assert!(
                    Algorithm::by_name(entry.name).is_some(),
                    "{:?} names no algorithm",
                    entry.name
                );
                // Consecutive from zero, so there is no spare value sitting
                // beside the effects for an off to have been given.
                assert_eq!(u16::try_from(value), Ok(entry.value));
            }
        }

        // Firmware 1.1 offers every algorithm; 1.0 is the same table without
        // the Vintage Pitch it had not gained yet.
        let newest = TableId::FxType.table_for(DEFAULT_FIRMWARE);
        assert_eq!(newest.entries.len(), ALGORITHM_COUNT);
        assert_eq!(
            TableId::FxType.table_for(FIRMWARE_1_0).entries.len(),
            ALGORITHM_COUNT - 1
        );
    }
}
