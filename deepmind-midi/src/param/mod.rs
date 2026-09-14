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
//! It does not carry the manual's prose. Notes, displayed ranges and the places
//! this specification departs from the printed table live in
//! `docs/midi-spec.md`, addressed by the same offsets, which keeps them out of
//! the binary on the targets this crate is meant for. Value-table entries carry
//! names and not descriptions for the same reason.
//!
//! Nor does it carry conversions from a raw value to the number the synthesizer
//! displays. The manual publishes the two ends of a range and almost never the
//! curve between them. Conversions arrive per parameter as they are measured;
//! see the scaling section of `docs/midi-spec.md`.
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

/// One value of an enumerated parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
pub struct ValueEntry {
    /// The value on the wire.
    pub value: u16,
    /// Name as the synthesizer displays it.
    pub name: &'static str,
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
        self.entries
            .iter()
            .find(|entry| entry.value == value)
            .map(|entry| entry.name)
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
    /// No two controllers claim the same parameter, which the specification
    /// checks when it loads.
    #[must_use]
    pub fn for_parameter(parameter: ParamId) -> Option<&'static Self> {
        CONTROLLERS
            .iter()
            .find(|controller| controller.parameter == Some(parameter))
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

    /// Returns the controller that drives this parameter, if one does.
    #[must_use]
    pub fn controller(self) -> Option<&'static Controller> {
        Controller::for_parameter(self)
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
}
