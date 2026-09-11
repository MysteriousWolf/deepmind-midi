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
//!
//! Further layers (`wire`, `sysex`, `param`, `program`, `device`, `syx`) land in
//! subsequent changes, generated where possible from the machine-readable
//! specification in `spec/`. See `docs/architecture.md` for the design and
//! `docs/midi-spec.md` for the protocol itself.
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

pub use error::{Error, Result};
pub use ids::{Bank, DeviceId, Model, ProgramNumber, ProtocolVersion};
