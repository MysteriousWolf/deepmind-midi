//! The packed MS-bit codec that carries 8-bit data over 7-bit `SysEx`.
//!
//! Seven raw bytes become eight transmitted bytes: one byte holding the seven
//! high bits, then the seven payload bytes with their high bit cleared.
//!
//! ```text
//! raw:     d0 d1 d2 d3 d4 d5 d6          7 bytes, 8 bits each
//! packed:  M  d0 d1 d2 d3 d4 d5 d6       8 bytes, 7 bits each
//!          ^
//!          bit n of M is bit 7 of raw byte n
//! ```
//!
//! # How long is a packed run
//!
//! A length that is not a multiple of seven leaves a choice: pad the last group
//! out to eight bytes, or send only the bytes it needs. The manual's figures
//! settle it. Five of the six lengths it tabulates (the globals, pattern, bank
//! names, single name and chord memory dumps) are exactly `ceil(raw / 7) * 8`,
//! the padded form, and none matches the short form. So [`pack_into`] pads, and
//! [`packed_len`] describes what it writes.
//!
//! The sixth is the program dump, where the manual prints 278 packed bytes for
//! 242 raw. That is neither rule: padded gives 280 and short gives 277. Nothing
//! here reproduces 278, and [`unpack_into`] accepts all three, since a decoder
//! that insisted on one would reject hardware over a figure that cannot be
//! right as printed. See the specification's open questions.
//!
//! Padding also means a packed run does not state its raw length: 280 packed
//! bytes hold 245, which is a comms protocol version 7 program and also a
//! version 6 program with three bytes to spare. The version byte says which, and
//! [`crate::ids::ProtocolVersion::program_data_len`] turns it into a length.

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::error::{Error, Result};

/// Raw bytes carried by one packed group.
pub const RAW_GROUP: usize = 7;

/// Wire bytes one packed group occupies.
pub const PACKED_GROUP: usize = 8;

/// Returns the number of bytes [`pack_into`] writes for `raw_len` raw bytes.
#[must_use]
pub const fn packed_len(raw_len: usize) -> usize {
    raw_len.div_ceil(RAW_GROUP) * PACKED_GROUP
}

/// Returns the number of raw bytes a packed run of `packed_len` bytes holds.
///
/// A padded run reports the padding as data: the caller knows the real length
/// from the message's comms protocol version and truncates.
///
/// # Errors
///
/// Returns [`Error::PackedRunLength`] when the run ends in a high-bit byte with
/// no data bytes for it to apply to, which no encoder produces.
pub const fn unpacked_len(packed_len: usize) -> Result<usize> {
    let rest = packed_len % PACKED_GROUP;
    if rest == 1 {
        return Err(Error::PackedRunLength(packed_len));
    }
    Ok(packed_len / PACKED_GROUP * RAW_GROUP + rest.saturating_sub(1))
}

/// Packs `raw` into `out`, padding the last group with zeros.
///
/// Returns the number of bytes written, which is [`packed_len`] of `raw.len()`.
///
/// # Errors
///
/// Returns [`Error::BufferTooSmall`] when `out` is shorter than that.
pub fn pack_into(raw: &[u8], out: &mut [u8]) -> Result<usize> {
    let needed = packed_len(raw.len());
    let available = out.len();
    let out = out
        .get_mut(..needed)
        .ok_or(Error::BufferTooSmall { needed, available })?;

    for (group, packed) in raw.chunks(RAW_GROUP).zip(out.chunks_mut(PACKED_GROUP)) {
        let high = group
            .iter()
            .enumerate()
            .fold(0u8, |bits, (index, byte)| bits | ((byte >> 7) << index));
        if let Some(slot) = packed.first_mut() {
            *slot = high;
        }
        let data = group
            .iter()
            .map(|byte| byte & 0x7F)
            .chain(core::iter::repeat(0));
        for (slot, byte) in packed.iter_mut().skip(1).zip(data) {
            *slot = byte;
        }
    }
    Ok(needed)
}

/// Unpacks `packed` into `out`, high bits restored.
///
/// Returns the number of bytes written, which is [`unpacked_len`] of
/// `packed.len()`. Bit 7 of an input byte is ignored: `SysEx` cannot carry it,
/// so a set one means the caller is not holding what it thinks it is, not that
/// the payload means something else.
///
/// # Errors
///
/// Returns [`Error::PackedRunLength`] for a run that cannot be unpacked, and
/// [`Error::BufferTooSmall`] when `out` is shorter than the result.
pub fn unpack_into(packed: &[u8], out: &mut [u8]) -> Result<usize> {
    let needed = unpacked_len(packed.len())?;
    let available = out.len();
    let out = out
        .get_mut(..needed)
        .ok_or(Error::BufferTooSmall { needed, available })?;

    for (group, raw) in packed.chunks(PACKED_GROUP).zip(out.chunks_mut(RAW_GROUP)) {
        let Some((high, data)) = group.split_first() else {
            continue;
        };
        for (index, (slot, byte)) in raw.iter_mut().zip(data).enumerate() {
            *slot = (byte & 0x7F) | (((high >> index) & 1) << 7);
        }
    }
    Ok(needed)
}

/// Packs `raw` into a new vector.
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[must_use]
pub fn pack(raw: &[u8]) -> Vec<u8> {
    let mut out = alloc::vec![0; packed_len(raw.len())];
    let written = pack_into(raw, &mut out).unwrap_or(0);
    out.truncate(written);
    out
}

/// Unpacks `packed` into a new vector.
///
/// # Errors
///
/// Returns [`Error::PackedRunLength`] for a run that cannot be unpacked.
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub fn unpack(packed: &[u8]) -> Result<Vec<u8>> {
    let mut out = alloc::vec![0; unpacked_len(packed.len())?];
    unpack_into(packed, &mut out)?;
    Ok(out)
}

#[cfg(all(test, feature = "alloc"))]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use super::*;

    /// Deterministic bytes with the high bit set in an uneven pattern.
    fn sample(len: usize) -> Vec<u8> {
        let mut state = 0x1234_5678_9ABC_DEF0_u64;
        (0..len)
            .map(|_| {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "the low byte is the sample"
                )]
                {
                    (state >> 33) as u8
                }
            })
            .collect()
    }

    #[test]
    fn packing_round_trips_for_every_length_up_to_four_groups() {
        for len in 0..=28 {
            let raw = sample(len);
            let packed = pack(&raw);
            assert_eq!(packed.len(), packed_len(len));
            assert!(
                packed.iter().all(|byte| byte < &0x80),
                "packed data must fit seven bits"
            );
            let back = unpack(&packed).expect("a packed run unpacks");
            assert_eq!(
                back.get(..len),
                Some(raw.as_slice()),
                "{len} bytes did not survive"
            );
            // Only the padding may follow, and it is zero.
            assert!(
                back.get(len..)
                    .is_some_and(|tail| tail.iter().all(|b| *b == 0))
            );
        }
    }

    #[test]
    fn a_group_carries_the_high_bits_in_its_first_byte() {
        let packed = pack(&[0x80, 0x00, 0xFF, 0x7F]);
        assert_eq!(packed, [0b0000_0101, 0x00, 0x00, 0x7F, 0x7F, 0, 0, 0]);
    }

    #[test]
    fn the_manual_tabulates_the_padded_length_everywhere_but_the_program_dump() {
        // Globals, pattern, bank names, single name, chord memory, poly chord.
        for (raw, packed) in [
            (45, 56),
            (65, 80),
            (2048, 2344),
            (16, 24),
            (26, 32),
            (512, 592),
        ] {
            assert_eq!(packed_len(raw), packed, "{raw} raw bytes");
        }
        // The program dump is the one figure that matches no rule: the manual
        // prints 278 where padding gives 280 and a short last group gives 277.
        assert_eq!(packed_len(242), 280);
        assert_ne!(packed_len(242), 278);
    }

    #[test]
    fn a_short_last_group_unpacks_as_readily_as_a_padded_one() {
        let raw = sample(242);
        let padded = pack(&raw);
        assert_eq!(padded.len(), 280);
        // Drop the three padding bytes, which is the short form of the same run.
        let short = padded.get(..277).expect("280 bytes to cut down").to_vec();
        let back = unpack(&short).expect("a short run unpacks");
        assert_eq!(back.len(), 242);
        assert_eq!(back, raw);
    }

    #[test]
    fn a_run_ending_in_a_lone_high_bit_byte_is_rejected() {
        assert_eq!(unpacked_len(1), Err(Error::PackedRunLength(1)));
        assert_eq!(unpacked_len(9), Err(Error::PackedRunLength(9)));
        assert_eq!(unpack(&[0x00]), Err(Error::PackedRunLength(1)));
        assert_eq!(unpacked_len(0), Ok(0));
        assert_eq!(unpacked_len(8), Ok(7));
        assert_eq!(unpacked_len(2), Ok(1));
    }

    #[test]
    fn a_buffer_that_cannot_hold_the_result_is_reported_rather_than_filled() {
        let mut small = [0u8; 4];
        assert_eq!(
            pack_into(&[1, 2, 3, 4], &mut small),
            Err(Error::BufferTooSmall {
                needed: 8,
                available: 4
            })
        );
        assert_eq!(small, [0; 4]);
        assert_eq!(
            unpack_into(&[0, 1, 2, 3, 4, 5, 6, 7], &mut small),
            Err(Error::BufferTooSmall {
                needed: 7,
                available: 4
            })
        );
    }

    #[test]
    fn bit_seven_of_an_input_byte_is_ignored() {
        let mut out = [0u8; 7];
        unpack_into(&[0xFF, 0xFF, 0, 0, 0, 0, 0, 0], &mut out).expect("buffer fits");
        // Bit 7 of the high-bit byte addresses nothing, and the data byte's own
        // bit 7 is replaced by the one the high byte carries.
        assert_eq!(out, [0xFF, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80]);
    }
}
