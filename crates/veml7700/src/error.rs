//! Public error taxonomy.

use crate::config::{ConfigDecodeError, IntegrationTime};
use crate::measurement::MeasurementCapture;
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
    /// Controlled one-shot measurement sequence.
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
    /// Observed threshold registers were reversed.
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
///
/// # Reporting without allocation
///
/// Every error type here implements [`core::fmt::Display`], and implements
/// [`core::error::Error`] when the bus error does too. A chain can therefore be
/// walked into a fixed buffer with no allocator, no `String`, and no `std`:
///
/// ```rust
/// use core::fmt::Write;
///
/// /// Write an error and its causes into a caller-owned buffer.
/// fn report<W: Write>(sink: &mut W, error: &dyn core::error::Error) -> core::fmt::Result {
///     write!(sink, "{error}")?;
///     let mut cause = error.source();
///     while let Some(next) = cause {
///         write!(sink, ": {next}")?;
///         cause = next.source();
///     }
///     Ok(())
/// }
/// ```
///
/// Applied to a failed one-shot measurement, that produces something like:
///
/// ```text
/// one-shot measurement failed at activating: one-shot measurement failed during a
/// configuration write: arbitration lost
/// ```
///
/// The last link is the HAL's own error, preserved rather than flattened into a
/// string at the point of failure.
///
/// # If the bus error is not a `core::error::Error`
///
/// [`embedded_hal_async::i2c::Error`] requires only [`core::fmt::Debug`], so many
/// HAL error types are not [`core::error::Error`] and some are not
/// [`core::fmt::Display`]. That is why `Display` here is **not** bounded on the
/// bus error: the semantic context this crate owns — operation, register, stage —
/// is always printable. Only the chain needs more, and only the chain is lost.
///
/// [`embedded_hal_async::i2c::Error`]: https://docs.rs/embedded-hal-async
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
    /// Full ID register did not match the supported VEML7700 identity word.
    WrongDevice {
        /// Raw unexpected ID register value.
        observed: u16,
    },
}

/// Stage of a complete one-shot measurement.
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
    /// Only reached when the operation started from an active device. Under the
    /// driver reaction to `S-19`, this write changes only the shutdown bit.
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

/// Complete one-shot-measurement failure.
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum MeasureOnceError<E> {
    /// Failure before a pair was captured.
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
    /// A pair was captured, but restoration failed and hardware state is uncertain.
    RestoreFailed {
        /// Captured sample remains useful with explicit qualification.
        sample: MeasurementCapture,
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
    /// goes first while the monitor bit remains enabled.
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
/// Programming begins with one configuration read and then follows a
/// start-state-dependent write sequence. The fields distinguish confirmed
/// writes from the step that failed.
///
/// # What each field establishes
///
/// - [`stage`](Self::stage) identifies the read or write that failed.
///   [`ThresholdMonitorStage::ObserveConfiguration`] means no device-state write
///   was attempted.
/// - [`confirmed`](Self::confirmed) is the most recent write in the actual
///   branch that returned success. `None` means no write was confirmed.
/// - When `stage` is a write, its commit status is unknown: the device may remain
///   at `confirmed`, or may also contain that write's effect. Later stages were
///   not attempted. No rollback is claimed.
///
/// # Recovering
///
/// Read the registers back rather than inferring. [`read_configuration`],
/// [`read_thresholds`] and [`read_power_saving`] together establish the actual
/// state, and re-arming from there installs a known domain.
///
/// A confirmed [`ThresholdMonitorStage::DisableMonitor`] establishes disabled
/// and shut down until a later confirmed write. Before that point, the original
/// active/enabled state may remain. A failed final enable may leave the device
/// disabled and shut down or fully active in the requested domain.
///
/// [`read_configuration`]: crate::Veml7700::read_configuration
/// [`read_thresholds`]: crate::Veml7700::read_thresholds
/// [`read_power_saving`]: crate::Veml7700::read_power_saving
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct ThresholdMonitorError<E> {
    /// Stage that failed. A write stage has unknown commit status; an observation
    /// stage made no device-state write.
    pub stage: ThresholdMonitorStage,
    /// Most recent write that completed successfully, or `None` if none did.
    pub confirmed: Option<ThresholdMonitorStage>,
    /// Underlying driver failure.
    pub source: Error<E>,
}

// Standard error integration.
//
// Two deliberate bound choices shape everything below.
//
// `Display` is implemented **without bounding `E`**. `embedded_hal::i2c::Error`
// requires only `Debug`, so a great many real HAL error types are not `Display`;
// bounding on it would make these impls unavailable to exactly the callers this
// driver exists for. The message therefore carries the semantic context this
// crate owns -- operation, register, stage -- and leaves the concrete bus error
// to `source()`.
//
// `core::error::Error` *is* bounded, on `E: core::error::Error + 'static`. That
// is what `source()` requires, and it is additive: a bus error that does not
// implement `Error` simply does not get these impls on the wrapper. It does not
// stop the driver being used, which is why the bound sits on the impl rather
// than on the type.

// The context enums print as short lowercase phrases so a chained report reads
// as a sentence rather than as a list of type names.

impl core::fmt::Display for Operation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Inspect => "inspection",
            Self::Snapshot => "snapshot",
            Self::MeasureOnce => "one-shot measurement",
            Self::Configure => "configuration change",
            Self::ThresholdMonitor => "threshold-monitor programming",
        })
    }
}

impl core::fmt::Display for BusContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::ReadConfiguration => "a configuration read",
            Self::WriteConfiguration => "a configuration write",
            Self::ReadPowerSaving => "a power-saving read",
            Self::WritePowerSaving => "a power-saving write",
            Self::ReadAls => "an ALS read",
            Self::ReadWhite => "a white-channel read",
            Self::ReadDeviceId => "a device-ID read",
            Self::ReadThresholdStatus => "a threshold-status read",
            Self::ReadLowThreshold => "a low-threshold read",
            Self::ReadHighThreshold => "a high-threshold read",
            Self::WriteLowThreshold => "a low-threshold write",
            Self::WriteHighThreshold => "a high-threshold write",
        })
    }
}

impl core::fmt::Display for MeasureStage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::ValidateTiming => "timing validation",
            Self::ObserveConfiguration => "observing configuration",
            Self::ObservePowerSaving => "observing power saving",
            Self::EnterShutdown => "entering shutdown",
            Self::DisablePowerSaving => "disabling power saving",
            Self::PrepareMeasurement => "installing the measurement domain",
            Self::ActivateMeasurement => "activating",
            Self::FreezeResult => "freezing the result",
            Self::ReadAls => "reading ALS",
            Self::ReadWhite => "reading white",
            Self::RestoreConfiguration => "restoring configuration",
            Self::RestorePowerSaving => "restoring power saving",
        })
    }
}

impl core::fmt::Display for ThresholdMonitorStage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::ObserveConfiguration => "observing configuration",
            Self::EnterShutdown => "entering shutdown",
            Self::DisableMonitor => "disabling the monitor",
            Self::WriteLowThreshold => "writing the low threshold",
            Self::WriteHighThreshold => "writing the high threshold",
            Self::ApplyPowerSaving => "applying power saving",
            Self::EnableMonitor => "enabling the monitor",
        })
    }
}

impl core::fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ConfigurationDecode(_) => f.write_str("configuration register did not decode"),
            Self::PowerSavingDecode(_) => f.write_str("power-saving register did not decode"),
            Self::ThresholdStatusDecode(_) => {
                f.write_str("threshold-status register did not decode")
            }
            Self::ReversedThresholds => f.write_str("thresholds were reversed"),
            Self::ThresholdMonitorOwnsDomain => {
                f.write_str("an enabled threshold monitor owns this domain")
            }
            Self::TimingIntegrationMismatch {
                measurement,
                timing,
            } => write!(
                f,
                "timing was derived for {} ms but the measurement selects {} ms",
                timing.milliseconds(),
                measurement.milliseconds()
            ),
        }
    }
}

impl core::error::Error for ConfigurationError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::ConfigurationDecode(source) => Some(source),
            Self::PowerSavingDecode(source) => Some(source),
            Self::ThresholdStatusDecode(source) => Some(source),
            _ => None,
        }
    }
}

impl<E> core::fmt::Display for Error<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Bus {
                operation, context, ..
            } => write!(f, "{operation} failed during {context}"),
            // Deliberately not the inner message. This level's information is
            // that the failure was a configuration rejection rather than a bus
            // fault; the inner error is the next link, and repeating it here
            // would print it twice in a chained report.
            Self::Configuration(_) => f.write_str("configuration was rejected"),
        }
    }
}

impl<E: core::error::Error + 'static> core::error::Error for Error<E> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Bus { source, .. } => Some(source),
            Self::Configuration(source) => Some(source),
        }
    }
}

impl<E> core::fmt::Display for ProbeError<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotPresent => f.write_str("no device acknowledged the fixed address"),
            Self::Bus(_) => f.write_str("probe transaction failed"),
            Self::WrongDevice { observed } => {
                write!(f, "identity {observed:#06x} is not a supported VEML7700")
            }
        }
    }
}

impl<E: core::error::Error + 'static> core::error::Error for ProbeError<E> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Bus(source) => Some(source),
            // Neither absence nor a wrong identity has an underlying cause: both
            // are conclusions this driver reached, not failures it forwarded.
            _ => None,
        }
    }
}

impl<E> core::fmt::Display for MeasureOnceError<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Operation { stage, .. } => write!(f, "one-shot measurement failed at {stage}"),
            Self::RecoveryFailed {
                failed_stage,
                recovery_stage,
                ..
            } => write!(
                f,
                "one-shot measurement failed at {failed_stage} and restoration failed at \
                 {recovery_stage}; device state is uncertain"
            ),
            Self::RestoreFailed { stage, .. } => write!(
                f,
                "a sample was captured but restoration failed at {stage}; device state is uncertain"
            ),
        }
    }
}

impl<E: core::error::Error + 'static> core::error::Error for MeasureOnceError<E> {
    /// The **primary** failure, in every variant.
    ///
    /// `RecoveryFailed` carries two errors and a chain can only express one.
    /// Reporting the recovery failure as the cause would invert what happened:
    /// the primary failure is why the operation stopped, and the recovery
    /// failure is why the device was left uncertain. The recovery error stays
    /// available as an ordinary field, which is the honest place for a second
    /// independent failure.
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Operation { source, .. }
            | Self::RecoveryFailed { source, .. }
            | Self::RestoreFailed { source, .. } => Some(source),
        }
    }
}

impl<E> core::fmt::Display for ThresholdMonitorError<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.confirmed {
            Some(confirmed) => write!(
                f,
                "threshold programming failed at {}; {} was the last confirmed write",
                self.stage, confirmed
            ),
            None => write!(
                f,
                "threshold programming failed at {}; no write was confirmed",
                self.stage
            ),
        }
    }
}

impl<E: core::error::Error + 'static> core::error::Error for ThresholdMonitorError<E> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        Some(&self.source)
    }
}

#[cfg(test)]
mod standard_error_tests {
    use super::*;
    use crate::config::{Gain, IntegrationTime, MeasurementConfig};
    use crate::measurement::{AlsCounts, MeasurementPairCoherence, WhiteCounts};
    use core::error::Error as _;
    use core::fmt::Write as _;

    /// A HAL error that participates in the standard chain.
    #[derive(Debug, PartialEq, Eq)]
    struct ReportableBusFault;

    impl core::fmt::Display for ReportableBusFault {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_str("arbitration lost")
        }
    }

    impl core::error::Error for ReportableBusFault {}

    /// A HAL error that implements only `Debug`, which is all
    /// `embedded_hal::i2c::Error` requires. Many real ones look like this.
    #[derive(Debug, PartialEq, Eq)]
    struct BareBusFault;

    /// Fixed-capacity sink: no allocator, no `String`, no `std`.
    struct Sink {
        buffer: [u8; 256],
        used: usize,
    }

    impl Sink {
        const fn new() -> Self {
            Self {
                buffer: [0; 256],
                used: 0,
            }
        }

        fn as_str(&self) -> &str {
            core::str::from_utf8(&self.buffer[..self.used]).expect("valid UTF-8")
        }
    }

    impl core::fmt::Write for Sink {
        fn write_str(&mut self, text: &str) -> core::fmt::Result {
            let bytes = text.as_bytes();
            let end = self.used + bytes.len();
            if end > self.buffer.len() {
                return Err(core::fmt::Error);
            }
            self.buffer[self.used..end].copy_from_slice(bytes);
            self.used = end;
            Ok(())
        }
    }

    /// Walk a standard error chain into a fixed buffer.
    fn report(error: &dyn core::error::Error) -> Sink {
        let mut sink = Sink::new();
        write!(sink, "{error}").expect("fits");
        let mut cause = error.source();
        while let Some(next) = cause {
            write!(sink, ": {next}").expect("fits");
            cause = next.source();
        }
        sink
    }

    fn bus_failure<E>(source: E) -> Error<E> {
        Error::Bus {
            operation: Operation::MeasureOnce,
            context: BusContext::WriteConfiguration,
            source,
        }
    }

    #[test]
    fn a_reportable_bus_error_reaches_the_end_of_the_chain() {
        let error = bus_failure(ReportableBusFault);
        assert_eq!(
            report(&error).as_str(),
            "one-shot measurement failed during a configuration write: arbitration lost"
        );
    }

    #[test]
    fn a_bus_error_that_is_not_a_standard_error_still_works() {
        // The wrapper is `Display` regardless: the semantic context this crate
        // owns is never hidden behind a bound the HAL does not satisfy. Only the
        // `Error` impl -- and therefore the chain -- requires more of `E`.
        let error = bus_failure(BareBusFault);
        let mut sink = Sink::new();
        write!(sink, "{error}").expect("fits");
        assert_eq!(
            sink.as_str(),
            "one-shot measurement failed during a configuration write"
        );
    }

    #[test]
    fn a_configuration_failure_chains_to_its_decode_cause() {
        let error: Error<ReportableBusFault> =
            Error::Configuration(ConfigurationError::ConfigurationDecode(
                ConfigDecodeError::ReservedBits { observed: 0x2000 },
            ));
        assert_eq!(
            report(&error).as_str(),
            concat!(
                "configuration was rejected: configuration register did not decode: ",
                "reserved configuration bits were set: 0x2000"
            )
        );
    }

    #[test]
    fn a_conclusion_this_driver_reached_has_no_cause() {
        // Absence and wrong identity are findings, not forwarded failures, so
        // reporting a cause for them would invent one.
        let absent: ProbeError<ReportableBusFault> = ProbeError::NotPresent;
        assert!(absent.source().is_none());
        assert_eq!(
            report(&absent).as_str(),
            "no device acknowledged the fixed address"
        );

        let mismatch: ProbeError<ReportableBusFault> = ProbeError::WrongDevice { observed: 0x1234 };
        assert!(mismatch.source().is_none());
    }

    #[test]
    fn a_nested_recovery_failure_reports_the_primary_cause() {
        let error = MeasureOnceError::RecoveryFailed {
            failed_stage: MeasureStage::ActivateMeasurement,
            source: bus_failure(ReportableBusFault),
            recovery_stage: MeasureStage::RestoreConfiguration,
            recovery_source: bus_failure(ReportableBusFault),
        };
        // Both failures are named in the message, because both happened. The
        // chain follows the primary one: it is why the operation stopped.
        assert_eq!(
            report(&error).as_str(),
            "one-shot measurement failed at activating and restoration failed at restoring \
             configuration; device state is uncertain: one-shot measurement failed during a \
             configuration write: arbitration lost"
        );
        let MeasureOnceError::RecoveryFailed {
            recovery_source, ..
        } = &error
        else {
            unreachable!()
        };
        assert!(matches!(recovery_source, Error::Bus { .. }));
    }

    #[test]
    fn a_captured_sample_survives_a_reported_restoration_failure() {
        let configuration = MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms100);
        let error = MeasureOnceError::RestoreFailed {
            sample: MeasurementCapture {
                als: AlsCounts::from_counts(0x1234),
                white: WhiteCounts::from_counts(0x5678),
                configuration,
                nominal_illuminance: AlsCounts::from_counts(0x1234)
                    .nominal_micro_lux(configuration),
                requested_wait_us: 133_500,
                coherence: MeasurementPairCoherence::FrozenAfterRequestedWait,
            },
            stage: MeasureStage::RestorePowerSaving,
            source: bus_failure(ReportableBusFault),
        };
        assert!(
            report(&error).as_str().starts_with(
                "a sample was captured but restoration failed at restoring power saving"
            )
        );
        let MeasureOnceError::RestoreFailed { sample, .. } = &error else {
            unreachable!()
        };
        assert_eq!(sample.als, AlsCounts::from_counts(0x1234));
    }

    #[test]
    fn threshold_failures_report_confirmed_progress() {
        let unconfirmed = ThresholdMonitorError {
            stage: ThresholdMonitorStage::DisableMonitor,
            confirmed: None,
            source: bus_failure(ReportableBusFault),
        };
        assert_eq!(
            report(&unconfirmed).as_str(),
            "threshold programming failed at disabling the monitor; no write was confirmed: \
             one-shot measurement failed during a configuration write: arbitration lost"
        );

        let partial = ThresholdMonitorError {
            stage: ThresholdMonitorStage::ApplyPowerSaving,
            confirmed: Some(ThresholdMonitorStage::WriteHighThreshold),
            source: bus_failure(ReportableBusFault),
        };
        assert!(report(&partial).as_str().starts_with(
            "threshold programming failed at applying power saving; writing the high threshold \
             was the last confirmed write"
        ));
    }
}
