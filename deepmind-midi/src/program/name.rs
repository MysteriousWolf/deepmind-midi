//! The name of a program: sixteen characters, as the synthesizer displays them.

use core::fmt;
use core::str::FromStr;

use crate::error::{Error, Result};

/// A program's name.
///
/// Sixteen characters at most, printable ASCII, and no terminator: the
/// terminator belongs to the field a name is stored in, not to the name.
///
/// ```
/// use deepmind_midi::program::ProgramName;
///
/// let name = ProgramName::new("Bass Sweep")?;
/// assert_eq!(name.as_str(), "Bass Sweep");
/// assert_eq!(name.to_string(), "Bass Sweep");
///
/// assert!(ProgramName::new("Seventeen Charact").is_err());
/// assert!(ProgramName::new("Caffè").is_err());
/// # Ok::<(), deepmind_midi::Error>(())
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProgramName {
    bytes: [u8; ProgramName::MAX_CHARS],
    len: u8,
}

impl ProgramName {
    /// Characters a name holds at most.
    pub const MAX_CHARS: usize = 16;

    /// The empty name, which is what an unnamed program carries.
    pub const EMPTY: Self = Self {
        bytes: [0; Self::MAX_CHARS],
        len: 0,
    };

    /// Builds a name from a string.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ProgramNameTooLong`] past [`ProgramName::MAX_CHARS`] and
    /// [`Error::ProgramNameCharacter`] for a character the synthesizer's display
    /// has no glyph for, which is anything but printable ASCII.
    pub fn new(text: &str) -> Result<Self> {
        if text.chars().count() > Self::MAX_CHARS {
            return Err(Error::ProgramNameTooLong(text.chars().count()));
        }
        let mut name = Self::EMPTY;
        for (slot, character) in name.bytes.iter_mut().zip(text.chars()) {
            if !character.is_ascii() || character.is_ascii_control() {
                return Err(Error::ProgramNameCharacter(character));
            }
            *slot = u8::try_from(character).unwrap_or(b' ');
            name.len = name.len.saturating_add(1);
        }
        Ok(name)
    }

    /// Reads a name out of the field that stores it, stopping at the
    /// terminator.
    ///
    /// Lenient, because this is what arrives from a port or a file: a field with
    /// no terminator is a full sixteen characters, and a byte the display has no
    /// glyph for becomes a question mark. Nothing the synthesizer sends reaches
    /// either case, since the name parameters stop at 127.
    pub(super) fn from_field(field: &[u8]) -> Self {
        let mut name = Self::EMPTY;
        for (slot, byte) in name.bytes.iter_mut().zip(field) {
            if *byte == 0 {
                break;
            }
            *slot = if byte.is_ascii() && !byte.is_ascii_control() {
                *byte
            } else {
                b'?'
            };
            name.len = name.len.saturating_add(1);
        }
        name
    }

    /// Returns the name as a string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        // Every stored byte is printable ASCII, which is valid UTF-8.
        core::str::from_utf8(self.as_bytes()).unwrap_or_default()
    }

    /// Returns the characters the name holds, without a terminator.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.get(..usize::from(self.len)).unwrap_or_default()
    }

    /// Returns the number of characters in the name.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len as usize
    }

    /// Returns whether the name is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl fmt::Display for ProgramName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for ProgramName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl AsRef<str> for ProgramName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl TryFrom<&str> for ProgramName {
    type Error = Error;

    fn try_from(text: &str) -> Result<Self> {
        Self::new(text)
    }
}

impl FromStr for ProgramName {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self> {
        Self::new(text)
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use super::*;

    #[test]
    fn a_name_is_sixteen_printable_characters_at_most() {
        assert_eq!(
            ProgramName::new("Sixteen Charactr")
                .expect("sixteen characters")
                .len(),
            16
        );
        assert_eq!(
            ProgramName::new("Seventeen Charact"),
            Err(Error::ProgramNameTooLong(17))
        );
        assert_eq!(
            ProgramName::new("Tab\there"),
            Err(Error::ProgramNameCharacter('\t'))
        );
        assert_eq!(
            ProgramName::new("Caffè"),
            Err(Error::ProgramNameCharacter('è'))
        );
    }

    #[test]
    fn the_empty_name_is_the_default() {
        let empty = ProgramName::default();
        assert_eq!(empty, ProgramName::EMPTY);
        assert!(empty.is_empty());
        assert_eq!(empty.as_str(), "");
        assert_eq!(ProgramName::new("").expect("the empty name"), empty);
    }

    #[test]
    fn a_field_ends_at_its_terminator() {
        assert_eq!(ProgramName::from_field(b"Bass\0rubbish").as_str(), "Bass");
        assert_eq!(ProgramName::from_field(b"").as_str(), "");
        assert_eq!(ProgramName::from_field(b"\0").as_str(), "");
    }

    /// Not something hardware sends - the name parameters stop at 127 - but a
    /// file is not hardware.
    #[test]
    fn a_byte_with_no_glyph_becomes_a_question_mark() {
        assert_eq!(ProgramName::from_field(&[b'A', 0xFF, b'B']).as_str(), "A?B");
        assert_eq!(ProgramName::from_field(&[b'A', 0x07]).as_str(), "A?");
    }

    #[test]
    fn a_field_with_no_terminator_is_a_full_name() {
        let field = b"SixteenCharacterAndMore";
        assert_eq!(ProgramName::from_field(field).as_str(), "SixteenCharacter");
    }

    #[test]
    fn a_name_parses_from_a_string() {
        assert_eq!(
            "Bass"
                .parse::<ProgramName>()
                .expect("a valid name")
                .as_str(),
            "Bass"
        );
        assert_eq!(
            ProgramName::try_from("Bass")
                .expect("a valid name")
                .as_str(),
            "Bass"
        );
    }
}
