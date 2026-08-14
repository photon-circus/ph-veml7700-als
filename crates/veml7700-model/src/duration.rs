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
    /// No elapsed time.
    pub const ZERO: Self = Self { nanos: 0 };

    /// Construct from whole nanoseconds.
    #[must_use]
    pub const fn from_nanos(nanos: u64) -> Self {
        Self { nanos }
    }

    /// Construct from whole microseconds.
    ///
    /// # Panics
    ///
    /// If the value does not fit in nanoseconds.
    ///
    /// This used to saturate. Saturation is the wrong failure here: it silently
    /// substitutes a different duration for the one the harness asked for, and
    /// every later assertion is then made against a timeline nobody chose. A
    /// test that overflows this is a defect in the test, and it should say so
    /// where it happens rather than pass against ~584 years of virtual time.
    ///
    /// Use [`try_from_micros`](Self::try_from_micros) where the input is not a
    /// literal.
    #[must_use]
    pub const fn from_micros(micros: u64) -> Self {
        match Self::try_from_micros(micros) {
            Some(duration) => duration,
            None => panic!("microsecond duration overflows nanosecond resolution"),
        }
    }

    /// Construct from whole microseconds, or `None` on overflow.
    #[must_use]
    pub const fn try_from_micros(micros: u64) -> Option<Self> {
        match micros.checked_mul(1_000) {
            Some(nanos) => Some(Self { nanos }),
            None => None,
        }
    }

    /// Return the duration in nanoseconds.
    #[must_use]
    pub const fn as_nanos(self) -> u64 {
        self.nanos
    }
}
