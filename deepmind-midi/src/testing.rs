//! Helpers the crate's own tests use, written to work with no features on.
//!
//! `cargo test --no-default-features` builds these tests, and there is no
//! `String` to format into there. [`assert_display`] is what stands in for
//! comparing against `to_string()`.

use core::fmt::{self, Display, Write};

/// A fixed-size sink for one [`Display`] implementation's output.
struct Buffer {
    bytes: [u8; Self::LEN],
    used: usize,
}

impl Buffer {
    /// Longer than anything this crate displays.
    const LEN: usize = 64;

    const fn new() -> Self {
        Self {
            bytes: [0; Self::LEN],
            used: 0,
        }
    }

    fn as_str(&self) -> Option<&str> {
        core::str::from_utf8(self.bytes.get(..self.used)?).ok()
    }
}

impl Write for Buffer {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let end = self.used.checked_add(text.len()).ok_or(fmt::Error)?;
        let slot = self.bytes.get_mut(self.used..end).ok_or(fmt::Error)?;
        slot.copy_from_slice(text.as_bytes());
        self.used = end;
        Ok(())
    }
}

/// Asserts that `value` displays as `expected`.
#[track_caller]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
pub fn assert_display<T: Display>(value: T, expected: &str) {
    let mut buffer = Buffer::new();
    write!(buffer, "{value}").expect("nothing this crate displays is 64 bytes long");
    assert_eq!(buffer.as_str(), Some(expected));
}
