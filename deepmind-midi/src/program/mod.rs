//! A program: the 242 bytes that are one sound, and the types that read them.
//!
//! A [`Program`] holds a dump exactly as the synthesizer sent it and lends every
//! byte a name. The parameter layer says what each byte means; this layer is
//! where the bytes live.
//!
//! ```
//! use deepmind_midi::ids::ProtocolVersion;
//! use deepmind_midi::param::ParamId;
//! use deepmind_midi::program::{LfoShape, Program, ProgramName};
//!
//! let mut program = Program::new(ProtocolVersion::V6)?;
//! program.set_name(ProgramName::new("Bass Sweep")?);
//! program.set_lfo1_shape(LfoShape::Triangle);
//! program.set_vcf_frequency(200);
//!
//! assert_eq!(program.name().as_str(), "Bass Sweep");
//! assert_eq!(program.lfo1_shape(), Some(LfoShape::Triangle));
//! assert_eq!(program.get(ParamId::VcfFrequency), 200);
//! # Ok::<(), deepmind_midi::Error>(())
//! ```
//!
//! # The bytes are what is stored
//!
//! A program is its dump, not a decoded copy of one. Reading is a lookup and
//! writing is a byte, so a dump that goes in comes out unchanged - including the
//! reserved bytes a comms protocol version 7 dump carries, and including a value
//! no table lists, which hardware is entitled to send and a library is not
//! entitled to lose.
//!
//! That is also why the typed accessors answer `Option` for a parameter with a
//! value table. `None` means the byte is not one the table names; the byte is
//! still there, and [`Program::get`] returns it.
//!
//! # Firmware
//!
//! Three value tables were renumbered by firmware 1.1, and nothing in a stored
//! program says which firmware wrote it. The generated accessors assume
//! [`DEFAULT_FIRMWARE`](crate::param::DEFAULT_FIRMWARE); where the host knows
//! better, [`ModSource::from_raw_for`] and its neighbours take the version a
//! device inquiry reported and [`Program::get`] hands them the byte.
//!
//! # Getting one on and off the wire
//!
//! [`Program::from_dump`] takes the two messages that carry a program.
//! [`Program::pack_into`] is the way back, into a buffer a
//! [`Message`] can borrow:
//!
//! ```
//! use deepmind_midi::ids::{DeviceId, ProtocolVersion};
//! use deepmind_midi::program::Program;
//! use deepmind_midi::sysex::{Frame, Message};
//!
//! let program = Program::new(ProtocolVersion::V6)?;
//!
//! let mut packed = [0; Program::PACKED_MAX_LEN];
//! let len = program.pack_into(&mut packed)?;
//! let frame = Frame::new(
//!     DeviceId::Unit(0),
//!     Message::EditBufferDumpResponse {
//!         version: program.version(),
//!         packed: packed.get(..len).unwrap_or_default(),
//!     },
//! );
//!
//! let mut bytes = [0; 320];
//! let len = frame.encode_into(&mut bytes)?;
//! assert_eq!(bytes.get(..2), Some(&[0xF0, 0x00][..]));
//! # Ok::<(), deepmind_midi::Error>(())
//! ```
//!
//! # Serde
//!
//! [`ProgramName`] and the value types serialize under the `serde` feature.
//! [`Program`] does not: it is a byte string with a version, and
//! [`Program::as_bytes`] with [`Program::from_bytes`] is that byte string, in
//! the form the synthesizer itself uses. A host that wants a program inside a
//! document of its own has the bytes to put there.

mod generated;
mod name;

// The generated module is this module's other half: it holds a type per value
// table and a pair of accessors per parameter, both of which are this module's
// public surface. Naming its twenty-two value types here would be a list to keep
// in step by hand, and a table added to `spec/` would go missing from the API
// rather than fail to build.
pub use generated::*;
pub use name::ProgramName;

use core::fmt;

use crate::error::{Error, Result};
use crate::ids::{PROGRAMS_PER_BANK, ProgramNumber, ProtocolVersion};
use crate::param::{PARAMETER_COUNT, ParamId};
use crate::sysex::{Message, PROGRAM_NAME_LEN, packed};

/// One program: a sound, as the synthesizer stores and sends it.
///
/// See the [module documentation](self) for what it is and how it travels.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Program {
    version: ProtocolVersion,
    data: [u8; Program::MAX_LEN],
}

impl Program {
    /// Bytes the longest documented program carries.
    ///
    /// A comms protocol version 7 program; a version 6 program is three bytes
    /// shorter and those three bytes are the reserved ones.
    pub const MAX_LEN: usize = match ProtocolVersion::V7.program_data_len() {
        Some(len) => len,
        None => PARAMETER_COUNT,
    };

    /// Bytes [`Program::pack_into`] needs at most.
    pub const PACKED_MAX_LEN: usize = packed::packed_len(Self::MAX_LEN);

    /// Builds a program with every parameter at the lowest value it accepts.
    ///
    /// Not the initialised program the synthesizer's INIT button loads, which is
    /// a factory sound this specification does not record. It is a program that
    /// every parameter accepts, which is what building one from nothing needs.
    ///
    /// # Errors
    ///
    /// Returns [`Error::UnsupportedProtocolVersion`] for a version this build
    /// does not know the length of.
    pub fn new(version: ProtocolVersion) -> Result<Self> {
        version
            .program_data_len()
            .ok_or(Error::UnsupportedProtocolVersion(version.0))?;
        let mut program = Self {
            version,
            data: [0; Self::MAX_LEN],
        };
        for parameter in ParamId::ALL {
            let min = u8::try_from(parameter.min()).unwrap_or(0);
            program.set_clamped(parameter, min);
        }
        Ok(program)
    }

    /// Reads a program from the bytes of an unpacked dump.
    ///
    /// Nothing is checked beyond the length. A value outside what its parameter
    /// accepts is kept and reported by [`Program::invalid`]: the synthesizer is
    /// the authority on what it holds, and a decoder that refuses a dump over
    /// one byte is a decoder nobody can use.
    ///
    /// # Errors
    ///
    /// Returns [`Error::UnsupportedProtocolVersion`] for an unknown version and
    /// [`Error::ProgramLength`] when `bytes` is not as long as that version's
    /// program.
    pub fn from_bytes(version: ProtocolVersion, bytes: &[u8]) -> Result<Self> {
        let expected = version
            .program_data_len()
            .ok_or(Error::UnsupportedProtocolVersion(version.0))?;
        if bytes.len() != expected {
            return Err(Error::ProgramLength {
                expected,
                found: bytes.len(),
            });
        }
        let mut data = [0; Self::MAX_LEN];
        for (slot, byte) in data.iter_mut().zip(bytes) {
            *slot = *byte;
        }
        Ok(Self { version, data })
    }

    /// Reads a program from the packed payload of a dump.
    ///
    /// A packed run is padded to a multiple of eight bytes, so it can carry more
    /// than the program needs; the version says how much of it is the program
    /// and the rest is dropped.
    ///
    /// # Errors
    ///
    /// Returns [`Error::UnsupportedProtocolVersion`] for an unknown version,
    /// [`Error::PackedRunLength`] for a run that cannot be unpacked,
    /// [`Error::BufferTooSmall`] for one longer than any program, and
    /// [`Error::ProgramLength`] for one shorter than this version's program.
    pub fn from_packed(version: ProtocolVersion, packed: &[u8]) -> Result<Self> {
        let expected = version
            .program_data_len()
            .ok_or(Error::UnsupportedProtocolVersion(version.0))?;
        let mut data = [0; Self::MAX_LEN];
        let found = packed::unpack_into(packed, &mut data)?;
        if found < expected {
            return Err(Error::ProgramLength { expected, found });
        }
        for slot in data.iter_mut().skip(expected) {
            *slot = 0;
        }
        Ok(Self { version, data })
    }

    /// Reads a program from the message that carried it.
    ///
    /// Both dumps that carry a program are accepted: one stored in a bank, and
    /// the edit buffer.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotAProgramDump`] for any other message, and whatever
    /// [`Program::from_packed`] returns for the payload.
    pub fn from_dump(message: &Message<'_>) -> Result<Self> {
        match *message {
            Message::ProgramDumpResponse {
                version, packed, ..
            }
            | Message::EditBufferDumpResponse { version, packed } => {
                Self::from_packed(version, packed)
            }
            other => Err(Error::NotAProgramDump(other.command().to_byte())),
        }
    }

    /// Returns the comms protocol version this program is stored in.
    #[must_use]
    pub const fn version(&self) -> ProtocolVersion {
        self.version
    }

    /// Returns the program as an unpacked dump.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.data.get(..self.data_len()).unwrap_or(&self.data)
    }

    /// Returns the bytes past the last parameter.
    ///
    /// Empty under comms protocol version 6. Version 7 adds three reserved
    /// bytes, which are zero in every factory program examined and are carried
    /// through rather than decoded.
    #[must_use]
    pub fn reserved(&self) -> &[u8] {
        self.as_bytes().get(PARAMETER_COUNT..).unwrap_or_default()
    }

    /// Returns the number of bytes [`Program::pack_into`] writes.
    #[must_use]
    pub fn packed_len(&self) -> usize {
        packed::packed_len(self.data_len())
    }

    /// Packs the program into `out`, ready for a dump message to borrow.
    ///
    /// # Errors
    ///
    /// Returns [`Error::BufferTooSmall`] when `out` is shorter than
    /// [`Program::packed_len`].
    pub fn pack_into(&self, out: &mut [u8]) -> Result<usize> {
        packed::pack_into(self.as_bytes(), out)
    }

    /// Returns the raw value of one parameter.
    #[must_use]
    pub fn get(&self, parameter: ParamId) -> u8 {
        self.data
            .get(usize::from(parameter.offset()))
            .copied()
            .unwrap_or(0)
    }

    /// Sets one parameter, refusing a value it does not accept.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ValueOutOfRange`] when the parameter does not accept the
    /// value. [`Program::set_clamped`] is there for callers that would rather
    /// bend the value than refuse it.
    pub fn set(&mut self, parameter: ParamId, value: u8) -> Result<()> {
        if !parameter.accepts(u16::from(value)) {
            return Err(Error::ValueOutOfRange {
                parameter: parameter.offset(),
                value: u16::from(value),
                max: parameter.max(),
            });
        }
        self.write(parameter, value);
        Ok(())
    }

    /// Sets one parameter, bringing the value inside its range first.
    ///
    /// Returns what was stored. This is what the generated accessors use, since
    /// a `u8` reaches values a parameter does not.
    pub fn set_clamped(&mut self, parameter: ParamId, value: u8) -> u8 {
        // Every parameter's range fits in the byte it occupies in a dump, so the
        // clamped value is always a byte.
        let clamped = u8::try_from(parameter.clamp(u16::from(value))).unwrap_or(value);
        self.write(parameter, clamped);
        clamped
    }

    /// Returns every parameter and the value it holds, in offset order.
    pub fn values(&self) -> impl Iterator<Item = (ParamId, u8)> + '_ {
        ParamId::ALL
            .into_iter()
            .map(move |parameter| (parameter, self.get(parameter)))
    }

    /// Returns the parameters whose values differ, and what `target` holds.
    ///
    /// The list of edits that turn this program into `target`.
    /// [`ParamId::edit`] turns one into the NRPN message that sends it.
    ///
    /// ```
    /// use deepmind_midi::ids::ProtocolVersion;
    /// use deepmind_midi::param::ParamId;
    /// use deepmind_midi::program::Program;
    ///
    /// let before = Program::new(ProtocolVersion::V6)?;
    /// let mut after = before.clone();
    /// after.set_lfo1_rate(64);
    ///
    /// let changed: Vec<_> = before.changes(&after).collect();
    /// assert_eq!(changed, [(ParamId::Lfo1Rate, 64)]);
    /// # Ok::<(), deepmind_midi::Error>(())
    /// ```
    pub fn changes<'a>(&'a self, target: &'a Self) -> impl Iterator<Item = (ParamId, u8)> + 'a {
        ParamId::ALL.into_iter().filter_map(move |parameter| {
            let value = target.get(parameter);
            (self.get(parameter) != value).then_some((parameter, value))
        })
    }

    /// Returns the parameters holding a value they do not accept.
    ///
    /// Empty for anything the synthesizer sent, on the evidence so far. A dump
    /// read from a file is another matter.
    pub fn invalid(&self) -> impl Iterator<Item = (ParamId, u8)> + '_ {
        self.values()
            .filter(|(parameter, value)| !parameter.accepts(u16::from(*value)))
    }

    /// Returns whether every parameter holds a value it accepts.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ValueOutOfRange`] for the first parameter that does not,
    /// in offset order. [`Program::invalid`] lists them all.
    pub fn validate(&self) -> Result<()> {
        match self.invalid().next() {
            None => Ok(()),
            Some((parameter, value)) => Err(Error::ValueOutOfRange {
                parameter: parameter.offset(),
                value: u16::from(value),
                max: parameter.max(),
            }),
        }
    }

    /// Returns the program's name.
    #[must_use]
    pub fn name(&self) -> ProgramName {
        let start = usize::from(NAME_OFFSET);
        ProgramName::from_field(
            self.data
                .get(start..start.saturating_add(NAME_LEN))
                .unwrap_or_default(),
        )
    }

    /// Sets the program's name.
    ///
    /// [`ProgramName::new`] is where a string becomes one of these, and where a
    /// string that cannot be a program name is refused.
    pub fn set_name(&mut self, name: ProgramName) {
        let start = usize::from(NAME_OFFSET);
        let field = self
            .data
            .get_mut(start..start.saturating_add(NAME_LEN))
            .unwrap_or_default();
        for (slot, byte) in field
            .iter_mut()
            .zip(name.as_bytes().iter().copied().chain(core::iter::repeat(0)))
        {
            *slot = byte;
        }
    }

    /// Returns the program's transposition in semitones, -48 to 48.
    ///
    /// The one parameter that does not start at zero: the manual gives it as 80
    /// to 176 for -48 to +48, so the value it carries is its distance from the
    /// middle of that range.
    #[must_use]
    pub fn transpose(&self) -> i8 {
        let raw = i16::from(self.get(ParamId::ProgramTranspose));
        i8::try_from(raw - i16::from(transpose_centre())).unwrap_or(0)
    }

    /// Sets the program's transposition in semitones.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ValueOutOfRange`] outside -48 to 48.
    pub fn set_transpose(&mut self, semitones: i8) -> Result<()> {
        let raw = i16::from(transpose_centre()) + i16::from(semitones);
        let raw = u8::try_from(raw).unwrap_or(0);
        self.set(ParamId::ProgramTranspose, raw)
    }

    /// Returns the length of this program's dump.
    fn data_len(&self) -> usize {
        self.version.program_data_len().unwrap_or(PARAMETER_COUNT)
    }

    /// Writes one byte, at an offset the parameter table guarantees is inside
    /// the program.
    fn write(&mut self, parameter: ParamId, value: u8) {
        if let Some(slot) = self.data.get_mut(usize::from(parameter.offset())) {
            *slot = value;
        }
    }
}

/// Returns the raw value that transposes by nothing, which is the middle of the
/// parameter's range.
fn transpose_centre() -> u8 {
    let parameter = ParamId::ProgramTranspose;
    u8::try_from(parameter.min().midpoint(parameter.max())).unwrap_or(0)
}

impl fmt::Debug for Program {
    /// Names the program rather than listing 242 bytes.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Program")
            .field("version", &self.version)
            .field("name", &self.name())
            .finish_non_exhaustive()
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.name(), f)
    }
}

/// The names of the 128 programs in one bank, as the bank names dump carries
/// them.
///
/// Sixteen bytes each, borrowed where they lie. The dump answers a whole bank at
/// once, which is how a host fills a program list without asking for 128
/// programs.
///
/// ```
/// use deepmind_midi::ids::ProgramNumber;
/// use deepmind_midi::program::BankNames;
///
/// let mut unpacked = [0; BankNames::LEN];
/// unpacked.get_mut(..4).unwrap_or_default().copy_from_slice(b"Bass");
///
/// let names = BankNames::new(&unpacked)?;
/// assert_eq!(names.get(ProgramNumber::new(0)?).as_str(), "Bass");
/// assert_eq!(names.iter().count(), BankNames::COUNT);
/// # Ok::<(), deepmind_midi::Error>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BankNames<'a> {
    unpacked: &'a [u8],
}

impl<'a> BankNames<'a> {
    /// Names one dump carries, which is every program in a bank.
    pub const COUNT: usize = PROGRAMS_PER_BANK as usize;

    /// Bytes the unpacked dump holds.
    pub const LEN: usize = PROGRAM_NAME_LEN * Self::COUNT;

    /// Reads the names out of an unpacked bank names dump.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ProgramLength`] when `unpacked` is not [`BankNames::LEN`]
    /// bytes.
    pub fn new(unpacked: &'a [u8]) -> Result<Self> {
        if unpacked.len() != Self::LEN {
            return Err(Error::ProgramLength {
                expected: Self::LEN,
                found: unpacked.len(),
            });
        }
        Ok(Self { unpacked })
    }

    /// Returns one program's name.
    #[must_use]
    pub fn get(&self, program: ProgramNumber) -> ProgramName {
        let start = usize::from(program.get()) * PROGRAM_NAME_LEN;
        ProgramName::from_field(
            self.unpacked
                .get(start..start.saturating_add(PROGRAM_NAME_LEN))
                .unwrap_or_default(),
        )
    }

    /// Returns every name, in program order.
    pub fn iter(&self) -> impl Iterator<Item = ProgramName> + '_ {
        self.unpacked
            .chunks_exact(PROGRAM_NAME_LEN)
            .map(ProgramName::from_field)
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use super::*;
    use crate::ids::Bank;
    use crate::sysex::inquiry::Version;

    /// Firmware 1.0, which numbers three value tables differently.
    const FIRMWARE_1_0: Version = Version { major: 1, minor: 0 };

    fn program() -> Program {
        Program::new(ProtocolVersion::V6).expect("version 6 is supported")
    }

    #[test]
    fn a_dump_survives_the_round_trip_byte_for_byte() {
        let mut bytes = [0; PARAMETER_COUNT];
        for (offset, byte) in bytes.iter_mut().enumerate() {
            // Every byte pattern, including values no parameter accepts.
            *byte = u8::try_from(offset).unwrap_or(0).wrapping_mul(7);
        }
        let program = Program::from_bytes(ProtocolVersion::V6, &bytes).expect("a v6 program");
        assert_eq!(program.as_bytes(), bytes);

        let mut packed = [0; Program::PACKED_MAX_LEN];
        let len = program.pack_into(&mut packed).expect("room to pack");
        let read = Program::from_packed(ProtocolVersion::V6, packed.get(..len).expect("packed"))
            .expect("a v6 program");
        assert_eq!(read, program);
    }

    /// A packed run is padded to a multiple of eight, so 242 bytes travel as
    /// 280 and the last three are not the program's.
    #[test]
    fn padding_on_the_wire_is_not_part_of_the_program() {
        let program = program();
        assert_eq!(program.as_bytes().len(), 242);
        assert_eq!(program.packed_len(), 280);

        let mut packed = [0xFF; Program::PACKED_MAX_LEN];
        let len = program.pack_into(&mut packed).expect("room to pack");
        assert_eq!(len, 280);
        let read = Program::from_packed(ProtocolVersion::V6, packed.get(..len).expect("packed"))
            .expect("a v6 program");
        assert_eq!(read, program);
    }

    #[test]
    fn version_seven_keeps_its_reserved_bytes() {
        let mut bytes = [0; 245];
        for (index, byte) in bytes.iter_mut().enumerate().skip(PARAMETER_COUNT) {
            *byte = u8::try_from(index).unwrap_or(0);
        }
        let seven = Program::from_bytes(ProtocolVersion::V7, &bytes).expect("a v7 program");
        assert_eq!(seven.reserved(), &[242, 243, 244]);
        assert_eq!(seven.as_bytes(), bytes);
        assert!(program().reserved().is_empty());
    }

    /// Every parameter's range fits in the byte it occupies, over every byte a
    /// caller can offer. The clamp relies on it, and 242 rows are too many to
    /// take on trust.
    #[test]
    fn clamping_lands_inside_every_parameters_range() {
        let mut program = program();
        for parameter in ParamId::ALL {
            for value in 0..=u8::MAX {
                let stored = program.set_clamped(parameter, value);
                assert_eq!(program.get(parameter), stored);
                assert!(
                    parameter.accepts(u16::from(stored)),
                    "{parameter} clamped {value} to {stored}"
                );
            }
        }
    }

    #[test]
    fn a_dump_of_the_wrong_length_is_refused() {
        assert_eq!(
            Program::from_bytes(ProtocolVersion::V6, &[0; 240]),
            Err(Error::ProgramLength {
                expected: 242,
                found: 240,
            })
        );
        assert_eq!(
            Program::from_bytes(ProtocolVersion(99), &[0; 242]),
            Err(Error::UnsupportedProtocolVersion(99))
        );
    }

    #[test]
    fn a_new_program_holds_what_every_parameter_accepts() {
        let program = program();
        assert_eq!(program.validate(), Ok(()));
        assert_eq!(program.invalid().count(), 0);
        // The one parameter that does not start at zero.
        assert_eq!(program.program_transpose(), 80);
        assert_eq!(program.transpose(), -48);
    }

    #[test]
    fn a_value_outside_the_range_is_kept_but_reported() {
        let mut bytes = [0; PARAMETER_COUNT];
        // Offset 2 is LFO 1 Shape, which stops at 6.
        *bytes.get_mut(2).expect("offset 2") = 9;
        let program = Program::from_bytes(ProtocolVersion::V6, &bytes).expect("a v6 program");

        assert_eq!(program.get(ParamId::Lfo1Shape), 9);
        assert_eq!(program.lfo1_shape(), None);
        assert_eq!(
            program.invalid().collect::<Vec<_>>(),
            [(ParamId::Lfo1Shape, 9), (ParamId::ProgramTranspose, 0)]
        );
    }

    #[test]
    fn setting_refuses_what_clamping_bends() {
        let mut program = program();
        assert_eq!(
            program.set(ParamId::Lfo1Shape, 9),
            Err(Error::ValueOutOfRange {
                parameter: 2,
                value: 9,
                max: 6,
            })
        );
        assert_eq!(program.set_clamped(ParamId::Lfo1Shape, 9), 6);
        assert_eq!(program.lfo1_shape(), Some(LfoShape::SampleAndGlide));
    }

    #[test]
    fn the_typed_accessors_are_the_bytes_under_another_name() {
        let mut program = program();
        program.set_lfo1_shape(LfoShape::Square);
        program.set_lfo1_key_sync(true);
        program.set_vcf_frequency(128);

        assert_eq!(program.get(ParamId::Lfo1Shape), 2);
        assert_eq!(program.get(ParamId::Lfo1KeySync), 1);
        assert_eq!(program.get(ParamId::VcfFrequency), 128);
        assert!(program.lfo1_key_sync());
        assert_eq!(program.vcf_frequency(), 128);
    }

    /// Firmware 1.1 renumbered the modulation sources, so the same byte is a
    /// different source depending on which firmware wrote the program.
    #[test]
    fn a_renumbered_table_reads_by_firmware() {
        let mut program = program();
        program.set_mod1_source(ModSource::Expression);
        let raw = program.get(ParamId::Mod1Source);

        assert_eq!(raw, 6);
        assert_eq!(program.mod1_source(), Some(ModSource::Expression));
        assert_eq!(
            ModSource::from_raw_for(raw, FIRMWARE_1_0),
            Some(ModSource::Lfo1)
        );
        assert_eq!(ModSource::Expression.raw_for(FIRMWARE_1_0), None);
        assert_eq!(ModSource::Lfo1.raw_for(FIRMWARE_1_0), Some(6));
        assert_eq!(ModSource::Expression.name(), "Expression");
        assert_eq!(ModSource::Expression.name_for(FIRMWARE_1_0), None);
    }

    #[test]
    fn every_value_of_every_table_survives_the_round_trip() {
        for source in ModSource::ALL {
            assert_eq!(ModSource::from_raw(source.raw()), Some(source));
        }
        for destination in ModDestination::ALL {
            assert_eq!(
                ModDestination::from_raw(destination.raw()),
                Some(destination)
            );
        }
        for effect in FxType::ALL {
            assert_eq!(FxType::from_raw(effect.raw()), Some(effect));
        }
        for shape in LfoShape::ALL {
            assert_eq!(LfoShape::from_raw(shape.raw()), Some(shape));
            assert_eq!(
                Some(shape.name()),
                shape.name_for(crate::param::DEFAULT_FIRMWARE)
            );
        }
    }

    #[test]
    fn a_name_is_written_and_read_back() {
        let mut program = program();
        program.set_name(ProgramName::new("Bass Sweep").expect("a valid name"));

        assert_eq!(program.name().as_str(), "Bass Sweep");
        assert_eq!(program.get(ParamId::ProgramNameChar1), b'B');
        // The terminator, and everything after it, is cleared.
        assert_eq!(program.get(ParamId::ProgramNameChar11), 0);
        assert_eq!(program.get(ParamId::ProgramNameChar17), 0);

        program.set_name(ProgramName::new("A").expect("a valid name"));
        assert_eq!(program.name().as_str(), "A");
        assert_eq!(program.get(ParamId::ProgramNameChar2), 0);
    }

    #[test]
    fn transposition_counts_from_the_middle_of_its_range() {
        let mut program = program();
        program.set_transpose(0).expect("no transposition");
        assert_eq!(program.program_transpose(), 128);
        assert_eq!(program.transpose(), 0);

        program.set_transpose(48).expect("four octaves up");
        assert_eq!(program.program_transpose(), 176);
        assert_eq!(program.transpose(), 48);

        assert!(program.set_transpose(49).is_err());
        assert!(program.set_transpose(-49).is_err());
    }

    #[test]
    fn changes_are_the_edits_that_close_the_gap() {
        let before = program();
        let mut after = before.clone();
        after.set_lfo1_rate(64);
        after.set_lfo1_shape(LfoShape::Square);

        let changed: Vec<_> = before.changes(&after).collect();
        assert_eq!(changed, [(ParamId::Lfo1Rate, 64), (ParamId::Lfo1Shape, 2)]);

        let mut applied = before.clone();
        for (parameter, value) in before.changes(&after) {
            applied.set(parameter, value).expect("a value it accepts");
        }
        assert_eq!(applied, after);
        assert_eq!(before.changes(&before).count(), 0);
    }

    #[test]
    fn a_dump_message_carries_a_program_and_nothing_else_does() {
        let program = program();
        let mut packed = [0; Program::PACKED_MAX_LEN];
        let len = program.pack_into(&mut packed).expect("room to pack");
        let packed = packed.get(..len).expect("packed");

        let dump = Message::EditBufferDumpResponse {
            version: ProtocolVersion::V6,
            packed,
        };
        assert_eq!(Program::from_dump(&dump), Ok(program.clone()));

        let stored = Message::ProgramDumpResponse {
            version: ProtocolVersion::V6,
            bank: Bank::new(0).expect("bank A"),
            program: ProgramNumber::new(0).expect("program 0"),
            packed,
        };
        assert_eq!(Program::from_dump(&stored), Ok(program));

        assert_eq!(
            Program::from_dump(&Message::EditBufferDumpRequest),
            Err(Error::NotAProgramDump(0x03))
        );
    }

    #[test]
    fn a_bank_of_names_is_read_where_it_lies() {
        let mut unpacked = [0; BankNames::LEN];
        let first = unpacked.get_mut(..4).expect("the first name");
        first.copy_from_slice(b"Bass");
        let second = unpacked
            .get_mut(PROGRAM_NAME_LEN..PROGRAM_NAME_LEN + 16)
            .expect("the second name");
        second.copy_from_slice(b"SixteenCharacter");
        let names = BankNames::new(&unpacked).expect("a bank of names");

        assert_eq!(
            names
                .get(ProgramNumber::new(0).expect("program 0"))
                .as_str(),
            "Bass"
        );
        // Sixteen bytes with no terminator is still a name.
        assert_eq!(
            names
                .get(ProgramNumber::new(1).expect("program 1"))
                .as_str(),
            "SixteenCharacter"
        );
        assert_eq!(names.iter().count(), 128);
        assert_eq!(
            BankNames::new(&[0; 16]),
            Err(Error::ProgramLength {
                expected: BankNames::LEN,
                found: 16,
            })
        );
    }
}
