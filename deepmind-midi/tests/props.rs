//! Randomised invariant tests for the codecs.
//!
//! The invariants here are the ones the library's safety claim rests on: a MIDI
//! port is untrusted input, so no byte sequence may panic, and every codec that
//! has an inverse must be one. Both are statements about all inputs, and a
//! handful of hand-written cases cannot say them.
//!
//! The generator is a sixty-four bit xorshift seeded from a constant, so a run
//! is the same run on every machine and a failure is reproducible from the seed
//! printed with it. That is deliberate: `proptest` would shrink a failing case
//! for us, and it would also put a dependency, a lockfile entry and a minimum
//! toolchain of its own into a crate that currently has none of those. The
//! shrinking is worth less here than the zero dependencies, because these
//! inputs are byte strings, and a byte string that fails prints in full.
//!
//! What this file does not do is search adversarially. [`fuzz/`](../../fuzz) is
//! where that lives; these tests are the part of it that runs on every commit.

#![cfg(all(feature = "std", feature = "alloc"))]
#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "a failed expectation is the test failure"
)]

use deepmind_midi::ids::{Bank, DeviceId, ProgramNumber, ProtocolVersion};
use deepmind_midi::param::{PARAMETER_COUNT, ParamId};
use deepmind_midi::program::{Program, ProgramName};
use deepmind_midi::sysex::{self, Frame, Message, packed};
use deepmind_midi::syx::{File, MAX_BANK_LEN, Writer};
use deepmind_midi::wire::{Channel, ChannelMessage, Decoder, Event, Realtime, SystemCommon};

/// Every real-time status except the two the standard leaves undefined, which
/// the decoder drops rather than reports.
const REALTIME: [u8; 6] = [0xF8, 0xFA, 0xFB, 0xFC, 0xFE, 0xFF];

/// A `SysEx` buffer small enough that a realistic frame overruns it.
const SMALL_BUFFER: usize = 32;

/// Cases each property runs. Large enough to walk into the corners, small
/// enough that the whole file is well under a second.
const CASES: usize = 2_000;

/// Deterministic byte source.
///
/// xorshift64*, which is four lines and good enough to produce byte strings
/// that no hand-written case would think of. Nothing here needs a real
/// generator, and a real generator would be a dependency.
struct Rng(u64);

impl Rng {
    /// Seeds the generator. A zero seed is replaced, since xorshift cannot
    /// leave it.
    const fn new(seed: u64) -> Self {
        Self(if seed == 0 {
            0x9E37_79B9_7F4A_7C15
        } else {
            seed
        })
    }

    /// Returns the next value in the sequence.
    fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Returns a byte, taken from the middle of the word where the bits are
    /// best mixed.
    fn byte(&mut self) -> u8 {
        self.next_u64().to_le_bytes()[3]
    }

    /// Returns a value below `bound`, which must not be zero.
    fn below(&mut self, bound: usize) -> usize {
        let bound = u64::try_from(bound).unwrap_or(u64::MAX).max(1);
        usize::try_from(self.next_u64() % bound).unwrap_or_default()
    }

    /// Returns `len` bytes.
    fn bytes(&mut self, len: usize) -> Vec<u8> {
        (0..len).map(|_| self.byte()).collect()
    }

    /// Returns between zero and `max` bytes.
    fn bytes_up_to(&mut self, max: usize) -> Vec<u8> {
        let len = self.below(max + 1);
        self.bytes(len)
    }
}

/// An event with nothing borrowed, so two decoder runs can be compared.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Owned {
    Channel(Channel, ChannelMessage),
    SysEx(Vec<u8>),
    Common(SystemCommon),
    Realtime(Realtime),
    /// A rejection, as its `Display` text. The error type is what is being
    /// compared, not its payload's formatting, but the text distinguishes the
    /// variants and needs no `PartialEq` the library does not have.
    Error(String),
}

impl Owned {
    /// Copies an event out of the decoder's buffer.
    fn from(event: deepmind_midi::Result<Event<'_>>) -> Self {
        match event {
            Ok(Event::Channel { channel, message }) => Self::Channel(channel, message),
            Ok(Event::SysEx(frame)) => Self::SysEx(frame.to_vec()),
            Ok(Event::Common(message)) => Self::Common(message),
            Ok(Event::Realtime(message)) => Self::Realtime(message),
            Err(error) => Self::Error(error.to_string()),
            // `Event` is `non_exhaustive`; a new variant should fail loudly
            // rather than compare equal to itself by accident.
            _ => panic!("unhandled event variant"),
        }
    }

    /// Returns whether this is a real-time event, which may appear anywhere.
    const fn is_realtime(&self) -> bool {
        matches!(self, Self::Realtime(_))
    }
}

/// Feeds `bytes` to a fresh decoder in one call and collects what comes out.
fn decode_all(bytes: &[u8]) -> Vec<Owned> {
    let mut decoder: Decoder = Decoder::new();
    let mut events = Vec::new();
    decoder.feed(bytes, |event| events.push(Owned::from(event)));
    events
}

/// Feeds `bytes` in chunks of the sizes `chunks` gives, cycling.
fn decode_chunked(bytes: &[u8], chunks: &[usize]) -> Vec<Owned> {
    let mut decoder: Decoder = Decoder::new();
    let mut events = Vec::new();
    let mut rest = bytes;
    let mut sizes = chunks.iter().copied().cycle();
    while !rest.is_empty() {
        let take = sizes.next().unwrap_or(1).clamp(1, rest.len());
        let (head, tail) = rest.split_at(take);
        decoder.feed(head, |event| events.push(Owned::from(event)));
        rest = tail;
    }
    events
}

// ---------------------------------------------------------------------------
// The packed MS-bit codec
// ---------------------------------------------------------------------------

#[test]
fn packing_then_unpacking_restores_the_raw_bytes() {
    let mut rng = Rng::new(0x7061_636B_6564_0001);
    for _ in 0..CASES {
        let raw = rng.bytes_up_to(300);
        let wire = packed::pack(&raw);

        assert_eq!(wire.len(), packed::packed_len(raw.len()));
        assert!(
            wire.iter().all(|byte| *byte < 0x80),
            "packed output must be seven-bit: {wire:02X?}"
        );

        let back = packed::unpack(&wire).expect("a run this encoder wrote unpacks");
        assert!(
            back.starts_with(&raw),
            "round-trip lost data\n raw {raw:02X?}\nback {back:02X?}"
        );
        assert!(
            back[raw.len()..].iter().all(|byte| *byte == 0),
            "padding must be zero: {back:02X?}"
        );
    }
}

#[test]
fn unpacking_then_packing_restores_the_packed_run() {
    let mut rng = Rng::new(0x7061_636B_6564_0002);
    for _ in 0..CASES {
        // A run as a synthesizer would send it: seven-bit, whole groups.
        let groups = rng.below(40);
        let wire: Vec<u8> = (0..groups * packed::PACKED_GROUP)
            .map(|_| rng.byte() & 0x7F)
            .collect();

        let raw = packed::unpack(&wire).expect("a whole number of groups unpacks");
        assert_eq!(raw.len(), groups * packed::RAW_GROUP);
        assert_eq!(packed::pack(&raw), wire);
    }
}

#[test]
fn unpacking_rejects_exactly_the_impossible_lengths() {
    let mut rng = Rng::new(0x7061_636B_6564_0003);
    for _ in 0..CASES {
        let wire = rng.bytes_up_to(300);
        let expected = wire.len() % packed::PACKED_GROUP != 1;
        assert_eq!(
            packed::unpack(&wire).is_ok(),
            expected,
            "length {} should {}unpack",
            wire.len(),
            if expected { "" } else { "not " }
        );
    }
}

#[test]
fn packing_needs_exactly_the_length_it_reports() {
    let mut rng = Rng::new(0x7061_636B_6564_0004);
    for _ in 0..CASES {
        let raw = rng.bytes_up_to(120);
        let needed = packed::packed_len(raw.len());

        let mut exact = vec![0; needed];
        assert_eq!(packed::pack_into(&raw, &mut exact), Ok(needed));

        if needed > 0 {
            let mut short = vec![0; needed - 1];
            assert!(
                packed::pack_into(&raw, &mut short).is_err(),
                "a buffer one byte short must be refused"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// The wire decoder
// ---------------------------------------------------------------------------

/// Builds a stream a real port would carry: whole `SysEx` frames, channel
/// messages under running status, and junk between them.
///
/// Random bytes alone are not this. In two hundred random bytes an `F0` and a
/// matching `F7` almost never line up, so a decoder fed only noise is never
/// asked to reassemble anything, and reassembly is the part that is hard.
fn random_stream(rng: &mut Rng, corpus: &[Vec<u8>]) -> Vec<u8> {
    let mut stream = Vec::new();
    for _ in 0..rng.below(6) {
        match rng.below(4) {
            0 => {
                let pick = rng.below(corpus.len());
                stream.extend_from_slice(&corpus[pick]);
            }
            1 => {
                let pick = rng.below(corpus.len());
                stream.extend(mutate(rng, &corpus[pick]));
            }
            2 => {
                // Running status: one status byte and several pairs after it.
                stream.push(0x90 | (rng.byte() & 0x0F));
                for _ in 0..=rng.below(5) {
                    stream.push(rng.byte() & 0x7F);
                    stream.push(rng.byte() & 0x7F);
                }
            }
            _ => stream.extend(rng.bytes_up_to(24)),
        }
    }
    stream
}

#[test]
fn chunking_does_not_change_what_is_decoded() {
    let corpus = valid_frames();
    let mut rng = Rng::new(0x7769_7265_0001);
    let mut sysex = 0;

    for _ in 0..CASES {
        let stream = random_stream(&mut rng, &corpus);
        let whole = decode_all(&stream);
        sysex += whole
            .iter()
            .filter(|event| matches!(event, Owned::SysEx(_)))
            .count();

        for chunks in [
            vec![1],
            vec![2],
            vec![3, 1],
            vec![1, 7, 2, 13],
            vec![rng.below(16) + 1, rng.below(64) + 1],
        ] {
            assert_eq!(
                decode_chunked(&stream, &chunks),
                whole,
                "chunking by {chunks:?} changed the events for {stream:02X?}"
            );
        }
    }

    // Chunking only means anything where there is something to reassemble.
    assert!(
        sysex > CASES / 4,
        "only {sysex} frames were reassembled; the generator has stopped producing them"
    );
}

#[test]
fn real_time_bytes_disturb_nothing_around_them() {
    let corpus = valid_frames();
    let mut rng = Rng::new(0x7769_7265_0002);

    for _ in 0..CASES {
        let stream = random_stream(&mut rng, &corpus);
        let plain: Vec<Owned> = decode_all(&stream)
            .into_iter()
            .filter(|event| !event.is_realtime())
            .collect();

        // The same stream with real-time bytes sprinkled between any two bytes,
        // including inside a `SysEx` frame.
        let mut peppered = Vec::with_capacity(stream.len() * 2);
        for byte in &stream {
            if rng.below(4) == 0 {
                peppered.push(REALTIME[rng.below(REALTIME.len())]);
            }
            peppered.push(*byte);
        }

        let interleaved: Vec<Owned> = decode_all(&peppered)
            .into_iter()
            .filter(|event| !event.is_realtime())
            .collect();

        assert_eq!(
            interleaved, plain,
            "real-time bytes changed the stream\n{stream:02X?}\n{peppered:02X?}"
        );
    }
}

#[test]
fn every_channel_message_survives_the_wire() {
    let mut rng = Rng::new(0x7769_7265_0003);
    for _ in 0..CASES {
        let channel = Channel::new(rng.byte() % 16).expect("a channel below sixteen");
        let first = rng.byte() & 0x7F;
        let second = rng.byte() & 0x7F;
        let message = match rng.below(7) {
            0 => ChannelMessage::NoteOff {
                key: first,
                velocity: second,
            },
            1 => ChannelMessage::NoteOn {
                key: first,
                velocity: second,
            },
            2 => ChannelMessage::PolyKeyPressure {
                key: first,
                pressure: second,
            },
            3 => ChannelMessage::ControlChange {
                controller: first,
                value: second,
            },
            4 => ChannelMessage::ProgramChange { program: first },
            5 => ChannelMessage::ChannelPressure { pressure: first },
            _ => ChannelMessage::PitchBend {
                value: u16::from(first) | (u16::from(second) << 7),
            },
        };

        let mut out = [0; 3];
        let written = message
            .encode_into(channel, &mut out)
            .expect("three bytes is every channel message");
        assert_eq!(written, message.encoded_len());

        assert_eq!(
            decode_all(&out[..written]),
            vec![Owned::Channel(channel, message)],
            "message did not survive: {message:?}"
        );
    }
}

#[test]
fn a_frame_too_long_for_the_buffer_is_dropped_and_the_next_one_is_not() {
    let mut rng = Rng::new(0x7769_7265_0004);

    for _ in 0..CASES {
        let mut stream = vec![0xF0];
        stream.extend((0..SMALL_BUFFER * 2).map(|_| rng.byte() & 0x7F));
        stream.push(0xF7);

        let good = [0xF0, 0x00, 0x20, 0x32, 0x20, 0x00, 0x03, 0xF7];
        stream.extend_from_slice(&good);

        let mut decoder: Decoder<SMALL_BUFFER> = Decoder::new();
        let mut errors = 0;
        let mut frames = Vec::new();
        decoder.feed(&stream, |event| match event {
            Ok(Event::SysEx(frame)) => frames.push(frame.to_vec()),
            Err(_) => errors += 1,
            _ => {}
        });

        assert_eq!(errors, 1, "the overlong frame is rejected exactly once");
        assert_eq!(
            frames,
            vec![good.to_vec()],
            "the decoder must recover in time for the next frame"
        );
    }
}

#[test]
fn a_reset_decoder_decodes_as_a_new_one_does() {
    let corpus = valid_frames();
    let mut rng = Rng::new(0x7769_7265_0005);
    for _ in 0..CASES {
        // Half a frame, so the reset has something to forget.
        let mut junk = random_stream(&mut rng, &corpus);
        junk.truncate(junk.len() / 2);
        let stream = random_stream(&mut rng, &corpus);

        let mut decoder: Decoder = Decoder::new();
        decoder.feed(&junk, |_| {});
        decoder.reset();
        assert!(!decoder.in_sysex());

        let mut events = Vec::new();
        decoder.feed(&stream, |event| events.push(Owned::from(event)));
        assert_eq!(events, decode_all(&stream));
    }
}

// ---------------------------------------------------------------------------
// `SysEx` framing
// ---------------------------------------------------------------------------

/// Frames a synthesizer really sends, encoded.
///
/// Near misses are what a parser gets wrong, and a near miss has to start from
/// a hit: bytes shaped by guesswork are rejected at the manufacturer ID and
/// never reach the code that decides what a payload means.
fn valid_frames() -> Vec<Vec<u8>> {
    let device = DeviceId::Unit(3);
    let bank = Bank::A;
    let program = ProgramNumber::FIRST;
    let v7 = ProtocolVersion::V7;

    let dump = packed::pack(&[0x5A; 245]);
    let names = packed::pack(&[b'x'; sysex::BANK_NAMES_LEN]);
    let one_name = packed::pack(&[b'y'; sysex::PROGRAM_NAME_LEN]);
    let globals = packed::pack(&[0x11; sysex::GLOBAL_DATA_LEN]);
    let chord = packed::pack(&[0xFF; sysex::CHORD_MEMORY_LEN]);

    let messages = [
        Message::ProgramDumpRequest { bank, program },
        Message::EditBufferDumpRequest,
        Message::GlobalParameterDumpRequest,
        Message::BankProgramNamesDumpRequest { bank },
        Message::SingleProgramNameDumpRequest { bank, program },
        Message::ChordMemoryDumpRequest,
        Message::CalibrationDataDumpRequest,
        Message::ProgramDumpResponse {
            version: v7,
            bank,
            program,
            packed: &dump,
        },
        Message::EditBufferDumpResponse {
            version: v7,
            packed: &dump,
        },
        Message::GlobalParameterDumpResponse {
            version: v7,
            packed: &globals,
        },
        Message::BankProgramNamesDumpResponse {
            version: v7,
            bank,
            packed: &names,
        },
        Message::SingleProgramNameDumpResponse {
            version: v7,
            bank,
            packed: &one_name,
        },
        Message::ChordMemoryDumpResponse {
            version: v7,
            packed: &chord,
        },
    ];

    messages
        .into_iter()
        .map(|message| Frame::new(device, message).to_vec())
        .collect()
}

/// Corrupts `bytes` the way a cable, a firmware bug or an attacker would.
fn mutate(rng: &mut Rng, bytes: &[u8]) -> Vec<u8> {
    let mut out = bytes.to_vec();
    match rng.below(6) {
        // Flip one byte, which is the near miss that matters most: a length, a
        // command or a version that is one off.
        0 if !out.is_empty() => {
            let at = rng.below(out.len());
            out[at] = rng.byte();
        }
        // Flip a handful.
        1 if !out.is_empty() => {
            for _ in 0..=rng.below(7) {
                let at = rng.below(out.len());
                out[at] ^= 1 << rng.below(8);
            }
        }
        // Cut it short, anywhere.
        2 if !out.is_empty() => out.truncate(rng.below(out.len())),
        // Make the payload longer than the command allows.
        3 => {
            let at = rng.below(out.len() + 1);
            let extra = rng.bytes_up_to(20);
            out.splice(at..at, extra);
        }
        // Drop a run out of the middle.
        4 if out.len() > 2 => {
            let at = rng.below(out.len() - 1);
            let len = rng.below(out.len() - at);
            out.drain(at..at + len);
        }
        // Leave it alone, so the round-trip is exercised too.
        _ => {}
    }
    out
}

#[test]
fn a_frame_this_library_writes_is_a_frame_it_reads() {
    for bytes in valid_frames() {
        let frame = Frame::parse(&bytes).expect("a frame this library wrote parses");
        assert_eq!(frame.to_vec(), bytes);
        assert_eq!(frame.encoded_len(), bytes.len());
    }
}

#[test]
fn parsing_corrupted_frames_never_panics_and_never_lies() {
    let corpus = valid_frames();
    let mut rng = Rng::new(0x0073_7973_6578_0001);
    let mut parsed = 0;

    for _ in 0..CASES {
        let bytes = if rng.below(8) == 0 {
            rng.bytes_up_to(400)
        } else {
            let pick = rng.below(corpus.len());
            mutate(&mut rng, &corpus[pick])
        };

        if let Ok(frame) = Frame::parse(&bytes) {
            parsed += 1;
            // Anything that parsed has to re-encode to what it came from, or
            // a host is acting on a payload nobody sent.
            assert_eq!(
                frame.to_vec(),
                bytes,
                "a parsed frame did not re-encode: {bytes:02X?}"
            );
            assert_eq!(frame.encoded_len(), bytes.len());
        }
    }

    // The property is only worth anything if the mutations land inside the
    // parser rather than bouncing off its first check.
    assert!(
        parsed > CASES / 20,
        "only {parsed} of {CASES} mutants parsed; the generator has stopped reaching the parser"
    );
}

#[test]
fn every_request_survives_a_frame() {
    let mut rng = Rng::new(0x0073_7973_6578_0002);
    for _ in 0..CASES {
        let device = match rng.below(3) {
            0 => DeviceId::Broadcast,
            _ => DeviceId::from_byte(rng.byte() % 16).expect("a device id below sixteen"),
        };
        let bank = Bank::new(rng.byte() % 8).expect("a bank below eight");
        let number = ProgramNumber::new(rng.byte() % 128).expect("a program below 128");

        let message = match rng.below(6) {
            0 => Message::ProgramDumpRequest {
                bank,
                program: number,
            },
            1 => Message::EditBufferDumpRequest,
            2 => Message::GlobalParameterDumpRequest,
            3 => Message::BankProgramNamesDumpRequest { bank },
            4 => Message::ChordMemoryDumpRequest,
            _ => Message::CalibrationDataDumpRequest,
        };

        let frame = Frame::new(device, message);
        let bytes = frame.to_vec();
        assert_eq!(bytes.len(), frame.encoded_len());

        let parsed = Frame::parse(&bytes).expect("a frame this library wrote parses");
        assert_eq!(parsed, frame, "message did not survive: {message:?}");
    }
}

// ---------------------------------------------------------------------------
// Programs
// ---------------------------------------------------------------------------

/// Returns a program filled with bytes from `rng`, every value in range.
fn random_program(rng: &mut Rng, version: ProtocolVersion) -> Program {
    let len = version.program_data_len().expect("a version with a length");
    let mut program = Program::from_bytes(version, &vec![0; len]).expect("a zeroed program");
    for parameter in ParamId::ALL {
        program.set_clamped(parameter, rng.byte());
    }
    program
}

#[test]
fn a_program_is_the_bytes_it_was_built_from() {
    let mut rng = Rng::new(0x7072_6F67_0001);
    for _ in 0..CASES {
        let version = if rng.below(2) == 0 {
            ProtocolVersion::V6
        } else {
            ProtocolVersion::V7
        };
        let len = version.program_data_len().expect("a version with a length");
        let bytes = rng.bytes(len);

        let program = Program::from_bytes(version, &bytes).expect("the documented length");
        assert_eq!(program.as_bytes(), &bytes[..]);
        assert_eq!(program.version(), version);

        // And through the packed codec, which is how it arrives from a port.
        let wire = packed::pack(program.as_bytes());
        let back = Program::from_packed(version, &wire).expect("what this library just packed");
        assert_eq!(back.as_bytes(), program.as_bytes());
        assert_eq!(back.reserved(), program.reserved());
    }
}

#[test]
fn a_wrong_length_is_refused_rather_than_padded() {
    let mut rng = Rng::new(0x7072_6F67_0002);
    for _ in 0..CASES {
        let len = rng.below(400);
        let bytes = rng.bytes(len);
        for version in [ProtocolVersion::V6, ProtocolVersion::V7] {
            let expected = version.program_data_len() == Some(len);
            assert_eq!(
                Program::from_bytes(version, &bytes).is_ok(),
                expected,
                "{len} bytes under version {version:?}"
            );
        }
    }
}

#[test]
fn the_diff_is_exactly_what_it_takes_to_become_the_target() {
    let mut rng = Rng::new(0x7072_6F67_0003);
    for _ in 0..CASES {
        let source = random_program(&mut rng, ProtocolVersion::V7);
        let target = random_program(&mut rng, ProtocolVersion::V7);

        assert_eq!(
            source.changes(&source).count(),
            0,
            "a program is no distance from itself"
        );

        let mut applied = source.clone();
        for (parameter, value) in source.changes(&target) {
            assert_ne!(
                source.get(parameter),
                value,
                "the diff named a parameter that did not change"
            );
            applied
                .set(parameter, value)
                .expect("a value from a program");
        }

        for parameter in ParamId::ALL {
            assert_eq!(
                applied.get(parameter),
                target.get(parameter),
                "{parameter:?} did not arrive at the target"
            );
        }
    }
}

#[test]
fn clamping_always_lands_on_a_value_the_parameter_accepts() {
    let mut rng = Rng::new(0x7072_6F67_0004);
    for _ in 0..CASES {
        let mut program = Program::new(ProtocolVersion::V7).expect("a default program");
        for _ in 0..8 {
            let parameter = ParamId::ALL[rng.below(PARAMETER_COUNT)];
            let wanted = rng.byte();
            let landed = program.set_clamped(parameter, wanted);

            assert_eq!(program.get(parameter), landed);
            assert!(
                parameter.accepts(u16::from(landed)),
                "{parameter:?} rejects its own clamped value {landed}"
            );
            if parameter.accepts(u16::from(wanted)) {
                assert_eq!(landed, wanted, "{parameter:?} clamped a value it accepts");
            }
        }
        program.validate().expect("clamped values are valid values");
    }
}

#[test]
fn a_name_survives_the_field_it_is_stored_in() {
    let mut rng = Rng::new(0x6E61_6D65_0001);
    for _ in 0..CASES {
        let len = rng.below(ProgramName::MAX_CHARS + 1);
        // Printable ASCII, which is what the synthesizer's display has glyphs
        // for and all `ProgramName::new` accepts.
        let text: String = (0..len)
            .map(|_| char::from(0x20 + (rng.byte() % 0x5F)))
            .collect();
        let name = ProgramName::new(&text).expect("printable ASCII within sixteen characters");

        let mut program = Program::new(ProtocolVersion::V7).expect("a default program");
        program.set_name(name);
        assert_eq!(program.name(), name, "name did not survive: {text:?}");
        assert_eq!(program.name().as_str(), text);
    }
}

// ---------------------------------------------------------------------------
// `.syx` files
// ---------------------------------------------------------------------------

#[test]
fn a_written_bank_reads_back_as_what_was_written() {
    let mut rng = Rng::new(0x7379_7800_0001);
    // Fewer cases: each one writes and re-reads a whole bank.
    for _ in 0..CASES / 100 {
        let device = DeviceId::from_byte(rng.byte() % 16).expect("a device id below sixteen");
        let bank = Bank::new(rng.byte() % 8).expect("a bank below eight");
        let count = rng.below(8) + 1;

        let programs: Vec<Program> = (0..count)
            .map(|_| {
                let version = if rng.below(2) == 0 {
                    ProtocolVersion::V6
                } else {
                    ProtocolVersion::V7
                };
                random_program(&mut rng, version)
            })
            .collect();

        let mut out = vec![0; MAX_BANK_LEN];
        let mut writer = Writer::new(&mut out, device);
        for (index, program) in programs.iter().enumerate() {
            let number =
                ProgramNumber::new(u8::try_from(index).expect("fewer than 128")).expect("in range");
            writer
                .push_program(bank, number, program)
                .expect("a bank-sized buffer");
        }
        let written = writer.finish();

        let file = File::new(&out[..written]);
        let read: Vec<Program> = file
            .programs()
            .map(|entry| entry.expect("what this library just wrote").program)
            .collect();

        assert_eq!(read.len(), programs.len());
        for (back, original) in read.iter().zip(&programs) {
            assert_eq!(back.version(), original.version());
            assert_eq!(back.as_bytes(), original.as_bytes());
        }
    }
}

/// A `.syx` file holding a few programs, as this library writes one.
fn valid_bank() -> Vec<u8> {
    let mut rng = Rng::new(0x7379_7800_C0DE);
    let mut out = vec![0; MAX_BANK_LEN];
    let mut writer = Writer::new(&mut out, DeviceId::Unit(0));
    for index in 0..6 {
        let version = if index % 2 == 0 {
            ProtocolVersion::V6
        } else {
            ProtocolVersion::V7
        };
        let program = random_program(&mut rng, version);
        let number = ProgramNumber::new(index).expect("fewer than 128");
        writer
            .push_program(Bank::A, number, &program)
            .expect("a bank-sized buffer");
    }
    let written = writer.finish();
    out.truncate(written);
    out
}

#[test]
fn reading_a_corrupted_file_never_panics_and_always_terminates() {
    let bank = valid_bank();
    let mut rng = Rng::new(0x7379_7800_0002);
    let mut entries = 0;

    for _ in 0..CASES {
        let bytes = if rng.below(8) == 0 {
            rng.bytes_up_to(500)
        } else {
            mutate(&mut rng, &bank)
        };
        let file = File::new(&bytes);

        // Every iterator has to terminate, and it terminates by consuming the
        // file: no walk can yield more frames than the file has `F0` bytes to
        // start them with. Taking one more than that turns a stalled iterator
        // into a failure rather than a hang.
        #[expect(
            clippy::naive_bytecount,
            reason = "the `bytecount` crate is not worth a dependency to count \
                      openings in a five-hundred-byte test input"
        )]
        let limit = bytes.iter().filter(|byte| **byte == 0xF0).count();

        let mut offsets = Vec::new();
        let mut frames = file.frames();
        let mut seen = 0;
        while let Some(frame) = frames.next() {
            if frame.is_ok() {
                entries += 1;
            }
            offsets.push(frames.offset());
            seen += 1;
            assert!(
                seen <= limit,
                "the frame iterator yielded {seen} frames from {limit} openings"
            );
        }
        assert!(
            offsets.is_sorted(),
            "frames were reported out of order: {offsets:?}"
        );

        let mut programs = file.programs();
        let mut seen = 0;
        while programs.next().is_some() {
            seen += 1;
            assert!(
                seen <= limit,
                "the program iterator yielded {seen} programs from {limit} openings"
            );
        }
    }

    assert!(
        entries > CASES / 10,
        "only {entries} frames survived mutation; the generator has stopped reaching the reader"
    );
}
