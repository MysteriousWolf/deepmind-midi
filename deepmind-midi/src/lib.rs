//! Sans-IO protocol library for Behringer `DeepMind` synthesizers.
//!
//! This crate models the `DeepMind` MIDI protocol and nothing else. It never opens a
//! port, never spawns a thread and never blocks. The host application owns the
//! transport, feeds inbound bytes in and sends outbound bytes out.
//!
//! ```text
//! DeepMind <--MIDI--> host application <--bytes--> deepmind-midi
//! ```
//!
//! # Layout
//!
//! | Module    | Responsibility                                               |
//! |-----------|--------------------------------------------------------------|
//! | [`ids`]   | Addressing primitives: device ID, model, bank, program number |
//! | [`error`] | Crate-wide error type                                        |
//! | [`wire`]  | MIDI bytes: running status, channel messages, `SysEx` reassembly |
//! | [`sysex`] | `DeepMind` framing, the packed MS-bit codec, typed messages   |
//! | [`param`] | The 242 program parameters: names, ranges, value tables, NRPN |
//!
//! Each layer depends only on those above it: [`wire`] does not know what a
//! `DeepMind` is, and [`sysex`] does not know where its bytes came from.
//!
//! ```
//! use deepmind_midi::ids::DeviceId;
//! use deepmind_midi::sysex::{Frame, Message};
//! use deepmind_midi::wire::{Decoder, Event};
//!
//! // Bytes off a port, in whatever chunks they arrived in.
//! let mut decoder: Decoder = Decoder::new();
//! decoder.feed(&[0xF0, 0x00, 0x20, 0x32, 0x20, 0x00, 0x03, 0xF7], |event| {
//!     if let Ok(Event::SysEx(bytes)) = event {
//!         let frame = Frame::parse(bytes).expect("a DeepMind frame");
//!         assert_eq!(frame.device, DeviceId::Unit(0));
//!         assert_eq!(frame.message, Message::EditBufferDumpRequest);
//!     }
//! });
//! ```
//!
//! Further layers (`program`, `device`, `syx`) land in subsequent changes,
//! generated where possible from the machine-readable specification in `spec/`.
//! See `docs/architecture.md` for the design and `docs/midi-spec.md` for the
//! protocol itself.
//!
//! # Feature flags
//!
//! - `std` (default): `std::error::Error` impls and owned collection helpers.
//! - `alloc`: APIs that allocate, such as bank decoding and `.syx` handling.
//! - `serde`: `Serialize`/`Deserialize` derives on the public data types.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod error;
pub mod ids;
pub mod param;
pub mod sysex;
pub mod wire;

pub use error::{Error, Result};
pub use ids::{Bank, DeviceId, Model, PatternNumber, ProgramNumber, ProtocolVersion};
pub use param::{Group, NrpnEdit, ParamId};
pub use sysex::{Command, Frame, Message};
pub use wire::{Channel, ChannelMessage, Decoder};
