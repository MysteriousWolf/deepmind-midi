//! Where the time comes from, and what waiting means.

/// A monotonic millisecond clock, and a way to wait on it.
///
/// [`Device`](crate::device::Device) takes the time as an argument and never
/// asks for it; the blocking adapter is the layer that has to ask. Which is why
/// this is a trait rather than a call into `std`: a host on bare metal has a
/// timer and a `wfi`, not a `SystemTime`, and it should not have to give up the
/// driver loop to use its own.
///
/// # Implementing it
///
/// ```
/// use deepmind_midi::transport::Clock;
///
/// /// A clock that only moves when it is told to, which is what a test wants.
/// struct Fake(u64);
///
/// impl Clock for Fake {
///     fn now_ms(&mut self) -> u64 {
///         self.0
///     }
///
///     fn sleep_ms(&mut self, milliseconds: u64) {
///         self.0 = self.0.saturating_add(milliseconds);
///     }
/// }
/// ```
pub trait Clock {
    /// Returns the milliseconds elapsed since some fixed moment.
    ///
    /// Which moment does not matter and is never reported; only the differences
    /// are read. What does matter is that the answer never goes backwards, since
    /// a reading behind the last one is ignored rather than treated as time
    /// passing in reverse.
    fn now_ms(&mut self) -> u64;

    /// Waits about `milliseconds`, then returns.
    ///
    /// About, not exactly: this bounds how long a quiet loop spends between
    /// reads of the port, and a sleep that overshoots costs latency rather than
    /// correctness. Returning immediately is allowed and turns the driver loop
    /// into a spin, which is the right implementation for a host that has
    /// nothing else to do with the core.
    fn sleep_ms(&mut self, milliseconds: u64);
}

impl<C: Clock + ?Sized> Clock for &mut C {
    fn now_ms(&mut self) -> u64 {
        (**self).now_ms()
    }

    fn sleep_ms(&mut self, milliseconds: u64) {
        (**self).sleep_ms(milliseconds);
    }
}

/// The clock `std` already has: [`Instant`](std::time::Instant) and
/// [`thread::sleep`](std::thread::sleep).
///
/// Monotonic from the moment it was built, which is what
/// [`Instant`](std::time::Instant) guarantees and what
/// [`SystemTime`](std::time::SystemTime) does not.
///
/// ```
/// use deepmind_midi::transport::{Clock, StdClock};
///
/// let mut clock = StdClock::new();
/// let started = clock.now_ms();
/// clock.sleep_ms(2);
/// assert!(clock.now_ms() >= started + 2);
/// ```
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[derive(Debug, Clone, Copy)]
pub struct StdClock {
    start: std::time::Instant,
}

#[cfg(feature = "std")]
impl StdClock {
    /// Starts a clock, counting from now.
    #[must_use]
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }
}

#[cfg(feature = "std")]
impl Default for StdClock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "std")]
impl Clock for StdClock {
    fn now_ms(&mut self) -> u64 {
        // Saturating rather than wrapping: a process that has run for 584
        // million years has a bigger problem than a late timeout.
        u64::try_from(self.start.elapsed().as_millis()).unwrap_or(u64::MAX)
    }

    fn sleep_ms(&mut self, milliseconds: u64) {
        std::thread::sleep(core::time::Duration::from_millis(milliseconds));
    }
}
