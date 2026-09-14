//! `.syx` files, read from arbitrary bytes.
//!
//! A preset pack is a file off the internet, so the reader is untrusted input
//! in the same sense a port is. Both walks have to terminate, and they
//! terminate by consuming the file: neither can yield more items than the file
//! has `F0` bytes to start frames with.

#![no_main]

use libfuzzer_sys::fuzz_target;

use deepmind_midi::syx::File;

fuzz_target!(|data: &[u8]| {
    let file = File::new(data);
    let limit = data.iter().filter(|byte| **byte == 0xF0).count();

    let mut frames = file.frames();
    let mut seen = 0;
    let mut last = 0;
    while let Some(frame) = frames.next() {
        seen += 1;
        assert!(seen <= limit, "the frame walk is not consuming the file");
        assert!(frames.offset() >= last, "frames came back out of order");
        last = frames.offset();
        let _ = frame;
    }

    let mut seen = 0;
    for entry in file.programs() {
        seen += 1;
        assert!(seen <= limit, "the program walk is not consuming the file");
        if let Ok(entry) = entry {
            // A program that came out of a file has to be a program: its bytes
            // are its declared length, and they pack back to what was read.
            let expected = entry.program.version().program_data_len();
            assert_eq!(expected, entry.program.as_bytes().len());
        }
    }
});
