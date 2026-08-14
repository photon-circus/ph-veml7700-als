//! Threshold-monitor semantic types.

use crate::config::{MeasurementConfig, Persistence};
use crate::measurement::AlsCounts;
use crate::power::PowerSavingConfig;

/// Raw ALS low/high thresholds.
///
/// Fields are private so that `low <= high` cannot be bypassed by a struct
/// literal. [`Thresholds::new`] is the only way to build this type, and the
/// driver therefore cannot program a reversed pair that
/// [`Veml7700::read_thresholds`](crate::Veml7700::read_thresholds) would reject
/// when read back.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Thresholds {
    low: AlsCounts,
    high: AlsCounts,
}

impl Thresholds {
    /// Construct ordered thresholds, rejecting `low > high`.
    pub const fn new(low: AlsCounts, high: AlsCounts) -> Option<Self> {
        if low.counts() <= high.counts() {
            Some(Self { low, high })
        } else {
            None
        }
    }

    /// Return the low threshold in raw ALS counts.
    pub const fn low(self) -> AlsCounts {
        self.low
    }

    /// Return the high threshold in raw ALS counts.
    pub const fn high(self) -> AlsCounts {
        self.high
    }
}

/// Polled threshold flags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ThresholdStatus {
    /// Low threshold flag, register bit 15.
    pub low: bool,
    /// High threshold flag, register bit 14.
    pub high: bool,
}

impl ThresholdStatus {
    pub(crate) const fn decode(word: u16) -> Result<Self, ThresholdStatusDecodeError> {
        let reserved = word & 0x3FFF;
        if reserved != 0 {
            return Err(ThresholdStatusDecodeError::ReservedBits { observed: reserved });
        }
        Ok(Self {
            low: word & (1 << 15) != 0,
            high: word & (1 << 14) != 0,
        })
    }
}

/// Failure decoding the threshold-status register.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ThresholdStatusDecodeError {
    /// Reserved status bits were observed set.
    ReservedBits {
        /// Reserved bits that were observed set.
        observed: u16,
    },
}

/// Complete monitored domain for the VEML7700's polled threshold feature.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ThresholdMonitorConfig {
    /// Gain and integration time that define threshold count meaning.
    pub measurement: MeasurementConfig,
    /// Raw low/high thresholds in that measurement domain.
    pub thresholds: Thresholds,
    /// Consecutive qualifying measurement count.
    pub persistence: Persistence,
    /// Cadence that determines wall-clock qualification behavior.
    pub power_saving: PowerSavingConfig,
}

impl ThresholdMonitorConfig {
    /// Construct a complete monitored domain.
    pub const fn new(
        measurement: MeasurementConfig,
        thresholds: Thresholds,
        persistence: Persistence,
        power_saving: PowerSavingConfig,
    ) -> Self {
        Self {
            measurement,
            thresholds,
            persistence,
            power_saving,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_reserved_status_bit_is_rejected() {
        for bit in 0_u32..14 {
            let observed = 1_u16 << bit;
            assert_eq!(
                ThresholdStatus::decode(observed),
                Err(ThresholdStatusDecodeError::ReservedBits { observed })
            );
        }
    }

    #[test]
    fn all_documented_status_flag_combinations_decode() {
        for (word, low, high) in [
            (0x0000, false, false),
            (0x4000, false, true),
            (0x8000, true, false),
            (0xC000, true, true),
        ] {
            assert_eq!(
                ThresholdStatus::decode(word),
                Ok(ThresholdStatus { low, high })
            );
        }
    }

    #[test]
    fn thresholds_accept_equal_endpoints_and_reject_reversal() {
        let equal = AlsCounts::from_counts(42);
        let ordered = Thresholds::new(equal, equal).expect("equal endpoints are ordered");
        assert_eq!(ordered.low(), equal);
        assert_eq!(ordered.high(), equal);

        let ascending =
            Thresholds::new(AlsCounts::from_counts(42), AlsCounts::from_counts(43)).unwrap();
        assert_eq!(ascending.low().counts(), 42);
        assert_eq!(ascending.high().counts(), 43);

        assert_eq!(
            Thresholds::new(AlsCounts::from_counts(43), AlsCounts::from_counts(42)),
            None
        );
    }
}
