//! Building a `.syx` file: frames written back to back into a buffer.

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "alloc")]
use crate::error::Error;
use crate::error::Result;
use crate::ids::{Bank, DeviceId, PROGRAMS_PER_BANK, ProgramNumber};
use crate::program::Program;
use crate::sysex::{Frame, Message, OVERHEAD_LEN};

/// Payload bytes a stored program dump carries before its packed run: the comms
/// protocol version, the bank and the program number.
const PROGRAM_PREFIX_LEN: usize = 3;

/// Payload bytes an edit buffer dump carries before its packed run: the comms
/// protocol version.
const EDIT_BUFFER_PREFIX_LEN: usize = 1;

/// Bytes the longest stored program dump occupies in a file.
pub const MAX_PROGRAM_FRAME_LEN: usize =
    OVERHEAD_LEN + PROGRAM_PREFIX_LEN + Program::PACKED_MAX_LEN;

/// Bytes a file of every program in a bank occupies at most.
pub const MAX_BANK_LEN: usize = MAX_PROGRAM_FRAME_LEN * PROGRAMS_PER_BANK as usize;

/// Returns the bytes [`Writer::push_program`] writes for `program`.
#[must_use]
pub fn program_frame_len(program: &Program) -> usize {
    OVERHEAD_LEN + PROGRAM_PREFIX_LEN + program.packed_len()
}

/// Returns the bytes [`Writer::push_edit_buffer`] writes for `program`.
#[must_use]
pub fn edit_buffer_frame_len(program: &Program) -> usize {
    OVERHEAD_LEN + EDIT_BUFFER_PREFIX_LEN + program.packed_len()
}

/// Writes a `.syx` file into a buffer the caller owns.
///
/// A file is its frames and nothing else - no header, no index, no trailer - so
/// writing one is writing frames in order. The buffer is the caller's because
/// only the caller knows where forty kilobytes can go;
/// [`MAX_BANK_LEN`] is what a whole bank needs and
/// [`program_frame_len`] what one program does.
///
/// ```
/// use deepmind_midi::ids::{Bank, DeviceId, ProgramNumber, ProtocolVersion};
/// use deepmind_midi::program::Program;
/// use deepmind_midi::syx::{self, Writer};
///
/// let program = Program::new(ProtocolVersion::V6)?;
///
/// let mut bytes = [0; syx::MAX_PROGRAM_FRAME_LEN * 2];
/// let mut writer = Writer::new(&mut bytes, DeviceId::Unit(0));
/// writer.push_program(Bank::A, ProgramNumber::FIRST, &program)?;
/// writer.push_program(Bank::A, ProgramNumber::new(1)?, &program)?;
/// let written = writer.finish();
///
/// assert_eq!(written, syx::program_frame_len(&program) * 2);
/// # Ok::<(), deepmind_midi::Error>(())
/// ```
#[derive(Debug)]
pub struct Writer<'a> {
    out: &'a mut [u8],
    written: usize,
    device: DeviceId,
}

impl<'a> Writer<'a> {
    /// Starts a file in `out`, addressed to `device`.
    ///
    /// The device ID goes into every frame. A file meant to be replayed to
    /// whichever synthesizer is listening wants [`DeviceId::Broadcast`]; one
    /// meant for a particular unit wants that unit.
    #[must_use]
    pub fn new(out: &'a mut [u8], device: DeviceId) -> Self {
        Self {
            out,
            written: 0,
            device,
        }
    }

    /// Returns the device ID every frame is addressed to.
    #[must_use]
    pub const fn device(&self) -> DeviceId {
        self.device
    }

    /// Returns how many bytes have been written so far.
    #[must_use]
    pub const fn written(&self) -> usize {
        self.written
    }

    /// Returns the file as written so far.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.out.get(..self.written).unwrap_or_default()
    }

    /// Appends a frame, addressing it as this writer addresses everything.
    ///
    /// Takes a message rather than a frame because the device ID is the
    /// writer's; [`Frame::encode_into`] is there for a frame that is already
    /// addressed.
    ///
    /// # Errors
    ///
    /// Returns [`Error::BufferTooSmall`](crate::Error::BufferTooSmall) when what
    /// is left of the buffer cannot hold the frame. Nothing is written in that
    /// case, and `available` is what was left rather than the size of the
    /// buffer.
    pub fn push(&mut self, message: Message<'_>) -> Result<usize> {
        let frame = Frame::new(self.device, message);
        let rest = self.out.get_mut(self.written..).unwrap_or_default();
        let len = frame.encode_into(rest)?;
        self.written = self.written.saturating_add(len);
        Ok(len)
    }

    /// Appends one stored program, as the dump that carries it from a bank.
    ///
    /// # Errors
    ///
    /// Returns [`Error::BufferTooSmall`](crate::Error::BufferTooSmall) when what
    /// is left of the buffer cannot hold [`program_frame_len`] bytes.
    pub fn push_program(
        &mut self,
        bank: Bank,
        number: ProgramNumber,
        program: &Program,
    ) -> Result<usize> {
        let mut packed = [0; Program::PACKED_MAX_LEN];
        let len = program.pack_into(&mut packed)?;
        self.push(Message::ProgramDumpResponse {
            version: program.version(),
            bank,
            program: number,
            packed: packed.get(..len).unwrap_or_default(),
        })
    }

    /// Appends one program as the edit buffer, which names no slot.
    ///
    /// This is the frame that makes a synthesizer sound like the program without
    /// storing it anywhere.
    ///
    /// # Errors
    ///
    /// Returns [`Error::BufferTooSmall`](crate::Error::BufferTooSmall) when what
    /// is left of the buffer cannot hold [`edit_buffer_frame_len`] bytes.
    pub fn push_edit_buffer(&mut self, program: &Program) -> Result<usize> {
        let mut packed = [0; Program::PACKED_MAX_LEN];
        let len = program.pack_into(&mut packed)?;
        self.push(Message::EditBufferDumpResponse {
            version: program.version(),
            packed: packed.get(..len).unwrap_or_default(),
        })
    }

    /// Ends the file and returns its length.
    #[must_use]
    pub const fn finish(self) -> usize {
        self.written
    }
}

/// Builds a preset pack: one stored program dump per program, from `first` on.
///
/// ```
/// use deepmind_midi::ids::{Bank, DeviceId, ProgramNumber, ProtocolVersion};
/// use deepmind_midi::program::Program;
/// use deepmind_midi::syx::{File, bank_to_vec};
///
/// let programs = vec![Program::new(ProtocolVersion::V6)?; 4];
/// let bytes = bank_to_vec(
///     DeviceId::Broadcast,
///     Bank::A,
///     ProgramNumber::FIRST,
///     &programs,
/// )?;
///
/// assert_eq!(File::new(&bytes).programs().count(), 4);
/// # Ok::<(), deepmind_midi::Error>(())
/// ```
///
/// # Errors
///
/// Returns [`Error::ProgramOutOfRange`](crate::Error::ProgramOutOfRange) when
/// the programs would run past the end of the bank.
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub fn bank_to_vec(
    device: DeviceId,
    bank: Bank,
    first: ProgramNumber,
    programs: &[Program],
) -> Result<Vec<u8>> {
    let last = usize::from(first.get())
        .saturating_add(programs.len())
        .saturating_sub(1);
    if !programs.is_empty() && last >= usize::from(PROGRAMS_PER_BANK) {
        return Err(Error::ProgramOutOfRange(
            u8::try_from(last).unwrap_or(u8::MAX),
        ));
    }

    let mut out = alloc::vec![0; programs.len().saturating_mul(MAX_PROGRAM_FRAME_LEN)];
    let mut writer = Writer::new(&mut out, device);
    for (index, program) in programs.iter().enumerate() {
        // Checked above: every number from `first` on is inside the bank.
        let number = ProgramNumber::new(
            first
                .get()
                .saturating_add(u8::try_from(index).unwrap_or(u8::MAX)),
        )?;
        writer.push_program(bank, number, program)?;
    }
    let written = writer.finish();
    out.truncate(written);
    Ok(out)
}

#[cfg(all(test, feature = "alloc"))]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use alloc::vec::Vec;

    use super::*;
    use crate::ids::ProtocolVersion;
    use crate::syx::File;

    fn program() -> Program {
        Program::new(ProtocolVersion::V6).expect("version 6 is supported")
    }

    fn bank_c() -> Bank {
        Bank::new(2).expect("a bank in range")
    }

    /// The two length functions state a prefix width each. The frame encoder is
    /// what actually writes them, and this is what keeps the two in step.
    #[test]
    fn the_stated_lengths_are_what_the_encoder_writes() {
        let program = program();
        let mut packed = [0; Program::PACKED_MAX_LEN];
        let len = program.pack_into(&mut packed).expect("room to pack");
        let packed = packed.get(..len).expect("what was packed");

        let stored = Frame::new(
            DeviceId::Unit(0),
            Message::ProgramDumpResponse {
                version: program.version(),
                bank: Bank::A,
                program: ProgramNumber::FIRST,
                packed,
            },
        );
        assert_eq!(stored.encoded_len(), program_frame_len(&program));

        let edit = Frame::new(
            DeviceId::Unit(0),
            Message::EditBufferDumpResponse {
                version: program.version(),
                packed,
            },
        );
        assert_eq!(edit.encoded_len(), edit_buffer_frame_len(&program));

        assert!(program_frame_len(&program) <= MAX_PROGRAM_FRAME_LEN);
    }

    #[test]
    fn a_full_bank_fits_the_stated_maximum() {
        let programs = alloc::vec![program(); usize::from(PROGRAMS_PER_BANK)];
        let bytes = bank_to_vec(
            DeviceId::Broadcast,
            Bank::A,
            ProgramNumber::FIRST,
            &programs,
        )
        .expect("a bank that fits");

        assert!(bytes.len() <= MAX_BANK_LEN);
        assert_eq!(File::new(&bytes).programs().count(), 128);
    }

    #[test]
    fn a_bank_that_would_run_past_its_end_is_refused() {
        let programs = alloc::vec![program(); 2];
        assert_eq!(
            bank_to_vec(DeviceId::Unit(0), Bank::A, ProgramNumber::LAST, &programs),
            Err(Error::ProgramOutOfRange(128))
        );
    }

    #[test]
    fn no_programs_is_an_empty_file() {
        let bytes = bank_to_vec(DeviceId::Unit(0), Bank::A, ProgramNumber::LAST, &[])
            .expect("nothing to write");
        assert!(bytes.is_empty());
    }

    /// A short buffer leaves what was already written alone, so a host can size
    /// a bigger one and start again rather than find a half-written frame.
    #[test]
    fn a_buffer_too_small_writes_nothing_further() {
        let program = program();
        let mut bytes = [0; MAX_PROGRAM_FRAME_LEN + 4];
        let mut writer = Writer::new(&mut bytes, DeviceId::Unit(0));

        writer
            .push_program(Bank::A, ProgramNumber::FIRST, &program)
            .expect("room for the first");
        let after_first = writer.written();

        let full = program_frame_len(&program);
        assert_eq!(
            writer.push_program(Bank::A, ProgramNumber::FIRST, &program),
            Err(Error::BufferTooSmall {
                needed: full,
                available: 4,
            })
        );
        assert_eq!(writer.written(), after_first);
        assert_eq!(File::new(writer.as_bytes()).programs().count(), 1);
    }

    #[test]
    fn what_is_written_is_what_is_read_back() {
        let program = program();
        let mut bytes = alloc::vec![0; MAX_BANK_LEN];
        let mut writer = Writer::new(&mut bytes, DeviceId::Broadcast);
        writer
            .push_edit_buffer(&program)
            .expect("room for the edit buffer");
        writer
            .push_program(
                bank_c(),
                ProgramNumber::new(7).expect("a program"),
                &program,
            )
            .expect("room for the program");

        let entries: Vec<_> = File::new(writer.as_bytes())
            .programs()
            .collect::<Result<_>>()
            .expect("both parse");
        assert_eq!(entries.len(), 2);
        let edit = entries.first().expect("the edit buffer");
        let stored = entries.get(1).expect("the stored program");
        assert_eq!(edit.slot, None);
        assert_eq!(stored.slot.map(|slot| slot.bank), Some(bank_c()));
        assert!(entries.iter().all(|entry| entry.program == program));
    }
}
