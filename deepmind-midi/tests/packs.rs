//! Golden tests for the `.syx` layer.
//!
//! Two sources, because neither alone is enough.
//!
//! A synthetic pack is committed under `tests/fixtures`. It is small, it is
//! built by a recipe written down here, and its job is to freeze what this
//! library writes: a change in the encoder shows up as a changed fixture in
//! review rather than as a file the next release cannot read. `FIXTURE_HASH` is
//! the same guarantee for the fixture itself, so regenerating it is a visible
//! edit rather than a silent one.
//!
//! The factory preset packs are the other source, and they are not committed:
//! redistribution rights on Behringer sound banks are unclear and a library
//! repository is the wrong place to find out. Point `DEEPMIND_PACKS` at a
//! directory of `.syx` files and [`factory_packs_read_back_unchanged`] reads
//! every one of them; without it the test skips.
//!
//! ```sh
//! DEEPMIND_PACKS=~/Downloads/deepmind-presets cargo test -p deepmind-midi --test packs
//! UPDATE_FIXTURES=1 cargo test -p deepmind-midi --test packs   # rewrite the fixture
//! ```

#![cfg(feature = "std")]
#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "a failed expectation is the test failure"
)]

use std::fs;
use std::path::{Path, PathBuf};

use deepmind_midi::ids::{Bank, DeviceId, ProgramNumber, ProtocolVersion};
use deepmind_midi::program::{Program, ProgramName};
use deepmind_midi::sysex::{Frame, Message};
use deepmind_midi::syx::{File, MAX_BANK_LEN, Writer};

/// FNV-1a of the committed fixture.
///
/// A literal rather than a computation, so rebuilding the fixture has to be
/// agreed to in a diff. Print the new one with `UPDATE_FIXTURES=1`.
const FIXTURE_HASH: u64 = 0x6ae3_fe5d_c623_7c60;

/// Programs the synthetic pack stores in bank A, under comms protocol version 6.
const V6_PROGRAMS: u8 = 4;

/// Programs it stores in bank B, under comms protocol version 7.
const V7_PROGRAMS: u8 = 2;

fn fixture_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/synthetic-bank.syx")
}

/// Deterministic bytes, the same on every platform and every run.
fn filled(version: ProtocolVersion, seed: u64, name: &str) -> Program {
    let mut state = seed;
    let len = version.program_data_len();
    let bytes: Vec<u8> = (0..len)
        .map(|_| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            u8::try_from(state >> 56).unwrap_or(0)
        })
        .collect();

    let mut program = Program::from_bytes(version, &bytes).expect("bytes of the right length");
    program.set_name(ProgramName::new(name).expect("a name the display can show"));
    program
}

/// Builds the synthetic pack.
///
/// Four version 6 programs stored from `A1`, two version 7 programs stored from
/// `B1`, and one version 6 edit buffer dump, which names no slot. The bytes are
/// pseudo-random rather than valid: hardware is entitled to send a value no
/// table lists, and a pack the library cannot carry through unchanged is a bug
/// whether or not every byte is one the manual names.
fn synthetic_pack() -> Vec<u8> {
    let mut bytes = vec![0; MAX_BANK_LEN];
    let mut writer = Writer::new(&mut bytes, DeviceId::Broadcast);

    for index in 0..V6_PROGRAMS {
        let program = filled(
            ProtocolVersion::V6,
            u64::from(index) + 1,
            &format!("Synthetic {}", index + 1),
        );
        writer
            .push_program(
                Bank::A,
                ProgramNumber::new(index).expect("a program in range"),
                &program,
            )
            .expect("room for a program");
    }
    for index in 0..V7_PROGRAMS {
        let program = filled(
            ProtocolVersion::V7,
            u64::from(index) + 100,
            &format!("Reserved {}", index + 1),
        );
        writer
            .push_program(
                Bank::new(1).expect("bank B"),
                ProgramNumber::new(index).expect("a program in range"),
                &program,
            )
            .expect("room for a program");
    }
    writer
        .push_edit_buffer(&filled(ProtocolVersion::V6, 999, "Edit Buffer"))
        .expect("room for the edit buffer");

    let written = writer.finish();
    bytes.truncate(written);
    bytes
}

/// FNV-1a, 64-bit. Eight lines beats a dependency for a fixture checksum.
fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

#[test]
fn the_committed_fixture_is_what_this_library_writes() {
    let built = synthetic_pack();
    let path = fixture_path();

    if std::env::var_os("UPDATE_FIXTURES").is_some() {
        fs::create_dir_all(path.parent().expect("a fixtures directory"))
            .expect("the fixtures directory");
        fs::write(&path, &built).expect("writing the fixture");
        panic!(
            "fixture rewritten; set FIXTURE_HASH to {:#018x}",
            fnv1a(&built)
        );
    }

    let committed = fs::read(&path).unwrap_or_else(|error| {
        panic!(
            "cannot read {}: {error}. Rebuild it with UPDATE_FIXTURES=1.",
            path.display()
        )
    });

    assert_eq!(
        fnv1a(&committed),
        FIXTURE_HASH,
        "the committed fixture is not the one FIXTURE_HASH describes"
    );
    assert_eq!(
        committed, built,
        "what this library writes is no longer the committed fixture; \
         rebuild it with UPDATE_FIXTURES=1 if the change is meant"
    );
}

#[test]
fn the_fixture_carries_the_programs_it_was_built_from() {
    let bytes = fs::read(fixture_path()).expect("the committed fixture");
    let entries: Vec<_> = File::new(&bytes)
        .programs()
        .collect::<Result<_, _>>()
        .expect("every program parses");

    let total = usize::from(V6_PROGRAMS) + usize::from(V7_PROGRAMS) + 1;
    assert_eq!(entries.len(), total);

    let stored = usize::from(V6_PROGRAMS) + usize::from(V7_PROGRAMS);
    assert_eq!(
        entries.iter().filter(|entry| entry.slot.is_some()).count(),
        stored,
        "every program but the edit buffer names a slot"
    );

    assert_eq!(
        entries
            .first()
            .expect("the first program")
            .program
            .name()
            .as_str(),
        "Synthetic 1"
    );
    assert_eq!(
        entries
            .last()
            .expect("the edit buffer")
            .program
            .name()
            .as_str(),
        "Edit Buffer"
    );

    // The three reserved bytes a version 7 dump carries are carried, not decoded.
    let reserved: Vec<_> = entries
        .iter()
        .filter(|entry| entry.program.version() == ProtocolVersion::V7)
        .collect();
    assert_eq!(reserved.len(), usize::from(V7_PROGRAMS));
    assert!(
        reserved
            .iter()
            .all(|entry| entry.program.reserved().len() == 3)
    );
}

#[test]
fn factory_packs_read_back_unchanged() {
    let Some(dir) = std::env::var_os("DEEPMIND_PACKS") else {
        eprintln!("skipped: set DEEPMIND_PACKS to a directory of .syx files to run this");
        return;
    };

    let mut files = 0;
    let mut programs = 0;
    let mut repacked_identically = 0;

    for entry in fs::read_dir(&dir).expect("the pack directory") {
        let path = entry.expect("a directory entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("syx") {
            continue;
        }
        let bytes = fs::read(&path).expect("a pack");
        files += 1;

        let mut frames = File::new(&bytes).frames();
        while let Some(frame) = frames.next() {
            let frame = frame.unwrap_or_else(|error| {
                panic!("{}: byte {}: {error}", path.display(), frames.offset())
            });
            let (Message::ProgramDumpResponse { packed, .. }
            | Message::EditBufferDumpResponse { packed, .. }) = frame.message
            else {
                continue;
            };
            programs += 1;

            let program = Program::from_dump(&frame.message).unwrap_or_else(|error| {
                panic!("{}: byte {}: {error}", path.display(), frames.offset())
            });

            // The program the file carries is the program this library rebuilds.
            let mut out = vec![0; Program::PACKED_MAX_LEN];
            let len = program.pack_into(&mut out).expect("room to pack");
            let again =
                Program::from_packed(program.version(), &out[..len]).expect("what was just packed");
            assert_eq!(
                again,
                program,
                "{}: byte {}",
                path.display(),
                frames.offset()
            );

            // And the bytes, where the file pads its packed runs the way this
            // library does. The manual prints one length for the program dump
            // that neither padding nor truncating produces, so a file written to
            // that figure holds the same program in a different number of bytes.
            if packed.len() == len {
                let start = frames.offset();
                let original = bytes
                    .get(start..start + frame.encoded_len())
                    .expect("the frame as the file holds it");
                assert_eq!(frame.to_vec(), original, "{}: byte {start}", path.display());
                repacked_identically += 1;
            }
        }
    }

    assert!(files > 0, "no .syx files in {}", Path::new(&dir).display());
    assert!(programs > 0, "no programs in {files} pack(s)");
    eprintln!(
        "{programs} programs in {files} pack(s); {repacked_identically} re-encoded byte for byte"
    );
}

/// A `Frame` reaches the file's bytes only through the reader, so this is what
/// says the reader hands back frames the file actually holds.
#[test]
fn every_frame_in_the_fixture_re_encodes_to_where_it_came_from() {
    let bytes = fs::read(fixture_path()).expect("the committed fixture");
    let mut frames = File::new(&bytes).frames();

    let mut count = 0;
    while let Some(frame) = frames.next() {
        let frame: Frame<'_> = frame.expect("every frame parses");
        let start = frames.offset();
        let original = bytes
            .get(start..start + frame.encoded_len())
            .expect("the frame as the fixture holds it");
        assert_eq!(frame.to_vec(), original);
        count += 1;
    }
    assert_eq!(
        count,
        usize::from(V6_PROGRAMS) + usize::from(V7_PROGRAMS) + 1
    );
}
