//! Public error taxonomy.

use crate::config::{ConfigDecodeError, IntegrationTime};
use crate::measurement::FreshMeasurement;
use crate::power::PowerSavingDecodeError;
use crate::threshold::ThresholdStatusDecodeError;

/// High-level operation associated with a bus failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
pub enum MeasureStage {
    /// Validate that explicit timing belongs to the requested integration time.
    ValidateTiming,
    /// Observe original configuration.
    ObserveConfiguration,
    /// Observe original power-saving state.
    ObservePowerSaving,
    /// Enter shutdown in the original domain before reconfiguring.
    ///
    /// Only reached when the operation started from an active device. The
    /// sources require shutdown before any reconfiguration, so this write
    /// changes nothing but the shutdown bit.
    EnterShutdown,
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
#[non_exhaustive]
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
#[non_exhaustive]
pub enum ThresholdMonitorStage {
    /// Observe current configuration.
    ObserveConfiguration,
    /// Enter shutdown with the monitored domain intact.
    ///
    /// Only reached when re-arming an enabled monitor on an active device. The
    /// shutdown and monitor bits cannot move in one write there, so shutdown
    /// goes first and the monitor is disabled while shut down.
    EnterShutdown,
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
///
/// Programming is a sequence of writes, so a failure part way through leaves a
/// device that is neither in its old domain nor the requested one. The two
/// stage fields separate what a caller may rely on from what nobody can know.
///
/// # What each field establishes
///
/// - [`confirmed`](Self::confirmed) is the last stage that returned success.
///   Everything up to and including it **did** reach the device. `None` means no
///   write was confirmed.
/// - [`stage`](Self::stage) is the write that failed. **Its commit status is
///   unknown.** An I²C error can mean the byte never arrived, or that it arrived
///   and the acknowledgement was lost. The transport cannot tell them apart, and
///   this type does not pretend otherwise.
/// - Every stage after [`stage`](Self::stage) was not attempted.
///
/// So the device is in one of exactly two states: the one implied by
/// `confirmed`, or that state plus the effect of `stage`. There is no third
/// possibility, and no rollback was attempted — the driver does not claim a
/// physical commit state it cannot establish.
///
/// # Recovering
///
/// Read the registers back rather than inferring. [`read_configuration`],
/// [`read_thresholds`] and [`read_power_saving`] together establish the actual
/// state, and re-arming from there installs a known domain.
///
/// Once [`ThresholdMonitorStage::DisableMonitor`] appears in
/// [`confirmed`](Self::confirmed), the monitor is disabled and the device is
/// shut down, so it is not qualifying against a half-programmed domain while a
/// caller decides what to do.
///
/// That guarantee needs the write *confirmed*, not merely reached. A failure of
/// the disable write itself leaves it unknown: from an active monitor-disabled
/// start the device may still be active, and while re-arming an enabled monitor
/// it may still be enabled. Neither state is safe to assume — read back.
///
/// [`read_configuration`]: crate::Veml7700::read_configuration
/// [`read_thresholds`]: crate::Veml7700::read_thresholds
/// [`read_power_saving`]: crate::Veml7700::read_power_saving
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct ThresholdMonitorError<E> {
    /// Stage that failed. Its commit status is unknown.
    pub stage: ThresholdMonitorStage,
    /// Last stage that completed successfully, or `None` if none did.
    ///
    /// Everything up to and including this stage reached the device.
    pub confirmed: Option<ThresholdMonitorStage>,
    /// Underlying driver failure.
    pub source: Error<E>,
}
