//! Measurement result types.

use crate::config::{ConfigurationSnapshot, MeasurementConfig};
use crate::id::DeviceId;
use crate::illuminance::{MicroLux, NominalScale};
use crate::power::PowerSavingSnapshot;
use crate::threshold::{ThresholdStatus, Thresholds};

/// Raw ambient-light-channel counts (`S-32`).
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

    /// Return whether the observed ALS word is `0xFFFF`.
    ///
    /// This is an exact register observation. It does not itself prove physical
    /// clipping, scene overrange, or an illuminance lower bound (`S-51`, `S-52`).
    pub const fn is_max_code(self) -> bool {
        self.0 == u16::MAX
    }

    /// Convert with the nominal scale recorded by `S-26`.
    pub const fn nominal_micro_lux(self, config: MeasurementConfig) -> MicroLux {
        NominalScale::for_config(config).scale_counts(self.0)
    }
}

/// Raw white-channel counts (`S-33`).
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
    /// The driver entered shutdown after its requested conversion wait.
    FrozenAfterRequestedWait,
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

/// Measurement captured after a controlled configuration and requested wait.
///
/// This records the operation the driver performed, not proof that the returned
/// registers contain a new conversion. The default wait is a conditional timing
/// policy; [`requested_wait_us`](Self::requested_wait_us) records what was asked
/// of the delay provider.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MeasurementCapture {
    /// Ambient-light counts.
    pub als: AlsCounts,
    /// White-channel counts.
    pub white: WhiteCounts,
    /// Measurement configuration used for the conversion.
    pub configuration: MeasurementConfig,
    /// Nominal illuminance computed from ALS counts.
    ///
    /// At maximum code this driver conservatively treats the nominal value as an
    /// unreliable point estimate. The observable fact is only `als == 0xFFFF`;
    /// it does not prove clipping, overrange, or a lower bound on scene
    /// illuminance (`S-51`).
    ///
    /// Check [`AlsCounts::is_max_code`] before using this value. The maximum-code
    /// observation is not reported as an error.
    ///
    /// Nominal throughout: the `S-26` scale applied to counts, never calibrated
    /// system lux, and without the correction described by `S-29` and `S-30`.
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
                AlsCounts::from_counts(counts).is_max_code(),
                counts == u16::MAX
            );
        }
    }
}
