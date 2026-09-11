//! `DeepMind` `SysEx`: framing, the packed MS-bit codec and typed messages.
//!
//! Every `DeepMind` message is framed the same way:
//!
//! ```text
//! F0 00 20 32 20 <device> <command> <payload> F7
//!    \______/ \/
//!   Behringer  model ID, shared by all six variants
//! ```
//!
//! [`Frame::parse`] turns those bytes into a [`Message`], borrowing any bulk
//! payload where it lies rather than copying it. Bulk payloads stay packed:
//! [`packed::unpack_into`] takes them the rest of the way, into a buffer the
//! caller owns.
//!
//! The universal device inquiry is framed differently, since it is a MIDI
//! message rather than a Behringer one, and lives in [`inquiry`].
//!
//! ```
//! use deepmind_midi::ids::DeviceId;
//! use deepmind_midi::sysex::{Frame, Message};
//! use deepmind_midi::wire::{Decoder, Event};
//!
//! let bytes = [0xF0, 0x00, 0x20, 0x32, 0x20, 0x00, 0x03, 0xF7];
//! let mut requests = 0;
//!
//! let mut decoder: Decoder = Decoder::new();
//! decoder.feed(&bytes, |event| {
//!     if let Ok(Event::SysEx(frame)) = event {
//!         let frame = Frame::parse(frame).expect("a DeepMind frame");
//!         assert_eq!(frame.device, DeviceId::Unit(0));
//!         if frame.message == Message::EditBufferDumpRequest {
//!             requests += 1;
//!         }
//!     }
//! });
//!
//! assert_eq!(requests, 1);
//! ```

pub mod inquiry;
pub mod packed;

mod command;
mod message;

pub use command::{Command, Direction};
pub use inquiry::Identity;
pub use message::{
    BANK_NAMES_LEN, CHORD_MEMORY_LEN, Frame, GLOBAL_DATA_LEN, HEADER_LEN, Interface, Message,
    OVERHEAD_LEN, PATTERN_DATA_LEN, POLY_CHORD_MEMORY_LEN, PROGRAM_NAME_LEN,
};
