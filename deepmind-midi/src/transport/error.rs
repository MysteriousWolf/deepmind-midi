//! What a blocking call fails with.

use core::fmt;

use crate::device::Request;

/// Everything a [`Transport`](super::Transport) call can fail with.
///
/// Generic over the port's own error, which travels through unchanged, so a host
/// asks its own backend what went wrong rather than parsing a string from here.
///
/// Waiting ends without an answer two ways: [`Timeout`](Error::Timeout) is the
/// synthesizer's silence, and [`Elapsed`](Error::Elapsed) is the caller's own
/// limit running out first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    /// The limit given to [`wait_for`](super::Transport::wait_for) passed.
    ///
    /// The synthesizer has not been declared late; the device timeout is what
    /// says that. This is only the host's own limit running out.
    Elapsed,
}

impl<E> Error<E> {
    /// Returns the port's error, for the one variant that carries it.
    #[must_use]
    pub const fn port(&self) -> Option<&E> {
        match self {
            Self::Port(error) => Some(error),
            _ => None,
        }
    }

    /// Takes the port's error, for the one variant that carries it.
    #[must_use]
    pub fn into_port(self) -> Option<E> {
        match self {
            Self::Port(error) => Some(error),
            _ => None,
        }
    }

    /// Returns the library's rejection, for the one variant that carries it.
    #[must_use]
    pub const fn protocol(&self) -> Option<crate::Error> {
        match self {
            Self::Protocol(error) => Some(*error),
            _ => None,
        }
    }

    /// Returns the request that went unanswered, for the one variant about one.
    #[must_use]
    pub const fn request(&self) -> Option<Request> {
        match self {
            Self::Timeout(request) => Some(*request),
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
            Self::Elapsed => f.write_str("waited longer than the caller allowed"),
        }
    }
}

impl<E: core::error::Error + 'static> core::error::Error for Error<E> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Port(error) => Some(error),
            Self::Protocol(error) => Some(error),
            _ => None,
        }
    }
}
