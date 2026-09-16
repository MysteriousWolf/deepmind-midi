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

pub use generated::{ALGORITHM_COUNT, ENGINE_COUNT, MODE_COUNT, ROUTING_COUNT, SLOTS_PER_ENGINE};

use generated::{ALGORITHMS, ENGINES, GRID, MODES, PANELS, ROUTINGS};

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

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use super::{
        ALGORITHM_COUNT, Algorithm, Align, Engine, MODE_COUNT, Mode, ROUTING_COUNT, Routing,
        SLOTS_PER_ENGINE, Source, grid,
    };
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
}
