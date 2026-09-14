//! The MIDI decoder, fed whatever is on the cable.
//!
//! Two properties, because panic-freedom alone would be satisfied by a decoder
//! that dropped everything. The other one is that chunking is irrelevant: the
//! same bytes split at different places have to produce the same events, which
//! is the invariant that holds a reassembler together and the one a boundary
//! bug breaks.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

use deepmind_midi::wire::{Decoder, Event};

/// A stream and the chunking to feed it in.
#[derive(Arbitrary, Debug)]
struct Input {
    /// Bytes as they would arrive from a port.
    stream: Vec<u8>,
    /// Sizes of the reads that carry them, cycled. Empty means one read.
    chunks: Vec<u8>,
}

/// An event with nothing borrowed, so two runs can be compared.
#[derive(Debug, PartialEq, Eq)]
enum Owned {
    Event(String),
    SysEx(Vec<u8>),
    Error(String),
}

fn owned(event: deepmind_midi::Result<Event<'_>>) -> Owned {
    match event {
        Ok(Event::SysEx(frame)) => Owned::SysEx(frame.to_vec()),
        Ok(other) => Owned::Event(format!("{other:?}")),
        Err(error) => Owned::Error(error.to_string()),
    }
}

fuzz_target!(|input: Input| {
    let mut whole: Decoder = Decoder::new();
    let mut expected = Vec::new();
    whole.feed(&input.stream, |event| expected.push(owned(event)));

    let sizes: Vec<usize> = input
        .chunks
        .iter()
        .map(|size| usize::from(*size).max(1))
        .collect();

    let mut split: Decoder = Decoder::new();
    let mut found = Vec::new();
    let mut rest = input.stream.as_slice();
    let mut sizes = sizes.iter().copied().cycle();
    while !rest.is_empty() {
        let take = sizes.next().unwrap_or(rest.len()).min(rest.len());
        let (head, tail) = rest.split_at(take);
        split.feed(head, |event| found.push(owned(event)));
        rest = tail;
    }

    assert_eq!(found, expected, "chunking changed what was decoded");

    // A reset decoder is a new decoder, whatever it was in the middle of.
    split.reset();
    assert!(!split.in_sysex());
});
