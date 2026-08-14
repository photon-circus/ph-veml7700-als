//! Public error taxonomy.

use crate::config::{ConfigDecodeError, IntegrationTime};
use crate::measurement::FreshMeasurement;
use crate::power::PowerSavingDecodeError;
use crate::threshold::ThresholdStatusDecodeError;

/// High-level operation associated with a bus failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Operation {
    /// Read-only inspection.
    Inspect,
    /// Snapshot measurement.
    Snapshot,
    /// Fresh one-shot-style measurement sequence.
    MeasureOnce,
    /// Ordinary configuration change.
    Configure,
    /// Threshold-monitor configuration.
    ThresholdMonitor,
}

/// Exact register-level bus context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BusContext {
    /// Read configuration register.
    ReadConfiguration,
    /// Write configuration register.
    WriteConfiguration,
    /// Read power-saving register.
    ReadPowerSaving,
    /// Write power-saving register.
    WritePowerSaving,
    /// Read ambient-light data.
    ReadAls,
    /// Read white-channel data.
    ReadWhite,
    /// Read device ID.
    ReadDeviceId,
    /// Read threshold status.
    ReadThresholdStatus,
    /// Read low threshold.
    ReadLowThreshold,
    /// Read high threshold.
    ReadHighThreshold,
    /// Write low threshold.
    WriteLowThreshold,
    /// Write high threshold.
    WriteHighThreshold,
}

/// Configuration failure independent of the transport.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ConfigurationError {
    /// Configuration register contained an undocumented encoding.
    ConfigurationDecode(ConfigDecodeError),
    /// Power-saving register contained reserved bits.
    PowerSavingDecode(PowerSavingDecodeError),
    /// Threshold-status register contained reserved bits.
    ThresholdStatusDecode(ThresholdStatusDecodeError),
    /// Requested thresholds were reversed.
    ReversedThresholds,
    /// An enabled threshold monitor would be silently retargeted.
    ThresholdMonitorOwnsDomain,
    /// Explicit timing was derived for a different integration-time setting.
    TimingIntegrationMismatch {
        /// Integration time requested for the measurement.
        measurement: IntegrationTime,
        /// Integration time used to derive the supplied timing.
        timing: IntegrationTime,
    },
}

/// Ordinary driver failure preserving the concrete I²C error.
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<E> {
    /// I²C transaction failed.
    Bus {
        /// Semantic operation.
        operation: Operation,
        /// Register-level context.
        context: BusContext,
        /// Underlying HAL error.
        source: E,
    },
    /// Device state or requested configuration was invalid.
    Configuration(ConfigurationError),
}

/// Probe-specific failure.
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ProbeError<E> {
    /// Fixed address did not acknowledge.
    NotPresent,
    /// A non-address-NACK bus failure occurred.
    Bus(E),
    /// ID register did not match the supported VEML7700 address option.
    WrongDevice {
        /// Raw unexpected ID register value.
        observed: u16,
    },
}

/// Stage of a complete fresh measurement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MeasureStage {
    /// Validate that explicit timing belongs to the requested integration time.
    ValidateTiming,
    /// Observe original configuration.
    ObserveConfiguration,
    /// Observe original power-saving state.
    ObservePowerSaving,
    /// Disable autonomous power-saving cadence.
    DisablePowerSaving,
    /// Install the requested gain/integration fields while shut down.
    PrepareMeasurement,
    /// Leave shutdown to create a known wake edge and start conversion.
    ActivateMeasurement,
    /// Freeze the completed result by entering shutdown.
    FreezeResult,
    /// Read ALS data.
    ReadAls,
    /// Read white data.
    ReadWhite,
    /// Restore original configuration.
    RestoreConfiguration,
    /// Restore original power-saving register.
    RestorePowerSaving,
}

/// Complete fresh-measurement failure.
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MeasureOnceError<E> {
    /// Failure before a fresh pair was captured.
    Operation {
        /// Failing stage.
        stage: MeasureStage,
        /// Underlying driver failure.
        source: Error<E>,
    },
    /// The primary operation failed and restoration also failed; hardware state is uncertain.
    RecoveryFailed {
        /// Original failing stage.
        failed_stage: MeasureStage,
        /// Original failure.
        source: Error<E>,
        /// Restoration stage that also failed.
        recovery_stage: MeasureStage,
        /// Restoration failure.
        recovery_source: Error<E>,
    },
    /// A fresh pair was captured, but restoration failed and hardware state is uncertain.
    RestoreFailed {
        /// Captured sample remains useful with explicit qualification.
        sample: FreshMeasurement,
        /// Failing restoration stage.
        stage: MeasureStage,
        /// Underlying driver failure.
        source: Error<E>,
    },
}

/// Stage of threshold-monitor programming.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ThresholdMonitorStage {
    /// Observe current configuration.
    ObserveConfiguration,
    /// Disable the threshold monitor before changing its domain.
    DisableMonitor,
    /// Write low threshold.
    WriteLowThreshold,
    /// Write high threshold.
    WriteHighThreshold,
    /// Install power-saving cadence.
    ApplyPowerSaving,
    /// Enable the final monitored domain.
    EnableMonitor,
}

/// Threshold-monitor programming failure.
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ThresholdMonitorError<E> {
    /// Stage that failed.
    pub stage: ThresholdMonitorStage,
    /// Underlying driver failure.
    pub source: Error<E>,
}
