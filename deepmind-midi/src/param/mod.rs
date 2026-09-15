//! The parameter table: what each of the 242 program parameters is, and how it
//! reaches the wire.
//!
//! One number does two jobs on this synthesizer. A parameter's 14-bit NRPN
//! number is also its byte offset in an unpacked program dump, so [`ParamId`] is
//! both the thing you edit and the place it lives. Everything else here hangs
//! off that one identifier.
//!
//! ```
//! use deepmind_midi::param::{Kind, ParamId};
//! use deepmind_midi::wire::Channel;
//!
//! let shape = ParamId::Lfo1Shape;
//! assert_eq!(shape.offset(), 2);
//! assert_eq!(shape.name(), "LFO 1 Shape");
//! assert_eq!(shape.max(), 6);
//! assert_eq!(shape.label(3), Some("Ramp Up"));
//!
//! let mut bytes = [0; 12];
//! let len = shape.edit(3)?.encode_into(Channel::ONE, &mut bytes)?;
//! assert_eq!(
//!     &bytes[..len],
//!     &[0xB0, 99, 0, 0xB0, 98, 2, 0xB0, 6, 0, 0xB0, 38, 3],
//! );
//! # Ok::<(), deepmind_midi::Error>(())
//! ```
//!
//! # What the table carries
//!
//! What a host needs in order to act: the name the synthesizer shows, the
//! section of the panel it belongs to, the range it accepts, and the named
//! values it decodes into. That is [`Parameter`], reached through
//! [`ParamId::info`].
//!
//! It also carries what one parameter's value says about another, which the
//! modulation matrix is the whole of: a `Mod n Destination` holds an
//! abbreviation the display prints, and [`ParamId::targets`] is which parameters
//! that abbreviation moves. Empty where the destination names something no
//! program parameter addresses, which the specification says beside the entry.
//!
//! It carries what a raw value means beyond its range, where the manual says:
//! [`ParamId::shape`] is the point a bipolar parameter is read about,
//! [`ParamId::inactive`] the value that means "not set" rather than the
//! smallest one, and [`ParamId::bounded_by`] the parameter saying how many of a
//! run are played. Those three decide what a control *is*, so they are typed.
//!
//! The manual's own prose is reachable and costs nothing to ignore.
//! [`ParamId::note`], [`ParamId::display`] and [`ParamId::correction`] are each
//! a function over its own strings, referenced by nothing else, so a host that
//! never calls one does not carry it: the 11 kB of manual between them is
//! dropped by any linker collecting unreachable sections. Value-table entries
//! carry names and not descriptions, which is the same trade made the other
//! way — a description would sit inside a table every host already reaches.
//!
//! Nor does it carry conversions from a raw value to the number the synthesizer
//! displays. The manual publishes the two ends of a range and almost never the
//! curve between them, so [`ParamId::display`] hands over the ends as the
//! manual prints them and stops there: a reading invented between them would be
//! wrong in every host at once, and invisibly. Conversions arrive per parameter
//! as they are measured; see the scaling section of `docs/midi-spec.md`.
//!
//! # Firmware
//!
//! Firmware 1.1 renumbered three value tables rather than only appending to
//! them, so 17 of 23 modulation sources and 120 of 130 modulation destinations
//! mean something else on 1.0. Every lookup that touches a value table therefore
//! comes in two forms: a plain one that assumes [`DEFAULT_FIRMWARE`], and a
//! `_for` one that takes the [`Version`] a device inquiry reported. Nothing in a
//! stored dump says which firmware wrote it, so the older tables are reachable
//! only when the host knows the version another way.
//!
//! # Serde
//!
//! The tables are static: their strings are `&'static str` and their entries are
//! `&'static [ValueEntry]`. Under the `serde` feature they serialize, which is
//! what a host sending its parameter list somewhere needs, and they do not
//! deserialize, which would only allocate names the library already holds.

mod generated;

pub use generated::{
    CONTROLLER_COUNT, CONTROLLERS, DEFAULT_FIRMWARE, Group, PARAMETER_COUNT, ParamId, TABLE_COUNT,
    TableId,
};

use generated::{CONTROLLER_OF_PARAMETER, NO_CONTROLLER};

use core::fmt;

use crate::error::{Error, Result};
use crate::sysex::inquiry::Version;
use crate::wire::{Channel, ChannelMessage};

/// Controller that carries the high seven bits of an NRPN number (CC 99).
pub const NRPN_NUMBER_MSB: u8 = 99;

/// Controller that carries the low seven bits of an NRPN number (CC 98).
pub const NRPN_NUMBER_LSB: u8 = 98;

/// Controller that carries the high seven bits of a value (CC 6).
pub const DATA_ENTRY_MSB: u8 = 6;

/// Controller that carries the low seven bits of a value (CC 38).
pub const DATA_ENTRY_LSB: u8 = 38;

/// Everything the specification says about one parameter.
///
/// Returned by value from [`ParamId::info`]; the fields are also reachable one
/// at a time through the methods of the same names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
pub struct Parameter {
    /// Name as the synthesizer's display writes it.
    pub name: &'static str,
    /// Section of the instrument this parameter belongs to.
    pub group: Group,
    /// Lowest accepted value. Zero for every parameter but Program Transpose,
    /// which counts semitones either side of 128: -48 is 80 and +48 is 176.
    pub min: u16,
    /// Highest accepted value.
    pub max: u16,
    /// How the raw value is to be read.
    pub kind: Kind,
}

/// How a parameter's raw value is to be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Kind {
    /// A sweep. The raw value is the value, and what the synthesizer displays
    /// for it is a curve this library does not guess at.
    Continuous,
    /// Two states: off at zero, on at one.
    Switch,
    /// One of a named set, listed in the value table named here.
    Enumerated(TableId),
}

/// What a parameter's raw value means beyond the range it sits in.
///
/// [`Kind`] says how a byte is read — a sweep, two states, or one of a named
/// set. This says where the middle of a sweep is, which is a different question
/// and the one that decides what a control looks like: a bipolar value drawn
/// from the bottom of its range is a bar that is half full at no modulation, so
/// a matrix of eight depths set to nothing reads as a matrix of half of it.
///
/// Reached through [`ParamId::shape`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Shape {
    /// Counts up from [`min`](ParamId::min), which is 197 of the 242.
    Unipolar,
    /// Signed about a centre: `centre` reads as zero, and the two ends of the
    /// range as the largest readings either side of it.
    ///
    /// The ends are [`min`](ParamId::min) and [`max`](ParamId::max) as always,
    /// so what a reading is comes out as `value - centre` and needs nothing
    /// published here. They are deliberately not a single `±extent`: ten of the
    /// forty-five bipolar parameters run -128 to +127 about 128, which is one
    /// further below the centre than above it, and a symmetric extent would be
    /// a wrong answer for most of the set.
    Bipolar {
        /// The raw value that reads as zero.
        centre: u16,
    },
}

/// One value of an enumerated parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
pub struct ValueEntry {
    /// The value on the wire.
    pub value: u16,
    /// Name as the synthesizer displays it.
    pub name: &'static str,
    /// The program parameters this value names, where it names any.
    ///
    /// The modulation matrix is what this is for: a destination is an
    /// abbreviation the display prints, `VCF Freq` or `All Attack`, and this
    /// is the parameter table's own answer to what the abbreviation moves. A
    /// slice rather than one parameter because several destinations plainly
    /// move more than one, and empty where the destination names something no
    /// program parameter addresses — the pitch a key is playing, the amplitude
    /// of a voice — which is more honest than a wrong single answer.
    ///
    /// Empty for every other table, and for a mapping that has not been
    /// established. `docs/midi-spec.md` prints the reason beside the
    /// destinations that have one.
    pub parameters: &'static [ParamId],
}

/// A named set of values, as of one firmware version.
///
/// Reached through [`TableId::table`] or [`TableId::table_for`]. Several tables
/// may share an identifier when firmware renumbered their entries; those two
/// methods are what pick between them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
pub struct ValueTable {
    /// Identifier this table answers to.
    pub id: TableId,
    /// Human-readable table name.
    pub name: &'static str,
    /// `false` where the manual contradicts itself and hardware has not settled
    /// it. The entries are recorded as printed.
    pub confirmed: bool,
    /// `true` when [`ValueTable::entries`] lists only the start of a documented
    /// run, such as `SPREAD-1` standing for `SPREAD-1` through `SPREAD-254`.
    /// [`ValueTable::name_of`] answers `None` above the last listed entry rather
    /// than inventing the rest of the run.
    pub partial: bool,
    /// The values themselves, in value order.
    pub entries: &'static [ValueEntry],
}

impl ValueTable {
    /// Returns the name this table gives `value`.
    ///
    /// `None` when the table does not list it, which for a
    /// [`partial`](ValueTable::partial) table includes values inside a
    /// documented run whose first entry is listed.
    #[must_use]
    pub fn name_of(&self, value: u16) -> Option<&'static str> {
        self.entry(value).map(|entry| entry.name)
    }

    /// Returns the program parameters this table's `value` names.
    ///
    /// Empty both for a value this table does not list and for one that names
    /// nothing a program parameter addresses; see
    /// [`ValueEntry::parameters`] for what the difference is and where it is
    /// written down.
    #[must_use]
    pub fn parameters_of(&self, value: u16) -> &'static [ParamId] {
        self.entry(value).map_or(&[], |entry| entry.parameters)
    }

    /// Returns this table's entry for `value`.
    ///
    /// A binary search: the entries are in ascending value order, which the
    /// specification checks when it loads, and the largest table has 133 of
    /// them. It runs once per control a panel redraws.
    #[must_use]
    pub fn entry(&self, value: u16) -> Option<&'static ValueEntry> {
        let index = self
            .entries
            .binary_search_by_key(&value, |entry| entry.value)
            .ok()?;
        self.entries.get(index)
    }
}

impl fmt::Display for ValueTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name)
    }
}

/// What a MIDI continuous controller reaches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum ControllerKind {
    /// Drives one program parameter, named in [`Controller::parameter`].
    Parameter,
    /// A controller the MIDI specification defines, used as it is defined.
    Standard,
    /// Something else the synthesizer answers, such as an NRPN selector.
    Other,
}

/// A MIDI continuous controller the synthesizer answers.
///
/// Controllers are a shortcut, not a second address space: 90 of the 242
/// parameters have one, and a parameter whose range runs past 127 loses
/// resolution through it. NRPN is the general path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
pub struct Controller {
    /// Controller number, `0..=127`.
    pub cc: u8,
    /// What it controls.
    pub name: &'static str,
    /// What kind of thing that is.
    pub kind: ControllerKind,
    /// The program parameter it drives, where it drives one.
    pub parameter: Option<ParamId>,
}

impl Controller {
    /// Returns the controller with this number, if the synthesizer answers it.
    ///
    /// The table is in controller number order, so this is a binary search;
    /// it runs on every control change a port delivers.
    #[must_use]
    pub fn for_cc(cc: u8) -> Option<&'static Self> {
        CONTROLLERS
            .binary_search_by_key(&cc, |controller| controller.cc)
            .ok()
            .and_then(|index| CONTROLLERS.get(index))
    }

    /// Returns the controller that drives this parameter, if one does.
    ///
    /// A table indexed by the parameter's own offset rather than a scan over
    /// the 128 controllers; a panel asks it once per control it draws. No two
    /// controllers claim the same parameter, which the specification checks
    /// when it loads and which is what makes the table one entry wide.
    #[must_use]
    pub fn for_parameter(parameter: ParamId) -> Option<&'static Self> {
        let index = CONTROLLER_OF_PARAMETER
            .get(usize::from(parameter.offset()))
            .copied()
            .filter(|index| *index != NO_CONTROLLER)?;
        CONTROLLERS.get(usize::from(index))
    }
}

impl fmt::Display for Controller {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CC {} ({})", self.cc, self.name)
    }
}

impl TryFrom<u8> for ParamId {
    type Error = Error;

    fn try_from(offset: u8) -> Result<Self> {
        Self::from_offset(offset)
    }
}

impl From<ParamId> for u8 {
    fn from(parameter: ParamId) -> Self {
        parameter.offset()
    }
}

impl ParamId {
    /// Returns the NRPN number, which is also the byte offset in a program dump.
    #[must_use]
    pub const fn offset(self) -> u8 {
        self as u8
    }

    /// Returns the parameter at this offset.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ParameterOutOfRange`] for an offset of 242 or more.
    /// Offsets 242 to 244 exist in a comms protocol version 7 dump; they are
    /// reserved, carry no parameter, and are preserved rather than decoded.
    pub fn from_offset(offset: u8) -> Result<Self> {
        Self::ALL
            .get(usize::from(offset))
            .copied()
            .ok_or(Error::ParameterOutOfRange(offset))
    }

    /// Returns the name the synthesizer's display writes.
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.info().name
    }

    /// Returns the section of the instrument this parameter belongs to.
    #[must_use]
    pub const fn group(self) -> Group {
        self.info().group
    }

    /// Returns the name without the group's, where the name starts with it.
    ///
    /// `VCF Envelope Depth` is `Envelope Depth`, because a panel prints its
    /// group once as a heading and repeating it in every slot costs the width
    /// the rest of the name needs. A name that does not start with its group is
    /// returned whole, which is every parameter of the modulation matrix and of
    /// the arpeggiator.
    ///
    /// ```
    /// use deepmind_midi::param::ParamId;
    ///
    /// assert_eq!(ParamId::VcfEnvelopeDepth.short_name(), "Envelope Depth");
    /// assert_eq!(ParamId::Mod1Source.short_name(), "Mod 1 Source");
    /// ```
    #[must_use]
    pub fn short_name(self) -> &'static str {
        let name = self.name();
        name.strip_prefix(self.group().name())
            .map_or(name, |rest| rest.trim_start())
    }

    /// Returns how the raw value is to be read.
    #[must_use]
    pub const fn kind(self) -> Kind {
        self.info().kind
    }

    /// Returns the lowest accepted value.
    ///
    /// Zero for 241 of the 242 parameters; Program Transpose is the exception,
    /// running 80 to 176 for -48 to +48 semitones.
    #[must_use]
    pub const fn min(self) -> u16 {
        self.info().min
    }

    /// Returns the highest accepted value.
    #[must_use]
    pub const fn max(self) -> u16 {
        self.info().max
    }

    /// Returns whether this parameter accepts `value`.
    #[must_use]
    pub const fn accepts(self, value: u16) -> bool {
        let info = self.info();
        value >= info.min && value <= info.max
    }

    /// Returns `value` brought inside the accepted range.
    #[must_use]
    pub const fn clamp(self, value: u16) -> u16 {
        let info = self.info();
        if value < info.min {
            info.min
        } else if value > info.max {
            info.max
        } else {
            value
        }
    }

    /// Returns the name this parameter gives `value` on [`DEFAULT_FIRMWARE`].
    ///
    /// `None` for a continuous parameter, whose value has no name, and for a
    /// value its table does not list.
    #[must_use]
    pub fn label(self, value: u16) -> Option<&'static str> {
        self.label_for(value, DEFAULT_FIRMWARE)
    }

    /// Returns the name this parameter gives `value` on the firmware a device
    /// inquiry reported.
    ///
    /// Worth reaching for whenever the value came from a stored program: the
    /// three renumbered tables mean a 1.0 program read with 1.1 tables is
    /// mislabelled almost throughout.
    #[must_use]
    pub fn label_for(self, value: u16, firmware: Version) -> Option<&'static str> {
        match self.kind() {
            Kind::Continuous => None,
            Kind::Switch => match value {
                0 => Some("Off"),
                1 => Some("On"),
                _ => None,
            },
            Kind::Enumerated(table) => table.table_for(firmware).name_of(value),
        }
    }

    /// Returns the named values this parameter accepts, on
    /// [`DEFAULT_FIRMWARE`].
    ///
    /// `None` unless the parameter's table names **every** value it accepts,
    /// which is the question a host has to answer before drawing a list.
    /// A table that names some of them would make a control that silently
    /// dropped the rest: opening the list on an unnamed value and picking the
    /// nearest name is a control that moves the sound when somebody looks at
    /// it. Where this answers `None` the raw value is the honest reading.
    ///
    /// Also `None` for a switch, whose two values [`label`](ParamId::label)
    /// names and which a host draws as a light rather than a list.
    ///
    /// ```
    /// use deepmind_midi::param::ParamId;
    ///
    /// let shapes = ParamId::Lfo1Shape.choices().expect("all seven are named");
    /// assert_eq!(shapes.len(), 7);
    /// assert_eq!(shapes.first().map(|entry| entry.name), Some("Sine"));
    ///
    /// // A table that only writes down the start of a documented run cannot
    /// // name the rest of it, and says so rather than guessing.
    /// assert!(ParamId::Lfo1MonoMode.choices().is_none());
    /// assert!(ParamId::Lfo1Rate.choices().is_none());
    /// ```
    #[must_use]
    pub fn choices(self) -> Option<&'static [ValueEntry]> {
        self.choices_for(DEFAULT_FIRMWARE)
    }

    /// Returns the named values this parameter accepts, on the firmware a
    /// device inquiry reported.
    ///
    /// Worth reaching for wherever [`label_for`](ParamId::label_for) is: a list
    /// drawn from the wrong firmware's table is a list of the wrong names.
    #[must_use]
    pub fn choices_for(self, firmware: Version) -> Option<&'static [ValueEntry]> {
        let Kind::Enumerated(table) = self.kind() else {
            return None;
        };
        let entries = table.table_for(firmware).entries;
        // The entries are in value order, so the ones this parameter accepts
        // are one run of them. Counting the run and comparing it with the range
        // is what says the table covers the range rather than part of it.
        let first = entries.iter().position(|entry| self.accepts(entry.value))?;
        let run = entries.get(first..)?;
        let named = run
            .iter()
            .take_while(|entry| self.accepts(entry.value))
            .count();
        let span = usize::from(self.max() - self.min()) + 1;
        if named != span {
            return None;
        }
        run.get(..named)
    }

    /// Returns the controller that drives this parameter, if one does.
    #[must_use]
    pub fn controller(self) -> Option<&'static Controller> {
        Controller::for_parameter(self)
    }

    /// Returns the program parameters this parameter's `value` names, on
    /// [`DEFAULT_FIRMWARE`].
    ///
    /// The modulation matrix is what has an answer: a `Mod n Destination`
    /// holding 20 reads `VCF Freq` on the display and moves
    /// [`ParamId::VcfFrequency`], and a host drawing the matrix beside the
    /// panels is the caller. Empty for every parameter whose values name no
    /// other parameter, and for a destination that names something no program
    /// parameter addresses.
    ///
    /// ```
    /// use deepmind_midi::param::ParamId;
    ///
    /// assert_eq!(
    ///     ParamId::Mod3Destination.targets(20),
    ///     &[ParamId::VcfFrequency],
    /// );
    /// assert!(ParamId::Mod3Destination.targets(0).is_empty());   // Off
    /// assert!(ParamId::Lfo1Rate.targets(64).is_empty());         // a sweep
    /// ```
    #[must_use]
    pub fn targets(self, value: u16) -> &'static [ParamId] {
        self.targets_for(value, DEFAULT_FIRMWARE)
    }

    /// Returns the program parameters this parameter's `value` names, on the
    /// firmware a device inquiry reported.
    ///
    /// Worth reaching for wherever [`label_for`](ParamId::label_for) is, and
    /// for the same reason: firmware 1.1 renumbered the destination table, so
    /// 120 of its 130 entries mean something else on 1.0.
    #[must_use]
    pub fn targets_for(self, value: u16, firmware: Version) -> &'static [ParamId] {
        match self.kind() {
            Kind::Continuous | Kind::Switch => &[],
            Kind::Enumerated(table) => table.table_for(firmware).parameters_of(value),
        }
    }

    /// Builds an NRPN edit setting this parameter to `value`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ValueOutOfRange`] when the parameter does not accept the
    /// value. [`ParamId::clamp`] is there for callers that would rather bend the
    /// value than refuse it.
    pub const fn edit(self, value: u16) -> Result<NrpnEdit> {
        NrpnEdit::new(self, value)
    }

    /// Returns `raw` scaled down to the seven bits a control change carries.
    ///
    /// The manual's rule is the value over its maximum, in 127ths. Here it is
    /// the value over its *range*, which is the same arithmetic for every
    /// parameter that starts at zero (all 90 that have a controller) and the
    /// only reading that means anything for one that does not.
    ///
    /// Lossy for a parameter whose range runs past 127, which is most of them.
    #[must_use]
    pub const fn to_cc_value(self, raw: u16) -> u8 {
        let info = self.info();
        let span = info.max - info.min;
        if span == 0 {
            return 0;
        }
        let scaled = ((self.clamp(raw) - info.min) as u32 * 127) / span as u32;
        // The numerator is at most `span * 127`, so the quotient is at most 127.
        (scaled & 0x7F) as u8
    }

    /// Returns the raw value a control change of `value` sets.
    ///
    /// The inverse of [`ParamId::to_cc_value`] at both ends, and not a round
    /// trip between them: a controller reaches 128 of the 256 values an NRPN can
    /// address.
    #[must_use]
    pub const fn from_cc_value(self, value: u8) -> u16 {
        let info = self.info();
        let span = (info.max - info.min) as u32;
        let raw = info.min as u32 + ((value & 0x7F) as u32 * span) / 127;
        // `value` is seven bits, so the quotient is at most `span`, and
        // `min + span` is `max`, which is a u16.
        self.clamp((raw & 0xFFFF) as u16)
    }
}

impl fmt::Display for ParamId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl Group {
    /// Returns the parameters in this group, in offset order.
    ///
    /// Allocates nothing: it is a filter over [`ParamId::ALL`].
    pub fn parameters(self) -> impl Iterator<Item = ParamId> {
        ParamId::ALL
            .iter()
            .copied()
            .filter(move |parameter| parameter.group() == self)
    }
}

impl fmt::Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl TableId {
    /// Returns this table as of [`DEFAULT_FIRMWARE`].
    #[must_use]
    pub const fn table(self) -> &'static ValueTable {
        self.table_for(DEFAULT_FIRMWARE)
    }
}

impl fmt::Display for TableId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.table().name)
    }
}

/// One parameter edit, as the four control changes that carry it.
///
/// The synthesizer keeps a selected-parameter register per interface, so a host
/// sweeping one control can send the selection once and then data entry pairs
/// alone. This type always sends all four, which is correct in every order and
/// after any other traffic; a host that wants the shorter form knows when it is
/// safe and can send the pairs itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NrpnEdit {
    /// The parameter being set.
    pub parameter: ParamId,
    /// The value it is being set to, inside the parameter's range.
    pub value: u16,
}

impl NrpnEdit {
    /// Bytes an edit encodes to, without running status.
    pub const LEN: usize = 12;

    /// Builds an edit.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ValueOutOfRange`] when the parameter does not accept the
    /// value.
    pub const fn new(parameter: ParamId, value: u16) -> Result<Self> {
        if parameter.accepts(value) {
            Ok(Self { parameter, value })
        } else {
            Err(Error::ValueOutOfRange {
                parameter: parameter.offset(),
                value,
                max: parameter.max(),
            })
        }
    }

    /// Returns the four controller and value pairs, in the order they are sent.
    ///
    /// The NRPN number needs both of its bytes: 114 of the 242 parameters sit
    /// past offset 127, where the number's high byte stops being zero.
    ///
    /// The value is split across data entry MSB and LSB even where the
    /// parameter's range would fit in the LSB alone. Sending both costs three
    /// bytes and cannot be misread.
    #[must_use]
    pub const fn controls(self) -> [(u8, u8); 4] {
        let offset = self.parameter.offset();
        [
            (NRPN_NUMBER_MSB, (offset >> 7) & 0x7F),
            (NRPN_NUMBER_LSB, offset & 0x7F),
            (DATA_ENTRY_MSB, ((self.value >> 7) & 0x7F) as u8),
            (DATA_ENTRY_LSB, (self.value & 0x7F) as u8),
        ]
    }

    /// Writes the edit into `out` and returns how many bytes it used.
    ///
    /// # Errors
    ///
    /// Returns [`Error::BufferTooSmall`] for a buffer shorter than
    /// [`NrpnEdit::LEN`].
    pub fn encode_into(self, channel: Channel, out: &mut [u8]) -> Result<usize> {
        if out.len() < Self::LEN {
            return Err(Error::BufferTooSmall {
                needed: Self::LEN,
                available: out.len(),
            });
        }
        let mut written = 0;
        for (slot, (controller, value)) in out.chunks_exact_mut(3).zip(self.controls()) {
            written +=
                ChannelMessage::ControlChange { controller, value }.encode_into(channel, slot)?;
        }
        Ok(written)
    }
}

impl fmt::Display for NrpnEdit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.parameter.label(self.value) {
            Some(label) => write!(f, "{} = {label}", self.parameter),
            None => write!(f, "{} = {}", self.parameter, self.value),
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
    use crate::ids::ProtocolVersion;

    /// Firmware 1.0, the version the manual's NRPN appendix describes.
    const FIRMWARE_1_0: Version = Version { major: 1, minor: 0 };

    #[test]
    fn every_offset_names_exactly_one_parameter() {
        for (index, parameter) in ParamId::ALL.iter().copied().enumerate() {
            let offset = u8::try_from(index).expect("242 offsets fit in a byte");
            assert_eq!(parameter.offset(), offset);
            assert_eq!(ParamId::from_offset(offset), Ok(parameter));
        }
        assert_eq!(
            ParamId::from_offset(242),
            Err(Error::ParameterOutOfRange(242))
        );
    }

    /// The parameter table is the dump layout: one parameter per byte of a
    /// comms protocol version 6 program, with nothing left over.
    #[test]
    fn the_table_is_as_long_as_a_program_dump() {
        assert_eq!(
            ProtocolVersion::V6.program_data_len(),
            PARAMETER_COUNT,
            "every byte of a program is a parameter"
        );
    }

    #[test]
    fn every_parameter_belongs_to_exactly_one_group() {
        let counted: usize = Group::ALL
            .iter()
            .map(|group| group.parameters().count())
            .sum();
        assert_eq!(counted, PARAMETER_COUNT);
        assert_eq!(ParamId::Lfo1Rate.group(), Group::Lfo1);
        assert_eq!(Group::Lfo1.name(), "LFO 1");
    }

    #[test]
    fn an_nrpn_edit_is_four_control_changes() {
        let mut bytes = [0; NrpnEdit::LEN];
        let len = ParamId::Lfo1Shape
            .edit(3)
            .expect("3 is an LFO shape")
            .encode_into(Channel::ONE, &mut bytes)
            .expect("the buffer is long enough");
        assert_eq!(len, NrpnEdit::LEN);
        assert_eq!(
            bytes,
            [0xB0, 99, 0, 0xB0, 98, 2, 0xB0, 6, 0, 0xB0, 38, 3],
            "select parameter 2, then set it to 3"
        );
    }

    /// 114 parameters sit past offset 127, where the NRPN number needs its high
    /// byte. Zeroing it would edit a different parameter, quietly.
    #[test]
    fn a_parameter_past_offset_127_sets_the_number_high_byte() {
        let parameter = ParamId::from_offset(200).expect("200 is a parameter");
        let edit = parameter.edit(0).expect("zero is in range");
        assert_eq!(edit.controls()[0], (NRPN_NUMBER_MSB, 1));
        assert_eq!(edit.controls()[1], (NRPN_NUMBER_LSB, 72));
    }

    /// A value above 127 needs both data entry bytes, which is most of them:
    /// most parameters run to 255.
    #[test]
    fn a_value_past_127_sets_both_data_entry_bytes() {
        let edit = ParamId::Lfo1Rate.edit(200).expect("200 is in range");
        assert_eq!(edit.controls()[2], (DATA_ENTRY_MSB, 1));
        assert_eq!(edit.controls()[3], (DATA_ENTRY_LSB, 72));
    }

    #[test]
    fn a_value_outside_the_range_is_refused_rather_than_sent() {
        assert_eq!(
            ParamId::Lfo1Shape.edit(7),
            Err(Error::ValueOutOfRange {
                parameter: 2,
                value: 7,
                max: 6,
            })
        );
        assert_eq!(ParamId::Lfo1Shape.clamp(7), 6);
        assert!(ParamId::Lfo1Shape.accepts(6));
    }

    #[test]
    fn a_short_buffer_is_refused_rather_than_half_filled() {
        let mut bytes = [0; NrpnEdit::LEN - 1];
        assert_eq!(
            ParamId::Lfo1Rate
                .edit(0)
                .expect("zero is in range")
                .encode_into(Channel::ONE, &mut bytes),
            Err(Error::BufferTooSmall {
                needed: NrpnEdit::LEN,
                available: NrpnEdit::LEN - 1,
            })
        );
        assert_eq!(bytes, [0; NrpnEdit::LEN - 1]);
    }

    #[test]
    fn switches_and_value_tables_name_their_values_and_sweeps_do_not() {
        assert_eq!(ParamId::Lfo1KeySync.kind(), Kind::Switch);
        assert_eq!(ParamId::Lfo1KeySync.label(0), Some("Off"));
        assert_eq!(ParamId::Lfo1KeySync.label(1), Some("On"));
        assert_eq!(ParamId::Lfo1KeySync.label(2), None);

        assert_eq!(
            ParamId::Lfo1Shape.kind(),
            Kind::Enumerated(TableId::LfoShape)
        );
        assert_eq!(ParamId::Lfo1Shape.label(6), Some("Sample & Glide"));

        assert_eq!(ParamId::Lfo1Rate.kind(), Kind::Continuous);
        assert_eq!(ParamId::Lfo1Rate.label(64), None);
    }

    /// Every value a confirmed, complete table covers has a name. A gap would
    /// leave a host with a value it can address and cannot display.
    #[test]
    fn a_complete_value_table_names_every_value_its_parameter_accepts() {
        for parameter in ParamId::ALL.iter().copied() {
            let Kind::Enumerated(id) = parameter.kind() else {
                continue;
            };
            let table = id.table();
            if table.partial || !table.confirmed {
                continue;
            }
            for value in parameter.min()..=parameter.max() {
                assert!(
                    parameter.label(value).is_some(),
                    "{parameter} has no name for {value} in {table}"
                );
            }
        }
    }

    /// The renumbering that makes firmware a lookup key rather than a footnote:
    /// modulation source 6 is Expression on 1.1 and LFO 1 on 1.0.
    #[test]
    fn the_firmware_a_program_was_written_on_changes_what_its_values_mean() {
        assert_eq!(ParamId::Mod1Source.label(6), Some("Expression"));
        assert_eq!(ParamId::Mod1Source.label_for(6, FIRMWARE_1_0), Some("LFO1"));
        assert_eq!(TableId::ModSource.table_for(FIRMWARE_1_0).entries.len(), 23);
        assert_eq!(TableId::ModSource.table().entries.len(), 25);
    }

    /// The join the modulation matrix needs: a destination is an abbreviation
    /// the display prints, and this is what it addresses.
    #[test]
    fn a_modulation_destination_names_the_parameters_it_moves() {
        assert_eq!(ParamId::Mod3Destination.label(20), Some("VCF Freq"));
        assert_eq!(
            ParamId::Mod3Destination.targets(20),
            &[ParamId::VcfFrequency]
        );

        // Several destinations plainly move more than one, which is why this
        // is a slice.
        let attacks = ParamId::Mod1Destination.targets(25);
        assert_eq!(ParamId::Mod1Destination.label(25), Some("All Attack"));
        assert_eq!(attacks.len(), 3);
        assert!(attacks.contains(&ParamId::VcfEnvelopeAttackTime));
    }

    /// Empty is an answer here, and it means two different things: a
    /// destination that addresses nothing a program holds, and a parameter
    /// whose values name no parameter at all.
    #[test]
    fn a_destination_that_addresses_nothing_says_so_by_naming_nothing() {
        assert!(ParamId::Mod1Destination.targets(0).is_empty(), "Off");
        // Oscillator pitch is played rather than stored.
        assert_eq!(ParamId::Mod1Destination.label(11), Some("OSC1 Pitch"));
        assert!(ParamId::Mod1Destination.targets(11).is_empty());

        assert!(ParamId::Lfo1Rate.targets(64).is_empty(), "a sweep");
        assert!(ParamId::Lfo1KeySync.targets(1).is_empty(), "a switch");
        assert!(ParamId::Lfo1Shape.targets(3).is_empty(), "a named set");
    }

    /// The renumbering reaches the join as well as the names: destination 20 is
    /// the filter cutoff on 1.1 and the filter's LFO depth on 1.0.
    #[test]
    fn the_firmware_moves_what_a_destination_addresses() {
        assert_eq!(
            ParamId::Mod1Destination.targets(20),
            &[ParamId::VcfFrequency]
        );
        assert_eq!(
            ParamId::Mod1Destination.targets_for(20, FIRMWARE_1_0),
            &[ParamId::VcfLfoDepth]
        );
        assert_eq!(
            ParamId::Mod1Destination.targets_for(17, FIRMWARE_1_0),
            &[ParamId::VcfFrequency]
        );
    }

    /// Every FX slot a destination names is an FX slot parameter, which is what
    /// lets a host label the 48 bytes through the panel tables.
    #[test]
    fn every_effect_slot_is_reachable_from_the_matrix() {
        let table = TableId::ModDestination.table();
        let slots: Vec<ParamId> = table
            .entries
            .iter()
            .flat_map(|entry| entry.parameters.iter().copied())
            .filter(|parameter| parameter.group() == Group::Effects)
            .collect();
        // Four engines of twelve slots, and the four output gains.
        assert_eq!(slots.len(), 4 * 12 + 4);
    }

    /// A destination naming a parameter twice, or naming one that does not
    /// exist, is caught in the specification rather than here; what this
    /// checks is that nothing generated a name the table cannot resolve.
    #[test]
    fn every_parameter_a_table_names_is_a_parameter() {
        for id in TableId::ALL {
            for entry in id.table().entries {
                for parameter in entry.parameters {
                    assert_eq!(
                        ParamId::from_offset(parameter.offset()),
                        Ok(*parameter),
                        "{entry:?} in {id}"
                    );
                }
            }
        }
    }

    /// The order the panel is in, which a host would otherwise read off the
    /// offsets itself. `ALL` is alphabetical and is not that order.
    #[test]
    fn the_groups_are_listed_twice_in_two_different_orders() {
        assert_eq!(Group::ORDER.len(), Group::ALL.len());
        for group in Group::ALL {
            assert!(Group::ORDER.contains(group), "{group} is not laid out");
        }

        let starts: Vec<u8> = Group::ORDER
            .iter()
            .map(|group| {
                group
                    .parameters()
                    .next()
                    .expect("a group in the table has a parameter")
                    .offset()
            })
            .collect();
        let mut sorted = starts.clone();
        sorted.sort_unstable();
        assert_eq!(starts, sorted, "the order is not the instrument's own");
        assert_eq!(Group::ORDER.first(), Some(&Group::Lfo1));
        assert_eq!(Group::ORDER.last(), Some(&Group::Program));
    }

    /// A list is only honest where the table names every value the parameter
    /// accepts; anywhere else the raw number is the reading.
    #[test]
    fn a_parameter_offers_a_list_only_when_every_value_has_a_name() {
        let shapes = ParamId::Lfo1Shape.choices().expect("all seven are named");
        assert_eq!(shapes.len(), usize::from(ParamId::Lfo1Shape.max()) + 1);
        assert_eq!(shapes.first().map(|entry| entry.name), Some("Sine"));
        assert_eq!(
            shapes.last().map(|entry| entry.name),
            Some("Sample & Glide")
        );

        // A table that writes down the start of a documented run names the
        // first of 254 values and not the rest.
        assert!(TableId::LfoMonoMode.table().partial);
        assert!(ParamId::Lfo1MonoMode.choices().is_none());

        assert!(ParamId::Lfo1Rate.choices().is_none(), "a sweep");
        assert!(ParamId::Lfo1KeySync.choices().is_none(), "a switch");
    }

    /// Whatever a list offers has a name, which is what makes it safe to draw
    /// as one: a host picking an entry never picks an unnamed value.
    #[test]
    fn every_value_a_list_offers_is_named_and_accepted() {
        for firmware in [DEFAULT_FIRMWARE, FIRMWARE_1_0] {
            for parameter in ParamId::ALL.iter().copied() {
                let Some(choices) = parameter.choices_for(firmware) else {
                    continue;
                };
                for entry in choices {
                    assert!(parameter.accepts(entry.value), "{parameter}");
                    assert_eq!(
                        parameter.label_for(entry.value, firmware),
                        Some(entry.name),
                        "{parameter}"
                    );
                }
                for value in parameter.min()..=parameter.max() {
                    assert!(
                        choices.iter().any(|entry| entry.value == value),
                        "{parameter} offers a list that skips {value}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_parameter_name_drops_the_group_a_panel_already_prints() {
        assert_eq!(ParamId::VcfEnvelopeDepth.short_name(), "Envelope Depth");
        assert_eq!(ParamId::Lfo1Rate.short_name(), "Rate");
        assert_eq!(ParamId::ProgramNameChar1.short_name(), "Name Char 1");
        // A name that does not start with its group keeps all of itself.
        assert_eq!(ParamId::Mod1Source.short_name(), "Mod 1 Source");

        for parameter in ParamId::ALL.iter().copied() {
            let short = parameter.short_name();
            assert!(!short.is_empty(), "{parameter} shortens to nothing");
            assert!(parameter.name().ends_with(short), "{parameter}");
        }
    }

    #[test]
    fn a_table_answers_only_for_the_values_it_lists() {
        let partial = TableId::LfoMonoMode.table();
        assert!(partial.partial, "the SPREAD-n run is not written out");
        assert_eq!(partial.name_of(1), Some("Mono"));
        assert_eq!(partial.name_of(200), None);
    }

    /// `for_cc` is a binary search, which the table's order has to allow.
    #[test]
    fn the_controller_table_is_in_controller_number_order() {
        assert!(CONTROLLERS.windows(2).all(|pair| match pair {
            [a, b] => a.cc < b.cc,
            _ => true,
        }));
        assert_eq!(Controller::for_cc(255), None);
    }

    #[test]
    fn controllers_and_parameters_agree_in_both_directions() {
        for controller in &CONTROLLERS {
            assert_eq!(Controller::for_cc(controller.cc), Some(controller));
            let Some(parameter) = controller.parameter else {
                assert_ne!(controller.kind, ControllerKind::Parameter);
                continue;
            };
            assert_eq!(controller.kind, ControllerKind::Parameter);
            assert_eq!(parameter.controller(), Some(controller));
        }
        assert_eq!(Controller::for_cc(3), None, "CC 3 is undefined");
    }

    /// A controller carries seven bits, so it reaches 128 of the 256 values an
    /// NRPN can address. The two ends have to land exactly even so.
    #[test]
    fn a_control_change_reaches_the_ends_of_the_range_it_scales() {
        for parameter in ParamId::ALL.iter().copied() {
            assert_eq!(parameter.to_cc_value(parameter.min()), 0, "{parameter}");
            assert_eq!(parameter.to_cc_value(parameter.max()), 127, "{parameter}");
            assert_eq!(parameter.from_cc_value(0), parameter.min(), "{parameter}");
            assert_eq!(parameter.from_cc_value(127), parameter.max(), "{parameter}");
        }
    }

    /// A parameter that fits in seven bits round trips through a controller;
    /// one that does not loses the values between.
    #[test]
    fn scaling_is_lossless_only_where_the_range_fits_in_seven_bits() {
        let fine = ParamId::ProgramNameChar1;
        assert_eq!(fine.max(), 127);
        for raw in 0..=fine.max() {
            assert_eq!(fine.from_cc_value(fine.to_cc_value(raw)), raw);
        }

        let coarse = ParamId::Lfo1Rate;
        assert_eq!(coarse.max(), 255);
        assert_eq!(coarse.from_cc_value(coarse.to_cc_value(3)), 2);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn an_edit_prints_as_the_synthesizer_would_name_it() {
        let edit = ParamId::Lfo1Shape.edit(1).expect("1 is an LFO shape");
        assert_eq!(alloc::format!("{edit}"), "LFO 1 Shape = Triangle");
        let sweep = ParamId::Lfo1Rate.edit(64).expect("64 is in range");
        assert_eq!(alloc::format!("{sweep}"), "LFO 1 Rate = 64");
    }

    /// A centre outside the range would be a control drawn about a point it
    /// cannot reach, and a centre at either end would not be a centre.
    #[test]
    fn a_bipolar_centre_is_inside_the_range_and_not_at_an_end() {
        let mut bipolar = 0;
        for parameter in ParamId::ALL.iter().copied() {
            let Shape::Bipolar { centre } = parameter.shape() else {
                continue;
            };
            bipolar += 1;
            assert!(parameter.accepts(centre), "{parameter}");
            assert!(centre > parameter.min(), "{parameter} centres on its floor");
            assert!(centre < parameter.max(), "{parameter} centres on its top");
        }
        assert_eq!(bipolar, 45);

        assert_eq!(
            ParamId::Mod1Depth.shape(),
            Shape::Bipolar { centre: 128 },
            "-128 at 0 and +127 at 255"
        );
        assert_eq!(ParamId::Lfo1Rate.shape(), Shape::Unipolar);
    }

    /// The whole of what the sequencer strip needs: a centre to draw about, a
    /// value that is not a position, and the parameter that says how much of
    /// the run is played.
    #[test]
    fn a_sequencer_step_says_what_it_is() {
        let step = ParamId::SeqStepValue1;
        assert_eq!(step.shape(), Shape::Bipolar { centre: 128 });
        assert_eq!(step.inactive(), Some(0), "zero skips the step");
        assert_eq!(step.bounded_by(), Some(ParamId::SequenceLength));

        // All 32 of them, and nothing else in the table.
        let steps: Vec<ParamId> = ParamId::ALL
            .iter()
            .copied()
            .filter(|parameter| parameter.bounded_by().is_some())
            .collect();
        assert_eq!(steps.len(), 32);
        for step in steps {
            assert_eq!(step.group(), Group::ControlSequencer);
            assert_eq!(step.inactive(), Some(0));
            assert!(step.name().starts_with("Seq Step Value"));
        }
        assert_eq!(ParamId::SequenceLength.bounded_by(), None);
        assert_eq!(ParamId::Lfo1Rate.inactive(), None);
    }

    /// Two steps carried `kind = "switch"` while accepting 256 values, which
    /// their own range and their own note contradicted. Every step is the same
    /// sweep as every other, and the reason the table used to say otherwise is
    /// recorded where a host can show it.
    #[test]
    fn every_sequencer_step_reads_the_same_way() {
        for step in ParamId::ALL
            .iter()
            .copied()
            .filter(|p| p.name().starts_with("Seq Step Value"))
        {
            assert_eq!(step.kind(), Kind::Continuous, "{step}");
            assert_eq!(step.max(), 255, "{step}");
        }
        assert!(
            ParamId::SeqStepValue9
                .correction()
                .is_some_and(|text| text.contains("switch")),
            "the slip is recorded"
        );
        assert_eq!(ParamId::SeqStepValue1.correction(), None);
    }

    /// The prose is a getter over what the specification records, and it is
    /// there for every parameter the manual says anything about.
    #[test]
    fn the_manuals_own_words_are_reachable() {
        assert_eq!(
            ParamId::VcfFrequency.display(),
            Some("50.0 Hz to 20000.0 Hz")
        );
        assert_eq!(ParamId::Lfo1SlewRate.display(), None);
        assert_eq!(ParamId::Lfo1SlewRate.note(), None);
        assert!(
            ParamId::Lfo1Rate
                .note()
                .is_some_and(|note| note.contains("Arp Sync"))
        );

        let counted = |field: fn(ParamId) -> Option<&'static str>| {
            ParamId::ALL
                .iter()
                .copied()
                .filter(|parameter| field(*parameter).is_some())
                .count()
        };
        assert_eq!(counted(ParamId::display), 26);
        assert_eq!(counted(ParamId::note), 129);
        assert_eq!(counted(ParamId::correction), 37);
    }

    /// An unconfirmed reading says so, and says why in the same breath.
    #[test]
    fn an_inferred_reading_is_marked_and_explained() {
        assert!(!ParamId::PitchBendUpDepth.confirmed());
        assert!(ParamId::PitchBendUpDepth.correction().is_some());
        assert!(ParamId::VcfFrequency.confirmed());
        for parameter in ParamId::ALL.iter().copied() {
            assert!(
                parameter.confirmed() || parameter.correction().is_some(),
                "{parameter} is unconfirmed and does not say why"
            );
        }
    }
}
