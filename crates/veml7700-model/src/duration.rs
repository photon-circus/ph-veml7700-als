//! Unit-bearing relative duration consumed by the model.
//!
//! The model does not store a harness `now` or implement `DelayNs`. Callers
//! supply non-negative elapsed duration explicitly.

/// Non-negative elapsed duration with nanosecond resolution.
///
/// Nanoseconds are retained so valid partitions of the same elapsed interval
/// remain observationally equivalent, including `DelayNs` requests that arrive
/// as nanoseconds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct RelativeDuration {
    nanos: u64,
}

impl RelativeDuration {
    /// A zero-length step. The model remains unchanged.
    pub const ZERO: Self = Self { nanos: 0 };

    /// Construct from whole nanoseconds.
    #[must_use]
    pub const fn from_nanos(nanos: u64) -> Self {
        Self { nanos }
    }

    /// Construct from whole microseconds.
    #[must_use]
    pub const fn from_micros(micros: u64) -> Self {
        Self {
            nanos: micros.saturating_mul(1_000),
        }
    }

    /// Return the duration in nanoseconds.
    #[must_use]
    pub const fn as_nanos(self) -> u64 {
        self.nanos
    }
}
