//! `SysEx` frames, parsed from arbitrary bytes.
//!
//! The property that matters is not that parsing survives but that anything
//! which parses re-encodes to the bytes it came from. A parser that accepted a
//! frame and reported a payload nobody sent would pass a panic-freedom check
//! and still hand a host the wrong program.

#![no_main]

use libfuzzer_sys::fuzz_target;

use deepmind_midi::sysex::Frame;

fuzz_target!(|data: &[u8]| {
    let Ok(frame) = Frame::parse(data) else {
        return;
    };

    assert_eq!(frame.encoded_len(), data.len(), "length disagrees: {frame:?}");
    assert_eq!(frame.to_vec(), data, "re-encoding changed the frame");

    // Whatever the frame says about itself has to be reachable without
    // panicking, since this is what a host reads.
    let _ = frame.message.command();
    let _ = frame.message.packed();
    let _ = frame.message.version();
});
