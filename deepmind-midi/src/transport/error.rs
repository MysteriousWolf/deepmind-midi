//! What a blocking call fails with.

use core::fmt;

use crate::device::Request;

/// Everything a [`Transport`](super::Transport) call can fail with.
///
/// Generic over the port's own error, which travels through unchanged. A host
/// that wants to know whether its USB cable fell out still gets to ask its own
/// backend, rather than reading it out of a string this library built.
///
/// The three variants that are not the port or the protocol are the three ways
/// waiting ends without an answer, and they are separate because the right
/// response to each differs: [`Timeout`](Error::Timeout) means ask again,
/// [`Dropped`](Error::Dropped) means something else in the host cancelled the
/// request, and [`Elapsed`](Error::Elapsed) means the host's own limit ran out
/// while the synthesizer was still within its.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error<E> {
    /// The port failed, and what it failed with.
    Port(E),
    /// The library refused what was asked of it.
    ///
    /// A queue with no room, an edit before any program is known, a value
    /// outside a parameter's range: the same rejections
    /// [`Device`](crate::device::Device) makes, since a blocking call is those
    /// calls with a loop around them.
    Protocol(crate::Error),
    /// The synthesizer did not answer inside
    /// [`Device::with_timeout`](crate::device::Device::with_timeout).
    ///
    /// Nothing is retried. Only the host knows whether asking again is the right
    /// thing to do, and for a bank half-read it usually is not.
    Timeout(Request),
    /// The request stopped being outstanding without ever being answered.
    ///
    /// What a host sees when something else reached the same device in the
    /// meantime: a [`reset`](crate::device::Device::reset) between pumps, or a
    /// second identical request whose answer cleared this one. Not reachable by
    /// a host that drives one transport from one place.
    Dropped(Request),
    /// The limit given to [`wait_for`](super::Transport::wait_for) passed.
    ///
    /// The synthesizer has not been declared late; the device timeout is what
    /// says that. This is only the host's own limit running out.
    Elapsed,
}

impl<E> Error<E> {
    /// Returns the port's error, for the one variant that carries it.
    #[must_use]
    pub fn port(self) -> Option<E> {
        match self {
            Self::Port(error) => Some(error),
            _ => None,
        }
    }

    /// Returns the request that went unanswered, for the variants about one.
    #[must_use]
    pub const fn request(&self) -> Option<Request> {
        match self {
            Self::Timeout(request) | Self::Dropped(request) => Some(*request),
            _ => None,
        }
    }
}

impl<E> From<crate::Error> for Error<E> {
    fn from(error: crate::Error) -> Self {
        Self::Protocol(error)
    }
}

impl<E: fmt::Display> fmt::Display for Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Port(error) => write!(f, "MIDI port: {error}"),
            Self::Protocol(error) => error.fmt(f),
            Self::Timeout(request) => write!(f, "no answer to the {request}"),
            Self::Dropped(request) => write!(f, "the {request} was dropped before it was answered"),
            Self::Elapsed => f.write_str("waited longer than the caller allowed"),
        }
    }
}

#[cfg(feature = "std")]
impl<E: std::error::Error + 'static> std::error::Error for Error<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Port(error) => Some(error),
            Self::Protocol(error) => Some(error),
            _ => None,
        }
    }
}
