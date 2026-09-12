//! `.syx` files: the format programs are traded in, read and written.
//!
//! A `.syx` file is a run of `SysEx` frames and nothing else. There is no
//! header, no index and no trailer, so a preset pack is 128 program dumps end to
//! end, and a single patch is one dump. That is the whole format; this module is
//! what walks it.
//!
//! ```
//! use deepmind_midi::ids::{Bank, DeviceId, ProgramNumber, ProtocolVersion};
//! use deepmind_midi::program::{Program, ProgramName};
//! use deepmind_midi::syx::{self, File, Writer};
//!
//! let mut program = Program::new(ProtocolVersion::V6)?;
//! program.set_name(ProgramName::new("Bass Sweep")?);
//!
//! // Write. The buffer is the host's; `MAX_BANK_LEN` sizes the biggest one.
//! let mut bytes = [0; syx::MAX_PROGRAM_FRAME_LEN];
//! let mut writer = Writer::new(&mut bytes, DeviceId::Broadcast);
//! writer.push_program(Bank::A, ProgramNumber::FIRST, &program)?;
//! let written = writer.finish();
//!
//! // Read. Every program the file carries, and where it says each belongs.
//! for entry in File::new(bytes.get(..written).unwrap_or_default()).programs() {
//!     let entry = entry?;
//!     println!("{:?} {}", entry.slot, entry.program.name());
//! }
//! # Ok::<(), deepmind_midi::Error>(())
//! ```
//!
//! # No IO, here as everywhere
//!
//! Opening the file is the host's job. This module starts from bytes and ends at
//! bytes, which is also what makes it work on a file that never touched a disk:
//! a pack downloaded into memory, one assembled from a device, one under test.
//!
//! # Reading is lenient about the file, strict about the frame
//!
//! Files in the wild are padded, concatenated and appended to, so bytes between
//! frames are walked past rather than refused. What is inside an `F0` is another
//! matter: a frame that does not parse is yielded as the error it failed with,
//! and the walk goes on. A file is untrusted input, and one bad frame is not a
//! reason to lose the 127 good ones.
//!
//! [`Programs`] skips frames carrying something other than a program - the
//! globals, a pattern, a bank of names - because a pack is entitled to hold
//! those and a host asking for programs is not asking about them. Reach them
//! through [`Frames`], which hands back everything the file holds.
//!
//! # Writing does not reproduce every file byte for byte
//!
//! A program that goes in comes out unchanged, and a file that goes in does not
//! always. The reason is the one open question in the packed codec: the manual
//! prints 278 packed bytes for a 242-byte program where padding gives 280, and
//! [`packed`](crate::sysex::packed) pads. A file written by something that
//! packs the other way reads back to identical programs and rewrites to a
//! different length. The programs are the file's content; the padding is not.
//!
//! # What a file does not say
//!
//! Which firmware wrote it. A dump carries the comms protocol version, which
//! decides how many bytes the program occupies, and nothing carries the firmware
//! version, which decides what three of the value tables mean. A host that knows
//! the firmware another way passes it to the accessors that take one; a host
//! that does not gets the current tables. See
//! [`param`](crate::param#firmware) for the mechanism.

mod reader;
mod writer;

pub use reader::{Entry, File, Frames, Programs};
#[cfg(feature = "alloc")]
pub use writer::bank_to_vec;
pub use writer::{
    MAX_BANK_LEN, MAX_PROGRAM_FRAME_LEN, Writer, edit_buffer_frame_len, program_frame_len,
};
