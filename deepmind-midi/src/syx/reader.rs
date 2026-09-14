//! Walking a `.syx` file: the frames in it, and the programs they carry.

use core::iter::FusedIterator;

use crate::error::{Error, Result};
use crate::ids::Slot;
use crate::program::Program;
use crate::sysex::{Frame, Message};
use crate::wire::{SYSEX_END, SYSEX_START};

/// One program a file carries, and where the file says it belongs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Entry {
    /// Slot named by a stored program dump, absent for an edit buffer dump.
    ///
    /// [`Slot`](crate::ids::Slot) is where a program is stored; the edit buffer
    /// is where one is played, which is nowhere in particular.
    pub slot: Option<Slot>,
    /// The program itself.
    pub program: Program,
}

/// A `.syx` file: the bytes of one, borrowed where the host put them.
///
/// The library does no IO, so reading the file is the host's job and this type
/// starts from the bytes. Nothing is copied: every frame, and every program
/// built from one, borrows from the slice until the program is decoded.
///
/// ```
/// use deepmind_midi::ids::{Bank, DeviceId, ProgramNumber, ProtocolVersion};
/// use deepmind_midi::program::{Program, ProgramName};
/// use deepmind_midi::syx::{self, File};
///
/// // Whatever the host read off disk. Built here so the example stands alone.
/// let mut program = Program::new(ProtocolVersion::V6);
/// program.set_name(ProgramName::new("Bass Sweep")?);
///
/// let mut bytes = [0; syx::MAX_PROGRAM_FRAME_LEN];
/// let mut writer = syx::Writer::new(&mut bytes, DeviceId::Unit(0));
/// writer.push_program(Bank::A, ProgramNumber::FIRST, &program)?;
/// let written = writer.finish();
///
/// let file = File::new(bytes.get(..written).unwrap_or_default());
/// let entry = file.programs().next().expect("one program")?;
///
/// assert_eq!(entry.program.name().as_str(), "Bass Sweep");
/// assert_eq!(entry.slot.map(|slot| slot.to_string()).as_deref(), Some("A1"));
/// # Ok::<(), deepmind_midi::Error>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct File<'a> {
    bytes: &'a [u8],
}

impl<'a> File<'a> {
    /// Takes a file's bytes. Nothing is read until an iterator asks.
    #[must_use]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    /// Returns the bytes the file was built from.
    #[must_use]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.bytes
    }

    /// Returns every frame in the file, in the order it holds them.
    #[must_use]
    pub const fn frames(&self) -> Frames<'a> {
        Frames::new(self.bytes)
    }

    /// Returns every program in the file, in the order it holds them.
    #[must_use]
    pub const fn programs(&self) -> Programs<'a> {
        Programs::new(self.bytes)
    }
}

/// Every `SysEx` frame in a file, borrowed where it lies.
///
/// A frame runs from an `F0` to the next `F7`: a payload is seven-bit data, so
/// neither byte can occur inside one. Bytes between frames are skipped rather
/// than refused, since a file is free to pad. An `F0` followed by another `F0`
/// before any `F7` is a frame the file never closed; it yields
/// [`Error::Unframed`] and the walk goes on from the second `F0`, so a
/// truncated dump does not take the one after it down with it. An `F0` with no
/// `F7` anywhere after it yields the same error and ends the walk.
/// [`Frames::offset`] says where the bad frame started either way.
///
/// The item is a `Result` because a file is untrusted input: a frame another
/// device wrote, or one with a payload its command does not take, is one bad
/// item rather than the end of the file. A host that wants only what parses
/// writes `.filter_map(Result::ok)`.
#[derive(Debug, Clone)]
pub struct Frames<'a> {
    bytes: &'a [u8],
    pos: usize,
    offset: usize,
    done: bool,
}

impl<'a> Frames<'a> {
    /// Starts a walk over a file's bytes.
    #[must_use]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            pos: 0,
            offset: 0,
            done: false,
        }
    }

    /// Returns where in the file the frame just returned started.
    ///
    /// Zero before the first item. Read it after a failing `next` to say which
    /// frame of the file was the bad one.
    #[must_use]
    pub const fn offset(&self) -> usize {
        self.offset
    }
}

impl<'a> Iterator for Frames<'a> {
    type Item = Result<Frame<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let rest = self.bytes.get(self.pos..)?;
        let start = self
            .pos
            .saturating_add(rest.iter().position(|byte| *byte == SYSEX_START)?);
        let frame = self.bytes.get(start..)?;
        self.offset = start;

        let Some(end) = frame
            .iter()
            .skip(1)
            .position(|byte| *byte == SYSEX_END || *byte == SYSEX_START)
            .map(|index| index.saturating_add(1))
        else {
            // An F0 the file never closes. There is nothing after it to walk.
            self.done = true;
            self.pos = self.bytes.len();
            return Some(Err(Error::Unframed));
        };
        if frame.get(end) == Some(&SYSEX_START) {
            // A frame cut short by the next one, which is kept.
            self.pos = start.saturating_add(end);
            return Some(Err(Error::Unframed));
        }
        self.pos = start.saturating_add(end).saturating_add(1);
        Some(Frame::parse(frame.get(..=end)?))
    }
}

impl FusedIterator for Frames<'_> {}

/// Every program in a file, in the order it holds them.
///
/// Both dumps that carry a program are taken: one stored in a bank, and the edit
/// buffer. A frame carrying something else (the globals, a pattern, a bank of
/// names) is skipped: a pack may hold those, and a host asking for programs is
/// not asking about them. Anything that fails to parse is yielded as the error
/// it failed with, per [`Frames`].
#[derive(Debug, Clone)]
pub struct Programs<'a> {
    frames: Frames<'a>,
}

impl<'a> Programs<'a> {
    /// Starts a walk over the programs in a file's bytes.
    #[must_use]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self {
            frames: Frames::new(bytes),
        }
    }

    /// Returns where in the file the frame just returned started.
    #[must_use]
    pub const fn offset(&self) -> usize {
        self.frames.offset()
    }
}

impl Iterator for Programs<'_> {
    type Item = Result<Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let frame = match self.frames.next()? {
                Ok(frame) => frame,
                Err(error) => return Some(Err(error)),
            };
            let (slot, version, packed) = match frame.message {
                Message::ProgramDumpResponse {
                    version,
                    bank,
                    program,
                    packed,
                } => (Some(Slot::new(bank, program)), version, packed),
                Message::EditBufferDumpResponse { version, packed } => (None, version, packed),
                _ => continue,
            };
            return Some(
                Program::from_packed(version, packed).map(|program| Entry { slot, program }),
            );
        }
    }
}

impl FusedIterator for Programs<'_> {}

#[cfg(all(test, feature = "alloc"))]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use alloc::vec::Vec;

    use super::*;
    use crate::ids::{Bank, DeviceId, ProgramNumber, ProtocolVersion};
    use crate::program::ProgramName;
    use crate::syx::{MAX_PROGRAM_FRAME_LEN, Writer, bank_to_vec, program_frame_len};

    fn named(name: &str) -> Program {
        let mut program = Program::new(ProtocolVersion::V6);
        program.set_name(ProgramName::new(name).expect("a name the display can show"));
        program
    }

    /// A file holding the given programs, stored from `A1` on.
    fn pack(programs: &[Program]) -> Vec<u8> {
        bank_to_vec(DeviceId::Unit(0), Bank::A, ProgramNumber::FIRST, programs)
            .expect("a bank that fits")
    }

    #[test]
    fn a_file_hands_back_the_programs_put_into_it() {
        let written = [named("One"), named("Two"), named("Three")];
        let bytes = pack(&written);

        let read: Vec<Entry> = File::new(&bytes)
            .programs()
            .collect::<Result<_>>()
            .expect("every program parses");

        assert_eq!(read.len(), 3);
        for (index, (entry, program)) in read.iter().zip(&written).enumerate() {
            assert_eq!(&entry.program, program);
            assert_eq!(
                entry.slot,
                Some(Slot::new(
                    Bank::A,
                    ProgramNumber::new(u8::try_from(index).expect("a small index"))
                        .expect("a program in range")
                ))
            );
        }
    }

    #[test]
    fn an_empty_file_holds_nothing() {
        assert_eq!(File::new(&[]).frames().count(), 0);
        assert_eq!(File::new(&[]).programs().count(), 0);
    }

    /// Files are padded, concatenated and appended to. Bytes outside a frame are
    /// not what the file is for, and are walked past.
    #[test]
    fn bytes_between_frames_are_skipped() {
        let bytes = pack(&[named("One")]);
        let mut padded = alloc::vec![0x00, 0x00];
        padded.extend_from_slice(&bytes);
        padded.extend_from_slice(&[0x00, 0x00, 0x00]);

        let entries: Vec<Entry> = File::new(&padded)
            .programs()
            .collect::<Result<_>>()
            .expect("the frame between the padding");
        let entry = entries.first().expect("the one program");
        assert_eq!(entries.len(), 1);
        assert_eq!(entry.program.name().as_str(), "One");
    }

    #[test]
    fn an_unclosed_frame_is_reported_where_it_starts() {
        let bytes = pack(&[named("One"), named("Two")]);
        let truncated = bytes.get(..bytes.len() - 1).expect("a shortened file");

        let mut frames = File::new(truncated).frames();
        assert!(frames.next().expect("the first frame").is_ok());
        assert_eq!(frames.offset(), 0);

        assert_eq!(frames.next(), Some(Err(Error::Unframed)));
        assert_eq!(frames.offset(), program_frame_len(&named("One")));
        assert_eq!(frames.next(), None, "nothing follows an unclosed frame");
    }

    /// A pack may carry more than programs, and a host asking for programs is
    /// not asking about the rest.
    #[test]
    fn frames_carrying_no_program_are_skipped() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(
            &Frame::new(DeviceId::Unit(0), Message::EditBufferDumpRequest).to_vec(),
        );
        bytes.extend_from_slice(&pack(&[named("One")]));
        bytes.extend_from_slice(
            &Frame::new(DeviceId::Unit(0), Message::GlobalParameterDumpRequest).to_vec(),
        );

        assert_eq!(File::new(&bytes).frames().count(), 3);
        let entries: Vec<Entry> = File::new(&bytes)
            .programs()
            .collect::<Result<_>>()
            .expect("the one program");
        assert_eq!(entries.len(), 1);
    }

    /// The edit buffer dump names no slot, and is a program all the same.
    #[test]
    fn an_edit_buffer_dump_is_a_program_without_a_slot() {
        let program = named("Edit");
        let mut bytes = [0; MAX_PROGRAM_FRAME_LEN];
        let mut writer = Writer::new(&mut bytes, DeviceId::Unit(0));
        writer
            .push_edit_buffer(&program)
            .expect("room for one program");
        let written = writer.finish();

        let entry = File::new(bytes.get(..written).expect("what was written"))
            .programs()
            .next()
            .expect("one program")
            .expect("it parses");
        assert_eq!(entry.slot, None);
        assert_eq!(entry.program, program);
    }

    /// A dump cut off by the next one is one bad frame; the next one is kept.
    #[test]
    fn a_truncated_frame_does_not_take_the_one_after_it() {
        let whole = pack(&[named("One")]);
        let mut bytes = whole.get(..100).expect("a frame is longer").to_vec();
        bytes.extend_from_slice(&whole);

        let mut programs = File::new(&bytes).programs();
        assert_eq!(programs.next(), Some(Err(Error::Unframed)));
        assert_eq!(programs.offset(), 0);
        let entry = programs
            .next()
            .expect("the whole frame")
            .expect("it parses");
        assert_eq!(entry.program.name().as_str(), "One");
        assert_eq!(programs.offset(), 100);
        assert_eq!(programs.next(), None);
    }

    /// A file another device wrote is one bad frame, not the end of the walk.
    #[test]
    fn a_foreign_frame_is_an_error_and_the_walk_goes_on() {
        // A Yamaha frame, which is not this library's to read.
        let mut bytes = alloc::vec![0xF0, 0x43, 0x10, 0x20, 0x00, 0x01, 0x02, 0xF7];
        bytes.extend_from_slice(&pack(&[named("One")]));

        let mut programs = File::new(&bytes).programs();
        assert_eq!(
            programs.next(),
            Some(Err(Error::Foreign {
                manufacturer: [0x43, 0x10, 0x20],
                model: 0x00,
            }))
        );
        let entry = programs
            .next()
            .expect("the DeepMind frame after it")
            .expect("it parses");
        assert_eq!(entry.program.name().as_str(), "One");
    }
}
