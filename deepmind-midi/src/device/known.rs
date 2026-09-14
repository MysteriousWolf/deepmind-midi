//! Where a tracked value came from, carried with the value.

/// A value the library holds about the synthesizer, and how it came to hold it.
///
/// The synthesizer answers no per-parameter reads. A dump can be asked for and a
/// value can be sent and presumed to have landed, and those are not the same
/// claim. A plain `Option` would conflate them, so every tracked value records
/// which of the two it is and when.
///
/// ```
/// use deepmind_midi::device::Known;
///
/// let sent: Known<u8> = Known::Assumed { value: 64, at: 1_000 };
/// assert_eq!(sent.value(), Some(&64));
/// assert!(!sent.is_confirmed());
/// ```
///
/// [`Assumed`](Known::Assumed) is not a weaker [`Confirmed`](Known::Confirmed):
/// it says the host asked for something and nothing has contradicted it. After
/// sending edits, an edit buffer dump is the only way to turn one into the
/// other, and the library never asks for one on its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Known<T> {
    /// Nothing has been heard, and nothing has been claimed.
    #[default]
    Unknown,
    /// Sent to the synthesizer and not contradicted since.
    Assumed {
        /// The value as the host believes it to be.
        value: T,
        /// Host clock when it went out, in milliseconds.
        at: u64,
    },
    /// Read back from the synthesizer, which is the only thing that confirms it.
    Confirmed {
        /// The value as the synthesizer reported it.
        value: T,
        /// Host clock when it arrived, in milliseconds.
        at: u64,
    },
}

impl<T> Known<T> {
    /// Returns the value, however it came to be held.
    ///
    /// A host that cares which it is asks [`Known::is_confirmed`]; one that only
    /// wants the best answer available asks this.
    #[must_use]
    pub const fn value(&self) -> Option<&T> {
        match self {
            Self::Unknown => None,
            Self::Assumed { value, .. } | Self::Confirmed { value, .. } => Some(value),
        }
    }

    /// Returns the value, consuming the claim.
    #[must_use]
    pub fn into_value(self) -> Option<T> {
        match self {
            Self::Unknown => None,
            Self::Assumed { value, .. } | Self::Confirmed { value, .. } => Some(value),
        }
    }

    /// Returns the host clock reading the claim carries.
    #[must_use]
    pub const fn timestamp(&self) -> Option<u64> {
        match *self {
            Self::Unknown => None,
            Self::Assumed { at, .. } | Self::Confirmed { at, .. } => Some(at),
        }
    }

    /// Returns whether nothing is held.
    #[must_use]
    pub const fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }

    /// Returns whether the value was sent rather than read back.
    #[must_use]
    pub const fn is_assumed(&self) -> bool {
        matches!(self, Self::Assumed { .. })
    }

    /// Returns whether the synthesizer itself reported the value.
    #[must_use]
    pub const fn is_confirmed(&self) -> bool {
        matches!(self, Self::Confirmed { .. })
    }

    /// Returns the value for changing in place, keeping the claim around it.
    ///
    /// The claim is unchanged, which is right for a correction the host makes
    /// to its own copy and wrong for anything the synthesizer said.
    #[must_use]
    pub const fn value_mut(&mut self) -> Option<&mut T> {
        match self {
            Self::Unknown => None,
            Self::Assumed { value, .. } | Self::Confirmed { value, .. } => Some(value),
        }
    }

    /// Borrows the value while keeping the claim around it.
    #[must_use]
    pub const fn as_ref(&self) -> Known<&T> {
        match self {
            Self::Unknown => Known::Unknown,
            Self::Assumed { value, at } => Known::Assumed { value, at: *at },
            Self::Confirmed { value, at } => Known::Confirmed { value, at: *at },
        }
    }

    /// Applies `f` to the value, keeping the claim and its timestamp.
    #[must_use]
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Known<U> {
        match self {
            Self::Unknown => Known::Unknown,
            Self::Assumed { value, at } => Known::Assumed {
                value: f(value),
                at,
            },
            Self::Confirmed { value, at } => Known::Confirmed {
                value: f(value),
                at,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Known;

    #[test]
    fn unknown_holds_nothing() {
        let known: Known<u8> = Known::default();
        assert!(known.is_unknown());
        assert_eq!(known.value(), None);
        assert_eq!(known.timestamp(), None);
    }

    #[test]
    fn a_claim_carries_its_clock_reading() {
        let assumed = Known::Assumed { value: 7u8, at: 12 };
        let confirmed = Known::Confirmed { value: 7u8, at: 34 };

        assert_eq!(assumed.timestamp(), Some(12));
        assert_eq!(confirmed.timestamp(), Some(34));
        assert!(assumed.is_assumed());
        assert!(confirmed.is_confirmed());
        assert_eq!(assumed.value(), confirmed.value());
    }

    #[test]
    fn mapping_keeps_the_claim() {
        let mapped = Known::Confirmed { value: 2u8, at: 5 }.map(u16::from);
        assert_eq!(mapped, Known::Confirmed { value: 2u16, at: 5 });
        assert_eq!(Known::<u8>::Unknown.map(u16::from), Known::Unknown);
    }

    #[test]
    fn borrowing_keeps_the_claim() {
        let mut held = Known::Assumed {
            value: [1u8, 2, 3],
            at: 9,
        };
        assert_eq!(held.as_ref().value(), Some(&&[1, 2, 3]));
        assert_eq!(held.as_ref().timestamp(), Some(9));
        if let Some(value) = held.value_mut() {
            value[0] = 4;
        }
        assert_eq!(
            held,
            Known::Assumed {
                value: [4, 2, 3],
                at: 9
            }
        );
    }
}
