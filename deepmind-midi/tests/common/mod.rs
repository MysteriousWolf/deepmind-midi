//! Helpers the integration tests share.
//!
//! Every test binary compiles its own copy of this module, so a helper one
//! binary has no use for is dead code there and not a bug.

#![allow(dead_code, reason = "each test binary uses a different subset")]
#![expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]

use deepmind_midi::ids::{Bank, DeviceId, ProgramNumber, ProtocolVersion};
use deepmind_midi::program::{Program, ProgramName};
use deepmind_midi::syx::{MAX_BANK_LEN, Writer};

/// Deterministic byte source.
///
/// xorshift64*, which is four lines and good enough to produce byte strings
/// that no hand-written case would think of. Nothing here needs a real
/// generator, and a real generator would be a dependency. Seeded from a
/// constant, a run is the same run on every machine.
#[derive(Debug)]
pub struct Rng(u64);

impl Rng {
    /// Seeds the generator. A zero seed is replaced, since xorshift cannot
    /// leave it.
    pub const fn new(seed: u64) -> Self {
        Self(if seed == 0 {
            0x9E37_79B9_7F4A_7C15
        } else {
            seed
        })
    }

    /// Returns the next value in the sequence.
    pub fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Returns a byte, taken from the middle of the word where the bits are
    /// best mixed.
    pub fn byte(&mut self) -> u8 {
        self.next_u64().to_le_bytes()[3]
    }

    /// Returns a value below `bound`, which must not be zero.
    pub fn below(&mut self, bound: usize) -> usize {
        let bound = u64::try_from(bound).unwrap_or(u64::MAX).max(1);
        usize::try_from(self.next_u64() % bound).unwrap_or_default()
    }

    /// Returns `len` bytes.
    pub fn bytes(&mut self, len: usize) -> Vec<u8> {
        (0..len).map(|_| self.byte()).collect()
    }

    /// Returns between zero and `max` bytes.
    pub fn bytes_up_to(&mut self, max: usize) -> Vec<u8> {
        let len = self.below(max + 1);
        self.bytes(len)
    }
}

/// A version 7 program with nothing set but its name.
pub fn named(name: &str) -> Program {
    let mut program = Program::new(ProtocolVersion::V7);
    program.set_name(ProgramName::new(name).expect("a legal name"));
    program
}

/// Writes a `.syx` file the way this library writes one: `fill` pushes what
/// the file holds, and the result is exactly the bytes written.
pub fn write_pack(device: DeviceId, fill: impl FnOnce(&mut Writer<'_>)) -> Vec<u8> {
    let mut bytes = vec![0; MAX_BANK_LEN];
    let mut writer = Writer::new(&mut bytes, device);
    fill(&mut writer);
    let written = writer.finish();
    bytes.truncate(written);
    bytes
}

/// Writes `programs` into `bank`, stored from its first slot up.
pub fn write_bank(device: DeviceId, bank: Bank, programs: &[Program]) -> Vec<u8> {
    write_pack(device, |writer| {
        for (index, program) in programs.iter().enumerate() {
            let number =
                ProgramNumber::new(u8::try_from(index).expect("fewer than 128")).expect("in range");
            writer
                .push_program(bank, number, program)
                .expect("a bank-sized buffer");
        }
    })
}
