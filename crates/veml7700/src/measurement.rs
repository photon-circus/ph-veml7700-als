//! Measurement result types.

use crate::config::{ConfigurationSnapshot, MeasurementConfig};
use crate::id::DeviceId;
use crate::illuminance::{MicroLux, NominalScale};
use crate::power::PowerSavingSnapshot;
use crate::threshold::{ThresholdStatus, Thresholds};

/// Raw ambient-light-channel counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AlsCounts(u16);

impl AlsCounts {
    /// Construct from raw ADC counts.
    pub const fn from_counts(counts: u16) -> Self {
        Self(counts)
    }

    /// Return raw ADC counts.
    pub const fn counts(self) -> u16 {
        self.0
    }

    /// Return whether the ADC word is at its maximum code.
    pub const fn is_saturated(self) -> bool {
        self.0 == u16::MAX
    }

    /// Convert with the nominal vendor-table scale.
    pub const fn nominal_micro_lux(self, config: MeasurementConfig) -> MicroLux {
        NominalScale::for_config(config).scale_counts(self.0)
    }
}

/// Raw white-channel counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct WhiteCounts(u16);

impl WhiteCounts {
    /// Construct from raw counts.
    pub const fn from_counts(counts: u16) -> Self {
        Self(counts)
    }

    /// Return raw counts.
    pub const fn counts(self) -> u16 {
        self.0
    }
}

/// Coherence qualification for an ALS/white pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MeasurementPairCoherence {
    /// Registers were read sequentially and may straddle an autonomous refresh.
    SequentialRegisters,
    /// The driver first entered shutdown to freeze the most recently completed data.
    FrozenAfterFreshWait,
}

/// Diagnostic register snapshot with no freshness guarantee.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SnapshotMeasurement {
    /// Ambient-light channel counts.
    pub als: AlsCounts,
    /// White-channel counts.
    pub white: WhiteCounts,
    /// Configuration observed before reading data.
    pub configuration: ConfigurationSnapshot,
    /// Power-saving state observed before reading data.
    pub power_saving: PowerSavingSnapshot,
    /// Pair-coherence qualification.
    pub coherence: MeasurementPairCoherence,
}

/// Fresh measurement deliberately configured and observed after a bounded wait.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FreshMeasurement {
    /// Ambient-light counts.
    pub als: AlsCounts,
    /// White-channel counts.
    pub white: WhiteCounts,
    /// Measurement configuration used for the conversion.
    pub configuration: MeasurementConfig,
    /// Nominal illuminance computed from ALS counts.
    ///
    /// **Invalid as any kind of estimate when [`als`](Self::als) is saturated.**
    /// At maximum code the conversion clipped, so this is the domain's ceiling
    /// rather than an observation. It does not bound the actual illuminance
    /// either: the figure is nominal, so it says nothing about the scene outside
    /// its own scale. A clipped reading establishes only that the configuration
    /// was too narrow.
    ///
    /// Check [`AlsCounts::is_saturated`] before using this at all; saturation is
    /// not reported as an error, so an unchecked read looks like an ordinary
    /// value.
    ///
    /// Nominal throughout: the vendor scale factor applied to counts, never
    /// calibrated system lux, and uncorrected for the non-linearity the sources
    /// describe above roughly 1 000 lx.
    pub nominal_illuminance: MicroLux,
    /// Delay this driver **requested** before freezing the result.
    ///
    /// Not measured elapsed time. `embedded_hal_async::delay::DelayNs`
    /// guarantees *at least* the requested duration and may take longer —
    /// arbitrarily so under a loaded executor — and this driver reads no clock,
    /// so it cannot know what actually passed. Treat this as the conservative
    /// lower bound the conversion was given, not as evidence of how long it ran.
    pub requested_wait_us: u32,
    /// Pair-coherence qualification.
    pub coherence: MeasurementPairCoherence,
}

/// Read-only diagnostic snapshot that does not claim fresh optical data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DeviceSnapshot {
    /// Observed identity register.
    pub id: DeviceId,
    /// Observed configuration register.
    pub configuration: ConfigurationSnapshot,
    /// Observed power-saving register.
    pub power_saving: PowerSavingSnapshot,
    /// Observed low/high threshold registers.
    pub thresholds: Thresholds,
    /// Observed polled threshold flags.
    pub threshold_status: ThresholdStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_count_types_preserve_the_entire_word_domain() {
        for counts in [0, 1, u16::MAX - 1, u16::MAX] {
            assert_eq!(AlsCounts::from_counts(counts).counts(), counts);
            assert_eq!(WhiteCounts::from_counts(counts).counts(), counts);
            assert_eq!(
                AlsCounts::from_counts(counts).is_saturated(),
                counts == u16::MAX
            );
        }
    }
}
