//! I²C resource ownership and VEML7700 operation sequencing.

use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::i2c::{Error as I2cError, ErrorKind, I2c, NoAcknowledgeSource};

use crate::config::{
    ConfigWord, ConfigurationSnapshot, MeasurementConfig, PowerState, ThresholdMonitorState,
};
use crate::error::{
    BusContext, ConfigurationError, Error, MeasureOnceError, MeasureStage, Operation, ProbeError,
    ThresholdMonitorError, ThresholdMonitorStage,
};
use crate::id::DeviceId;
use crate::measurement::{
    AlsCounts, DeviceSnapshot, FreshMeasurement, MeasurementPairCoherence, SnapshotMeasurement,
    WhiteCounts,
};
use crate::power::{PowerSavingConfig, PowerSavingSnapshot, decode_power_saving};
use crate::register::Register;
use crate::threshold::{ThresholdMonitorConfig, ThresholdStatus, Thresholds};
use crate::timing::MeasurementTiming;

/// Fixed 7-bit I²C address of the VEML7700.
pub const I2C_ADDRESS: u8 = 0x10;

/// Async VEML7700 driver owning one I²C resource.
pub struct Veml7700<I2C> {
    i2c: I2C,
}

impl<I2C> Veml7700<I2C> {
    /// Construct an inert driver. Performs no I²C transaction.
    pub const fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    /// Return the exact owned I²C resource.
    pub fn release(self) -> I2C {
        self.i2c
    }
}

impl<I2C> Veml7700<I2C>
where
    I2C: I2c,
{
    /// Probe the fixed address and validate the VEML7700 ID register.
    pub async fn probe(&mut self) -> Result<DeviceId, ProbeError<I2C::Error>> {
        let mut bytes = [0_u8; 2];
        match self
            .i2c
            .write_read(I2C_ADDRESS, &[Register::DeviceId.pointer()], &mut bytes)
            .await
        {
            Ok(()) => {
                let id = DeviceId::from_raw(u16::from_le_bytes(bytes));
                if id.is_supported() {
                    Ok(id)
                } else {
                    Err(ProbeError::WrongDevice { observed: id.raw() })
                }
            }
            Err(source) => match source.kind() {
                ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address) => {
                    Err(ProbeError::NotPresent)
                }
                _ => Err(ProbeError::Bus(source)),
            },
        }
    }

    /// Read the identity register without requiring a match.
    pub async fn read_device_id(&mut self) -> Result<DeviceId, Error<I2C::Error>> {
        self.read_word(
            Register::DeviceId,
            Operation::Inspect,
            BusContext::ReadDeviceId,
        )
        .await
        .map(DeviceId::from_raw)
    }

    /// Read and strictly decode the configuration register.
    pub async fn read_configuration(&mut self) -> Result<ConfigurationSnapshot, Error<I2C::Error>> {
        self.read_configuration_for(Operation::Inspect).await
    }

    /// Read and strictly decode the power-saving register.
    pub async fn read_power_saving(&mut self) -> Result<PowerSavingSnapshot, Error<I2C::Error>> {
        self.read_power_saving_for(Operation::Inspect).await
    }

    /// Read the latest ALS register without any freshness claim.
    pub async fn read_als_snapshot(&mut self) -> Result<AlsCounts, Error<I2C::Error>> {
        self.read_als_for(Operation::Snapshot).await
    }

    /// Read the latest white-channel register without any freshness claim.
    pub async fn read_white_snapshot(&mut self) -> Result<WhiteCounts, Error<I2C::Error>> {
        self.read_white_for(Operation::Snapshot).await
    }

    /// Read the polled threshold flags.
    ///
    /// The VEML7700 has no dedicated interrupt pin. This method makes no claim
    /// about flag-clearing side effects beyond the vendor's documented read.
    pub async fn read_threshold_status(&mut self) -> Result<ThresholdStatus, Error<I2C::Error>> {
        self.read_threshold_status_for(Operation::Inspect).await
    }

    /// Read both raw threshold registers.
    pub async fn read_thresholds(&mut self) -> Result<Thresholds, Error<I2C::Error>> {
        self.read_thresholds_for(Operation::Inspect).await
    }

    /// Perform a read-only diagnostic sweep without claiming fresh optical data.
    pub async fn inspect(&mut self) -> Result<DeviceSnapshot, Error<I2C::Error>> {
        Ok(DeviceSnapshot {
            id: self.read_device_id().await?,
            configuration: self.read_configuration_for(Operation::Inspect).await?,
            power_saving: self.read_power_saving_for(Operation::Inspect).await?,
            thresholds: self.read_thresholds_for(Operation::Inspect).await?,
            threshold_status: self.read_threshold_status_for(Operation::Inspect).await?,
        })
    }

    /// Read configuration, ALS, and white registers as a diagnostic snapshot.
    ///
    /// The ALS and white registers are sequential transactions and may straddle
    /// an autonomous refresh. In shutdown they may be retained old data.
    pub async fn snapshot(&mut self) -> Result<SnapshotMeasurement, Error<I2C::Error>> {
        let configuration = self.read_configuration_for(Operation::Snapshot).await?;
        let power_saving = self.read_power_saving_for(Operation::Snapshot).await?;
        let als = self.read_als_for(Operation::Snapshot).await?;
        let white = self.read_white_for(Operation::Snapshot).await?;
        Ok(SnapshotMeasurement {
            als,
            white,
            configuration,
            power_saving,
            coherence: MeasurementPairCoherence::SequentialRegisters,
        })
    }

    /// Change gain and integration time while preserving unrelated fields.
    ///
    /// An enabled threshold monitor prevents retargeting its measurement domain.
    ///
    /// # Sequence
    ///
    /// The sources require shutdown before any reconfiguration, so an active
    /// device is shut down first, reconfigured while shut down, and returned to
    /// active last — three writes rather than one. A device that is already shut
    /// down takes the single-write path and stays shut down.
    ///
    /// # State after a failure
    ///
    /// Because shutdown comes first, a failure part way through can leave an
    /// originally active device shut down, with the measurement domain either
    /// old or new. Read the configuration back to establish which. This is the
    /// cost of following the required sequence: the alternative is a write the
    /// sources do not sanction.
    pub async fn set_measurement_config(
        &mut self,
        measurement: MeasurementConfig,
    ) -> Result<(), Error<I2C::Error>> {
        let current = self.read_configuration_for(Operation::Configure).await?;
        if current.threshold_monitor == ThresholdMonitorState::Enabled
            && current.measurement != measurement
        {
            return Err(Error::Configuration(
                ConfigurationError::ThresholdMonitorOwnsDomain,
            ));
        }
        // Nothing to change, and doing nothing matters here. The sequence below
        // cycles power; running it for a call that alters no field would
        // interrupt an enabled monitor's active domain, which is the one thing
        // the ownership guard above exists to prevent.
        if current.measurement == measurement {
            return Ok(());
        }
        if current.power_state == PowerState::Shutdown {
            return self
                .write_configuration_for(
                    current.with_measurement(measurement),
                    Operation::Configure,
                )
                .await;
        }

        // Shutdown carries the old domain: the shutdown bit must land before the
        // new gain and integration time, not with them.
        self.write_configuration_for(
            current.with_power_state(PowerState::Shutdown),
            Operation::Configure,
        )
        .await?;
        let reconfigured = current
            .with_measurement(measurement)
            .with_power_state(PowerState::Shutdown);
        self.write_configuration_for(reconfigured, Operation::Configure)
            .await?;
        self.write_configuration_for(
            reconfigured.with_power_state(PowerState::Active),
            Operation::Configure,
        )
        .await
    }

    /// Change active/shutdown state while preserving unrelated fields.
    ///
    /// An enabled threshold monitor prevents changing its active monitored state.
    pub async fn set_power_state(
        &mut self,
        power_state: PowerState,
    ) -> Result<(), Error<I2C::Error>> {
        let current = self.read_configuration_for(Operation::Configure).await?;
        if current.threshold_monitor == ThresholdMonitorState::Enabled
            && current.power_state != power_state
        {
            return Err(Error::Configuration(
                ConfigurationError::ThresholdMonitorOwnsDomain,
            ));
        }
        self.write_configuration_for(current.with_power_state(power_state), Operation::Configure)
            .await
    }

    /// Change power-saving cadence while preserving the configuration register.
    ///
    /// An enabled threshold monitor prevents changing its qualification cadence.
    ///
    /// # Sequence
    ///
    /// Power saving is part of the measurement domain, so the same requirement
    /// applies: an active device is shut down first, the cadence is written while
    /// shut down, and the device is returned to active last. A device that is
    /// already shut down takes the single-write path.
    ///
    /// # State after a failure
    ///
    /// A failure part way through can leave an originally active device shut
    /// down, with the cadence either old or new. Read both registers back to
    /// establish which.
    pub async fn set_power_saving(
        &mut self,
        power_saving: PowerSavingConfig,
    ) -> Result<(), Error<I2C::Error>> {
        let configuration = self.read_configuration_for(Operation::Configure).await?;
        let current = self.read_power_saving_for(Operation::Configure).await?;
        if configuration.threshold_monitor == ThresholdMonitorState::Enabled
            && current.as_config() != power_saving
        {
            return Err(Error::Configuration(
                ConfigurationError::ThresholdMonitorOwnsDomain,
            ));
        }
        // Same reasoning as `set_measurement_config`: an unchanged cadence must
        // not cost the caller a power cycle.
        if current.as_config() == power_saving {
            return Ok(());
        }
        if configuration.power_state == PowerState::Shutdown {
            return self
                .write_power_saving_for(power_saving, Operation::Configure)
                .await;
        }

        self.write_configuration_for(
            configuration.with_power_state(PowerState::Shutdown),
            Operation::Configure,
        )
        .await?;
        self.write_power_saving_for(power_saving, Operation::Configure)
            .await?;
        self.write_configuration_for(configuration, Operation::Configure)
            .await
    }

    /// Capture one fresh measurement using conservative vendor-derived timing.
    pub async fn measure_once<D>(
        &mut self,
        delay: &mut D,
        measurement: MeasurementConfig,
    ) -> Result<FreshMeasurement, MeasureOnceError<I2C::Error>>
    where
        D: DelayNs,
    {
        self.measure_once_with_timing(
            delay,
            measurement,
            MeasurementTiming::conservative(measurement.integration_time()),
        )
        .await
    }

    /// Capture one fresh measurement using explicit conservative-or-longer timing.
    ///
    /// [`MeasurementTiming`] cannot represent a wait shorter than the documented
    /// conservative minimum for its selected integration time.
    ///
    /// The operation disables power-saving cadence, installs the requested
    /// measurement domain while shut down, creates a known shutdown-to-active
    /// wake edge, waits, enters shutdown again to freeze data, reads ALS and
    /// white, then restores the original configuration and power-saving register.
    pub async fn measure_once_with_timing<D>(
        &mut self,
        delay: &mut D,
        measurement: MeasurementConfig,
        timing: MeasurementTiming,
    ) -> Result<FreshMeasurement, MeasureOnceError<I2C::Error>>
    where
        D: DelayNs,
    {
        if timing.integration_time() != measurement.integration_time() {
            return Err(MeasureOnceError::Operation {
                stage: MeasureStage::ValidateTiming,
                source: Error::Configuration(ConfigurationError::TimingIntegrationMismatch {
                    measurement: measurement.integration_time(),
                    timing: timing.integration_time(),
                }),
            });
        }

        let original_configuration = self
            .read_configuration_for(Operation::MeasureOnce)
            .await
            .map_err(|source| MeasureOnceError::Operation {
                stage: MeasureStage::ObserveConfiguration,
                source,
            })?;
        if original_configuration.threshold_monitor == ThresholdMonitorState::Enabled {
            return Err(MeasureOnceError::Operation {
                stage: MeasureStage::ObserveConfiguration,
                source: Error::Configuration(ConfigurationError::ThresholdMonitorOwnsDomain),
            });
        }
        let original_power_saving = self
            .read_power_saving_for(Operation::MeasureOnce)
            .await
            .map_err(|source| MeasureOnceError::Operation {
                stage: MeasureStage::ObservePowerSaving,
                source,
            })?;

        // Shutdown before any reconfiguration, carrying the original domain
        // unchanged. Every later write then happens on a shut-down device, which
        // also makes the recovery path safe: it can never write while active.
        if original_configuration.power_state == PowerState::Active
            && let Err(source) = self
                .write_configuration_for(
                    original_configuration.with_power_state(PowerState::Shutdown),
                    Operation::MeasureOnce,
                )
                .await
        {
            // Report without attempting restoration. Nothing has been mutated
            // yet, so there is nothing to restore — and the device may well
            // still be active, which is exactly where the generic restoration
            // sequence would write the power-saving register. That would commit
            // the active write this operation exists to avoid, and would turn a
            // clean single fault into `RecoveryFailed`.
            return Err(MeasureOnceError::Operation {
                stage: MeasureStage::EnterShutdown,
                source,
            });
        }

        if let Err(source) = self
            .write_power_saving_for(
                PowerSavingConfig::new(false, original_power_saving.mode),
                Operation::MeasureOnce,
            )
            .await
        {
            return Err(self
                .recover_pre_capture(
                    MeasureStage::DisablePowerSaving,
                    source,
                    original_configuration,
                    original_power_saving,
                )
                .await);
        }

        let prepared = original_configuration
            .with_measurement(measurement)
            .with_monitor(ThresholdMonitorState::Disabled)
            .with_power_state(PowerState::Shutdown);
        if let Err(source) = self
            .write_configuration_for(prepared, Operation::MeasureOnce)
            .await
        {
            return Err(self
                .recover_pre_capture(
                    MeasureStage::PrepareMeasurement,
                    source,
                    original_configuration,
                    original_power_saving,
                )
                .await);
        }

        let active = prepared.with_power_state(PowerState::Active);
        if let Err(source) = self
            .write_configuration_for(active, Operation::MeasureOnce)
            .await
        {
            return Err(self
                .recover_pre_capture(
                    MeasureStage::ActivateMeasurement,
                    source,
                    original_configuration,
                    original_power_saving,
                )
                .await);
        }

        delay.delay_us(timing.total_us()).await;

        let frozen = active.with_power_state(PowerState::Shutdown);
        if let Err(source) = self
            .write_configuration_for(frozen, Operation::MeasureOnce)
            .await
        {
            return Err(self
                .recover_pre_capture(
                    MeasureStage::FreezeResult,
                    source,
                    original_configuration,
                    original_power_saving,
                )
                .await);
        }

        let als = match self.read_als_for(Operation::MeasureOnce).await {
            Ok(value) => value,
            Err(source) => {
                return Err(self
                    .recover_pre_capture(
                        MeasureStage::ReadAls,
                        source,
                        original_configuration,
                        original_power_saving,
                    )
                    .await);
            }
        };
        let white = match self.read_white_for(Operation::MeasureOnce).await {
            Ok(value) => value,
            Err(source) => {
                return Err(self
                    .recover_pre_capture(
                        MeasureStage::ReadWhite,
                        source,
                        original_configuration,
                        original_power_saving,
                    )
                    .await);
            }
        };
        let sample = FreshMeasurement {
            als,
            white,
            configuration: measurement,
            nominal_illuminance: als.nominal_micro_lux(measurement),
            waited_us: timing.total_us(),
            coherence: MeasurementPairCoherence::FrozenAfterFreshWait,
        };

        if let Err((stage, source)) = self
            .restore_state(original_configuration, original_power_saving)
            .await
        {
            return Err(MeasureOnceError::RestoreFailed {
                sample,
                stage,
                source,
            });
        }
        Ok(sample)
    }

    /// Program and enable a complete threshold-monitor domain.
    ///
    /// Configuration is applied disable-first and enable-last. Threshold status
    /// is polled through register `0x06`; there is no GPIO to own.
    ///
    /// # Sequence
    ///
    /// Accepts an active or shut-down starting state. The first write disables
    /// the monitor and enters shutdown while carrying the existing measurement
    /// domain; thresholds, cadence, and the new domain are written to a
    /// shut-down device; the final write installs the monitored domain and
    /// returns to active. The sources require shutdown before any
    /// reconfiguration, so no field other than the shutdown and monitor bits
    /// changes while the device is active.
    ///
    /// # State after a failure
    ///
    /// [`ThresholdMonitorError::stage`] names the write that failed. Every stage
    /// after the first leaves the device shut down with the monitor disabled, so
    /// it is not measuring against a half-programmed domain. Which thresholds
    /// and cadence are installed depends on how far the sequence reached; read
    /// them back rather than assuming, and re-arm to establish a known domain.
    pub async fn arm_threshold_monitor(
        &mut self,
        monitor: ThresholdMonitorConfig,
    ) -> Result<(), ThresholdMonitorError<I2C::Error>> {
        let current = self
            .read_configuration_for(Operation::ThresholdMonitor)
            .await
            .map_err(|source| ThresholdMonitorError {
                stage: ThresholdMonitorStage::ObserveConfiguration,
                source,
            })?;

        // A single write may move the shutdown bit or the monitor bit, not both:
        // the device accepts either as a transition, but the two together are a
        // reconfiguration, which the sources require shutdown for first. That
        // only bites when re-arming an enabled monitor on an active device —
        // every other starting state needs one write.
        if current.power_state == PowerState::Active
            && current.threshold_monitor == ThresholdMonitorState::Enabled
        {
            self.write_configuration_for(
                current.with_power_state(PowerState::Shutdown),
                Operation::ThresholdMonitor,
            )
            .await
            .map_err(|source| ThresholdMonitorError {
                stage: ThresholdMonitorStage::EnterShutdown,
                source,
            })?;
        }

        // Thresholds, cadence and the new domain are then all written to a
        // shut-down device, and the monitored domain is enabled last.
        let disabled = current
            .with_monitor(ThresholdMonitorState::Disabled)
            .with_power_state(PowerState::Shutdown);
        self.write_configuration_for(disabled, Operation::ThresholdMonitor)
            .await
            .map_err(|source| ThresholdMonitorError {
                stage: ThresholdMonitorStage::DisableMonitor,
                source,
            })?;
        self.write_word(
            Register::LowThreshold,
            monitor.thresholds.low().counts(),
            Operation::ThresholdMonitor,
            BusContext::WriteLowThreshold,
        )
        .await
        .map_err(|source| ThresholdMonitorError {
            stage: ThresholdMonitorStage::WriteLowThreshold,
            source,
        })?;
        self.write_word(
            Register::HighThreshold,
            monitor.thresholds.high().counts(),
            Operation::ThresholdMonitor,
            BusContext::WriteHighThreshold,
        )
        .await
        .map_err(|source| ThresholdMonitorError {
            stage: ThresholdMonitorStage::WriteHighThreshold,
            source,
        })?;
        self.write_power_saving_for(monitor.power_saving, Operation::ThresholdMonitor)
            .await
            .map_err(|source| ThresholdMonitorError {
                stage: ThresholdMonitorStage::ApplyPowerSaving,
                source,
            })?;

        let enabled = disabled
            .with_measurement(monitor.measurement)
            .with_persistence(monitor.persistence)
            .with_power_state(PowerState::Active)
            .with_monitor(ThresholdMonitorState::Enabled);
        self.write_configuration_for(enabled, Operation::ThresholdMonitor)
            .await
            .map_err(|source| ThresholdMonitorError {
                stage: ThresholdMonitorStage::EnableMonitor,
                source,
            })
    }

    /// Disable threshold monitoring while preserving all other configuration fields.
    pub async fn disable_threshold_monitor(&mut self) -> Result<(), Error<I2C::Error>> {
        let current = self
            .read_configuration_for(Operation::ThresholdMonitor)
            .await?;
        self.write_configuration_for(
            current.with_monitor(ThresholdMonitorState::Disabled),
            Operation::ThresholdMonitor,
        )
        .await
    }

    async fn recover_pre_capture(
        &mut self,
        failed_stage: MeasureStage,
        source: Error<I2C::Error>,
        original_configuration: ConfigurationSnapshot,
        original_power_saving: PowerSavingSnapshot,
    ) -> MeasureOnceError<I2C::Error> {
        match self
            .restore_state(original_configuration, original_power_saving)
            .await
        {
            Ok(()) => MeasureOnceError::Operation {
                stage: failed_stage,
                source,
            },
            Err((recovery_stage, recovery_source)) => MeasureOnceError::RecoveryFailed {
                failed_stage,
                source,
                recovery_stage,
                recovery_source,
            },
        }
    }

    async fn restore_state(
        &mut self,
        configuration: ConfigurationSnapshot,
        power_saving: PowerSavingSnapshot,
    ) -> Result<(), (MeasureStage, Error<I2C::Error>)> {
        self.write_power_saving_for(power_saving.as_config(), Operation::MeasureOnce)
            .await
            .map_err(|source| (MeasureStage::RestorePowerSaving, source))?;
        self.write_configuration_for(configuration, Operation::MeasureOnce)
            .await
            .map_err(|source| (MeasureStage::RestoreConfiguration, source))
    }

    async fn read_configuration_for(
        &mut self,
        operation: Operation,
    ) -> Result<ConfigurationSnapshot, Error<I2C::Error>> {
        let word = self
            .read_word(
                Register::Configuration,
                operation,
                BusContext::ReadConfiguration,
            )
            .await?;
        ConfigWord::from_raw(word)
            .decode()
            .map_err(|error| Error::Configuration(ConfigurationError::ConfigurationDecode(error)))
    }

    async fn write_configuration_for(
        &mut self,
        configuration: ConfigurationSnapshot,
        operation: Operation,
    ) -> Result<(), Error<I2C::Error>> {
        self.write_word(
            Register::Configuration,
            ConfigWord::from_snapshot(configuration).raw(),
            operation,
            BusContext::WriteConfiguration,
        )
        .await
    }

    async fn read_power_saving_for(
        &mut self,
        operation: Operation,
    ) -> Result<PowerSavingSnapshot, Error<I2C::Error>> {
        let word = self
            .read_word(
                Register::PowerSaving,
                operation,
                BusContext::ReadPowerSaving,
            )
            .await?;
        decode_power_saving(word)
            .map_err(|error| Error::Configuration(ConfigurationError::PowerSavingDecode(error)))
    }

    async fn write_power_saving_for(
        &mut self,
        power_saving: PowerSavingConfig,
        operation: Operation,
    ) -> Result<(), Error<I2C::Error>> {
        self.write_word(
            Register::PowerSaving,
            power_saving.encode(),
            operation,
            BusContext::WritePowerSaving,
        )
        .await
    }

    async fn read_als_for(&mut self, operation: Operation) -> Result<AlsCounts, Error<I2C::Error>> {
        self.read_word(Register::Als, operation, BusContext::ReadAls)
            .await
            .map(AlsCounts::from_counts)
    }

    async fn read_white_for(
        &mut self,
        operation: Operation,
    ) -> Result<WhiteCounts, Error<I2C::Error>> {
        self.read_word(Register::White, operation, BusContext::ReadWhite)
            .await
            .map(WhiteCounts::from_counts)
    }

    async fn read_threshold_status_for(
        &mut self,
        operation: Operation,
    ) -> Result<ThresholdStatus, Error<I2C::Error>> {
        let word = self
            .read_word(
                Register::ThresholdStatus,
                operation,
                BusContext::ReadThresholdStatus,
            )
            .await?;
        ThresholdStatus::decode(word)
            .map_err(|error| Error::Configuration(ConfigurationError::ThresholdStatusDecode(error)))
    }

    async fn read_thresholds_for(
        &mut self,
        operation: Operation,
    ) -> Result<Thresholds, Error<I2C::Error>> {
        let low = self
            .read_word(
                Register::LowThreshold,
                operation,
                BusContext::ReadLowThreshold,
            )
            .await?;
        let high = self
            .read_word(
                Register::HighThreshold,
                operation,
                BusContext::ReadHighThreshold,
            )
            .await?;
        Thresholds::new(AlsCounts::from_counts(low), AlsCounts::from_counts(high))
            .ok_or(Error::Configuration(ConfigurationError::ReversedThresholds))
    }

    async fn read_word(
        &mut self,
        register: Register,
        operation: Operation,
        context: BusContext,
    ) -> Result<u16, Error<I2C::Error>> {
        let mut bytes = [0_u8; 2];
        self.i2c
            .write_read(I2C_ADDRESS, &[register.pointer()], &mut bytes)
            .await
            .map_err(|source| Error::Bus {
                operation,
                context,
                source,
            })?;
        Ok(u16::from_le_bytes(bytes))
    }

    async fn write_word(
        &mut self,
        register: Register,
        value: u16,
        operation: Operation,
        context: BusContext,
    ) -> Result<(), Error<I2C::Error>> {
        let [low, high] = value.to_le_bytes();
        self.i2c
            .write(I2C_ADDRESS, &[register.pointer(), low, high])
            .await
            .map_err(|source| Error::Bus {
                operation,
                context,
                source,
            })
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use embedded_hal_async::delay::DelayNs;
    use embedded_hal_async::i2c::{ErrorKind, NoAcknowledgeSource};
    use futures::executor::block_on;

    use super::{I2C_ADDRESS, Veml7700};
    use crate::testing::scripted_i2c::{Expectation, ScriptError, ScriptedI2c};
    use crate::{
        AlsCounts, BusContext, ConfigurationError, Error, Gain, IntegrationTime, MeasureOnceError,
        MeasureStage, MeasurementConfig, Operation, Persistence, PowerSavingConfig,
        PowerSavingMode, PowerState, ProbeError, ThresholdMonitorConfig, ThresholdMonitorError,
        ThresholdMonitorStage, Thresholds, WhiteCounts,
    };

    struct RecordingDelay {
        elapsed_ns: u64,
    }

    impl DelayNs for RecordingDelay {
        async fn delay_ns(&mut self, ns: u32) {
            self.elapsed_ns += u64::from(ns);
        }
    }

    fn read_word(register: u8, value: u16) -> Expectation {
        Expectation::WriteRead {
            address: I2C_ADDRESS,
            write: vec![register],
            returns: value.to_le_bytes().to_vec(),
            result: Ok(()),
        }
    }

    fn read_failure(register: u8, error: ScriptError) -> Expectation {
        Expectation::WriteRead {
            address: I2C_ADDRESS,
            write: vec![register],
            returns: vec![0, 0],
            result: Err(error),
        }
    }

    fn write_word(register: u8, value: u16, result: Result<(), ScriptError>) -> Expectation {
        let [low, high] = value.to_le_bytes();
        Expectation::Write {
            address: I2C_ADDRESS,
            data: vec![register, low, high],
            result,
        }
    }

    #[test]
    fn fresh_measurement_uses_known_wake_edge_and_restores_state() {
        let bus = ScriptedI2c::new([
            read_word(0x00, 0x0001),
            read_word(0x03, 0x0000),
            write_word(0x03, 0x0000, Ok(())),
            write_word(0x00, 0x1001, Ok(())),
            write_word(0x00, 0x1000, Ok(())),
            write_word(0x00, 0x1001, Ok(())),
            read_word(0x04, 0x1234),
            read_word(0x05, 0x5678),
            write_word(0x03, 0x0000, Ok(())),
            write_word(0x00, 0x0001, Ok(())),
        ]);
        let mut delay = RecordingDelay { elapsed_ns: 0 };
        let mut sensor = Veml7700::new(bus);
        let sample =
            block_on(sensor.measure_once(&mut delay, MeasurementConfig::safe_bright_start()))
                .unwrap();
        assert_eq!(sample.als, AlsCounts::from_counts(0x1234));
        assert_eq!(sample.white, WhiteCounts::from_counts(0x5678));
        assert_eq!(sample.waited_us, 133_500);
        assert_eq!(delay.elapsed_ns, 133_500_000);
        sensor.release().done();
    }

    #[test]
    fn explicit_timing_must_match_the_measurement_integration_time() {
        let bus = ScriptedI2c::new([]);
        let mut delay = RecordingDelay { elapsed_ns: 0 };
        let mut sensor = Veml7700::new(bus);
        let result = block_on(sensor.measure_once_with_timing(
            &mut delay,
            MeasurementConfig::new(crate::Gain::Div8, crate::IntegrationTime::Ms800),
            crate::MeasurementTiming::conservative(crate::IntegrationTime::Ms25),
        ));
        assert_eq!(
            result,
            Err(MeasureOnceError::Operation {
                stage: MeasureStage::ValidateTiming,
                source: Error::Configuration(ConfigurationError::TimingIntegrationMismatch {
                    measurement: crate::IntegrationTime::Ms800,
                    timing: crate::IntegrationTime::Ms25,
                },),
            })
        );
        assert_eq!(delay.elapsed_ns, 0);
        sensor.release().done();
    }

    #[test]
    fn disable_power_saving_failure_is_followed_by_state_restoration() {
        let failure = ScriptError::new(ErrorKind::Bus);
        let bus = ScriptedI2c::new([
            read_word(0x00, 0x0001),
            read_word(0x03, 0x0000),
            write_word(0x03, 0x0000, Err(failure)),
            write_word(0x03, 0x0000, Ok(())),
            write_word(0x00, 0x0001, Ok(())),
        ]);
        let mut delay = RecordingDelay { elapsed_ns: 0 };
        let mut sensor = Veml7700::new(bus);
        let result =
            block_on(sensor.measure_once(&mut delay, MeasurementConfig::safe_bright_start()));
        assert_eq!(
            result,
            Err(MeasureOnceError::Operation {
                stage: MeasureStage::DisablePowerSaving,
                source: Error::Bus {
                    operation: Operation::MeasureOnce,
                    context: BusContext::WritePowerSaving,
                    source: failure,
                },
            })
        );
        assert_eq!(delay.elapsed_ns, 0);
        sensor.release().done();
    }

    #[test]
    fn threshold_monitor_is_enabled_only_after_its_domain_is_programmed() {
        let thresholds =
            Thresholds::new(AlsCounts::from_counts(100), AlsCounts::from_counts(1_000)).unwrap();
        let monitor = ThresholdMonitorConfig::new(
            MeasurementConfig::safe_bright_start(),
            thresholds,
            Persistence::Four,
            PowerSavingConfig::new(true, PowerSavingMode::Mode2),
        );
        let bus = ScriptedI2c::new([
            read_word(0x00, 0x0001),
            write_word(0x00, 0x0001, Ok(())),
            write_word(0x02, 100, Ok(())),
            write_word(0x01, 1_000, Ok(())),
            write_word(0x03, 0x0003, Ok(())),
            write_word(0x00, 0x1022, Ok(())),
        ]);
        let mut sensor = Veml7700::new(bus);
        block_on(sensor.arm_threshold_monitor(monitor)).unwrap();
        sensor.release().done();
    }

    #[test]
    fn probe_reads_little_endian_id() {
        let bus = ScriptedI2c::new([read_word(0x07, 0xC481)]);
        let mut sensor = Veml7700::new(bus);
        let id = block_on(sensor.probe()).unwrap();
        assert_eq!(id.raw(), 0xC481);
        sensor.release().done();
    }

    #[test]
    fn configuration_write_is_low_byte_first() {
        let bus = ScriptedI2c::new([read_word(0x00, 0x0001), write_word(0x00, 0x1001, Ok(()))]);
        let mut sensor = Veml7700::new(bus);
        block_on(sensor.set_measurement_config(MeasurementConfig::safe_bright_start())).unwrap();
        sensor.release().done();
    }

    #[test]
    fn threshold_monitor_blocks_domain_retarget_before_write() {
        let bus = ScriptedI2c::new([read_word(0x00, 0x0002)]);
        let mut sensor = Veml7700::new(bus);
        let result =
            block_on(sensor.set_measurement_config(MeasurementConfig::safe_bright_start()));
        assert_eq!(
            result,
            Err(Error::Configuration(
                ConfigurationError::ThresholdMonitorOwnsDomain
            ))
        );
        sensor.release().done();
    }

    #[test]
    fn probe_preserves_non_address_bus_error() {
        let failure = ScriptError::new(ErrorKind::Bus);
        let bus = ScriptedI2c::new([read_failure(0x07, failure)]);
        let mut sensor = Veml7700::new(bus);
        assert_eq!(block_on(sensor.probe()), Err(ProbeError::Bus(failure)));
        sensor.release().done();
    }

    #[test]
    fn probe_classifies_address_nack_and_wrong_identity() {
        let nack = ScriptError::new(ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address));
        let mut absent = Veml7700::new(ScriptedI2c::new([read_failure(0x07, nack)]));
        assert_eq!(block_on(absent.probe()), Err(ProbeError::NotPresent));
        absent.release().done();

        let mut wrong = Veml7700::new(ScriptedI2c::new([read_word(0x07, 0x0081)]));
        assert_eq!(
            block_on(wrong.probe()),
            Err(ProbeError::WrongDevice { observed: 0x0081 })
        );
        wrong.release().done();
    }

    #[test]
    fn snapshot_reads_provenance_before_sequential_channels() {
        let bus = ScriptedI2c::new([
            read_word(0x00, 0x0001),
            read_word(0x03, 0x0000),
            read_word(0x04, 0x1234),
            read_word(0x05, 0x5678),
        ]);
        let mut sensor = Veml7700::new(bus);
        let snapshot = block_on(sensor.snapshot()).unwrap();
        assert_eq!(snapshot.als, AlsCounts::from_counts(0x1234));
        assert_eq!(snapshot.white, WhiteCounts::from_counts(0x5678));
        assert_eq!(
            snapshot.configuration,
            crate::ConfigurationSnapshot::silicon_reset_default()
        );
        assert_eq!(
            snapshot.coherence,
            crate::MeasurementPairCoherence::SequentialRegisters
        );
        sensor.release().done();
    }

    #[test]
    fn inspect_reads_the_complete_diagnostic_register_set() {
        let bus = ScriptedI2c::new([
            read_word(0x07, 0xC481),
            read_word(0x00, 0x0001),
            read_word(0x03, 0x0000),
            read_word(0x02, 100),
            read_word(0x01, 1_000),
            read_word(0x06, 0x4000),
        ]);
        let mut sensor = Veml7700::new(bus);
        let snapshot = block_on(sensor.inspect()).unwrap();
        assert_eq!(snapshot.id.raw(), 0xC481);
        assert_eq!(snapshot.thresholds.low().counts(), 100);
        assert_eq!(snapshot.thresholds.high().counts(), 1_000);
        assert!(!snapshot.threshold_status.low);
        assert!(snapshot.threshold_status.high);
        sensor.release().done();
    }

    #[test]
    fn strict_decode_errors_preserve_semantic_context() {
        let mut configuration = Veml7700::new(ScriptedI2c::new([read_word(0x00, 0x0004)]));
        assert_eq!(
            block_on(configuration.read_configuration()),
            Err(Error::Configuration(
                ConfigurationError::ConfigurationDecode(crate::ConfigDecodeError::ReservedBits {
                    observed: 4
                })
            ))
        );
        configuration.release().done();

        let mut power = Veml7700::new(ScriptedI2c::new([read_word(0x03, 0x0008)]));
        assert_eq!(
            block_on(power.read_power_saving()),
            Err(Error::Configuration(ConfigurationError::PowerSavingDecode(
                crate::PowerSavingDecodeError::ReservedBits { observed: 8 }
            )))
        );
        power.release().done();

        let mut status = Veml7700::new(ScriptedI2c::new([read_word(0x06, 0x0001)]));
        assert_eq!(
            block_on(status.read_threshold_status()),
            Err(Error::Configuration(
                ConfigurationError::ThresholdStatusDecode(
                    crate::ThresholdStatusDecodeError::ReservedBits { observed: 1 }
                )
            ))
        );
        status.release().done();

        let mut thresholds =
            Veml7700::new(ScriptedI2c::new([read_word(0x02, 2), read_word(0x01, 1)]));
        assert_eq!(
            block_on(thresholds.read_thresholds()),
            Err(Error::Configuration(ConfigurationError::ReversedThresholds))
        );
        thresholds.release().done();
    }

    #[test]
    fn monitor_blocks_power_and_cadence_changes_before_write() {
        let mut power = Veml7700::new(ScriptedI2c::new([read_word(0x00, 0x0002)]));
        assert_eq!(
            block_on(power.set_power_state(PowerState::Shutdown)),
            Err(Error::Configuration(
                ConfigurationError::ThresholdMonitorOwnsDomain
            ))
        );
        power.release().done();

        let mut cadence = Veml7700::new(ScriptedI2c::new([
            read_word(0x00, 0x0002),
            read_word(0x03, 0x0000),
        ]));
        assert_eq!(
            block_on(
                cadence.set_power_saving(PowerSavingConfig::new(true, PowerSavingMode::Mode1,))
            ),
            Err(Error::Configuration(
                ConfigurationError::ThresholdMonitorOwnsDomain
            ))
        );
        cadence.release().done();
    }

    // The sources require `ALS_SD = 1` before any reconfiguration. Each test
    // below starts from an active device, which is the case every other test in
    // this module misses: from shutdown the sequencing writes collapse into
    // no-ops, so a shutdown-only suite passes whether or not the rule is
    // followed. The exact word order is the assertion — a shutdown write that
    // also carried the new domain would satisfy a transaction count but violate
    // the contract.

    #[test]
    fn reconfiguration_from_active_shuts_down_before_changing_the_domain() {
        // 0x0000 active, gain ×1, 100 ms. Target Div8/800 ms is 0x10C0 active.
        let bus = ScriptedI2c::new([
            read_word(0x00, 0x0000),
            // Shutdown carries the *old* domain: bit 0 only.
            write_word(0x00, 0x0001, Ok(())),
            // The new domain lands while shut down.
            write_word(0x00, 0x10C1, Ok(())),
            // Active last.
            write_word(0x00, 0x10C0, Ok(())),
        ]);
        let mut sensor = Veml7700::new(bus);
        block_on(sensor.set_measurement_config(MeasurementConfig::new(
            crate::Gain::Div8,
            crate::IntegrationTime::Ms800,
        )))
        .unwrap();
        sensor.release().done();
    }

    #[test]
    fn reconfiguration_from_shutdown_stays_a_single_write() {
        let bus = ScriptedI2c::new([read_word(0x00, 0x0001), write_word(0x00, 0x10C1, Ok(()))]);
        let mut sensor = Veml7700::new(bus);
        block_on(sensor.set_measurement_config(MeasurementConfig::new(
            crate::Gain::Div8,
            crate::IntegrationTime::Ms800,
        )))
        .unwrap();
        // An already shut-down device needs no shutdown write and must not be
        // woken as a side effect of reconfiguring it.
        sensor.release().done();
    }

    #[test]
    fn cadence_change_from_active_shuts_down_before_writing_power_saving() {
        let bus = ScriptedI2c::new([
            read_word(0x00, 0x0000),
            read_word(0x03, 0x0000),
            write_word(0x00, 0x0001, Ok(())),
            write_word(0x03, 0x0003, Ok(())),
            write_word(0x00, 0x0000, Ok(())),
        ]);
        let mut sensor = Veml7700::new(bus);
        block_on(sensor.set_power_saving(PowerSavingConfig::new(true, PowerSavingMode::Mode2)))
            .unwrap();
        sensor.release().done();
    }

    #[test]
    fn fresh_capture_from_active_enters_shutdown_before_touching_the_domain() {
        let bus = ScriptedI2c::new([
            read_word(0x00, 0x0000),
            read_word(0x03, 0x0000),
            // Shutdown in the original domain, before power saving or gain move.
            write_word(0x00, 0x0001, Ok(())),
            write_word(0x03, 0x0000, Ok(())),
            write_word(0x00, 0x1001, Ok(())),
            write_word(0x00, 0x1000, Ok(())),
            write_word(0x00, 0x1001, Ok(())),
            read_word(0x04, 0x1234),
            read_word(0x05, 0x5678),
            write_word(0x03, 0x0000, Ok(())),
            // Restoration returns the device to the active state it started in.
            write_word(0x00, 0x0000, Ok(())),
        ]);
        let mut delay = RecordingDelay { elapsed_ns: 0 };
        let mut sensor = Veml7700::new(bus);
        let sample =
            block_on(sensor.measure_once(&mut delay, MeasurementConfig::safe_bright_start()))
                .unwrap();
        assert_eq!(sample.als, AlsCounts::from_counts(0x1234));
        assert_eq!(sample.white, WhiteCounts::from_counts(0x5678));
        sensor.release().done();
    }

    #[test]
    fn arming_from_active_disables_the_monitor_and_shuts_down_together() {
        let thresholds =
            Thresholds::new(AlsCounts::from_counts(100), AlsCounts::from_counts(1_000)).unwrap();
        let monitor = ThresholdMonitorConfig::new(
            MeasurementConfig::safe_bright_start(),
            thresholds,
            Persistence::Four,
            PowerSavingConfig::new(true, PowerSavingMode::Mode2),
        );
        let bus = ScriptedI2c::new([
            read_word(0x00, 0x0000),
            // Monitor-disable and shutdown in one write, still the old domain.
            write_word(0x00, 0x0001, Ok(())),
            write_word(0x02, 100, Ok(())),
            write_word(0x01, 1_000, Ok(())),
            write_word(0x03, 0x0003, Ok(())),
            // The monitored domain is installed and activated last.
            write_word(0x00, 0x1022, Ok(())),
        ]);
        let mut sensor = Veml7700::new(bus);
        block_on(sensor.arm_threshold_monitor(monitor)).unwrap();
        sensor.release().done();
    }

    #[test]
    fn re_arming_an_active_monitor_shuts_down_before_disabling_it() {
        let thresholds =
            Thresholds::new(AlsCounts::from_counts(100), AlsCounts::from_counts(1_000)).unwrap();
        let monitor = ThresholdMonitorConfig::new(
            MeasurementConfig::safe_bright_start(),
            thresholds,
            Persistence::Four,
            PowerSavingConfig::new(true, PowerSavingMode::Mode2),
        );
        // 0x0002 is active with the monitor enabled. The shutdown and monitor
        // bits cannot move together, so this is the one starting state needing
        // an extra write.
        let bus = ScriptedI2c::new([
            read_word(0x00, 0x0002),
            // Shutdown with the monitored domain intact: only bit 0 moves.
            write_word(0x00, 0x0003, Ok(())),
            // Monitor disabled while shut down: only bit 1 moves.
            write_word(0x00, 0x0001, Ok(())),
            write_word(0x02, 100, Ok(())),
            write_word(0x01, 1_000, Ok(())),
            write_word(0x03, 0x0003, Ok(())),
            write_word(0x00, 0x1022, Ok(())),
        ]);
        let mut sensor = Veml7700::new(bus);
        block_on(sensor.arm_threshold_monitor(monitor)).unwrap();
        sensor.release().done();
    }

    #[test]
    fn setting_an_unchanged_value_writes_nothing() {
        // Reset default is gain ×1 / 100 ms. Requesting it back must not cycle
        // power: an enabled monitor would lose its active domain for a call
        // that changes no field.
        let mut measurement = Veml7700::new(ScriptedI2c::new([read_word(0x00, 0x0002)]));
        block_on(measurement.set_measurement_config(MeasurementConfig::silicon_reset_default()))
            .unwrap();
        measurement.release().done();

        let mut cadence = Veml7700::new(ScriptedI2c::new([
            read_word(0x00, 0x0002),
            read_word(0x03, 0x0000),
        ]));
        block_on(cadence.set_power_saving(PowerSavingConfig::disabled())).unwrap();
        cadence.release().done();
    }

    #[test]
    fn a_failed_shutdown_reports_without_attempting_restoration() {
        // Nothing has been mutated when the first write fails, and the device
        // may still be active. Restoring would write power saving to an active
        // device — the exact write this sequence exists to avoid.
        let bus = ScriptedI2c::new([
            read_word(0x00, 0x0000),
            read_word(0x03, 0x0000),
            write_word(0x00, 0x0001, Err(ScriptError::new(ErrorKind::Bus))),
        ]);
        let mut delay = RecordingDelay { elapsed_ns: 0 };
        let mut sensor = Veml7700::new(bus);
        let result =
            block_on(sensor.measure_once(&mut delay, MeasurementConfig::safe_bright_start()));
        assert!(matches!(
            result,
            Err(MeasureOnceError::Operation {
                stage: MeasureStage::EnterShutdown,
                ..
            })
        ));
        sensor.release().done();
    }

    #[test]
    fn every_fresh_capture_stage_failure_is_restored_and_identified() {
        let failure = ScriptError::new(ErrorKind::Bus);
        let stages = [
            (
                MeasureStage::DisablePowerSaving,
                BusContext::WritePowerSaving,
            ),
            (
                MeasureStage::PrepareMeasurement,
                BusContext::WriteConfiguration,
            ),
            (
                MeasureStage::ActivateMeasurement,
                BusContext::WriteConfiguration,
            ),
            (MeasureStage::FreezeResult, BusContext::WriteConfiguration),
            (MeasureStage::ReadAls, BusContext::ReadAls),
            (MeasureStage::ReadWhite, BusContext::ReadWhite),
        ];

        for (failed_index, (stage, context)) in stages.into_iter().enumerate() {
            let mut expectations = vec![read_word(0x00, 0x0001), read_word(0x03, 0x0000)];
            for index in 0..=failed_index {
                let result = (index != failed_index).then_some(()).ok_or(failure);
                expectations.push(match index {
                    0 => write_word(0x03, 0x0000, result),
                    1 | 3 => write_word(0x00, 0x1001, result),
                    2 => write_word(0x00, 0x1000, result),
                    4 if result.is_err() => read_failure(0x04, failure),
                    4 => read_word(0x04, 0x1234),
                    5 => read_failure(0x05, failure),
                    _ => unreachable!(),
                });
            }
            expectations.push(write_word(0x03, 0x0000, Ok(())));
            expectations.push(write_word(0x00, 0x0001, Ok(())));

            let mut delay = RecordingDelay { elapsed_ns: 0 };
            let mut sensor = Veml7700::new(ScriptedI2c::new(expectations));
            assert_eq!(
                block_on(sensor.measure_once(&mut delay, MeasurementConfig::safe_bright_start(),)),
                Err(MeasureOnceError::Operation {
                    stage,
                    source: Error::Bus {
                        operation: Operation::MeasureOnce,
                        context,
                        source: failure,
                    },
                })
            );
            assert_eq!(delay.elapsed_ns != 0, failed_index >= 3);
            sensor.release().done();
        }
    }

    #[test]
    fn observation_failures_are_identified_without_cleanup_writes() {
        let failure = ScriptError::new(ErrorKind::Bus);
        let mut delay = RecordingDelay { elapsed_ns: 0 };
        let mut configuration = Veml7700::new(ScriptedI2c::new([read_failure(0x00, failure)]));
        assert_eq!(
            block_on(
                configuration.measure_once(&mut delay, MeasurementConfig::safe_bright_start(),)
            ),
            Err(MeasureOnceError::Operation {
                stage: MeasureStage::ObserveConfiguration,
                source: Error::Bus {
                    operation: Operation::MeasureOnce,
                    context: BusContext::ReadConfiguration,
                    source: failure,
                },
            })
        );
        configuration.release().done();

        let mut power = Veml7700::new(ScriptedI2c::new([
            read_word(0x00, 0x0001),
            read_failure(0x03, failure),
        ]));
        assert_eq!(
            block_on(power.measure_once(&mut delay, MeasurementConfig::safe_bright_start())),
            Err(MeasureOnceError::Operation {
                stage: MeasureStage::ObservePowerSaving,
                source: Error::Bus {
                    operation: Operation::MeasureOnce,
                    context: BusContext::ReadPowerSaving,
                    source: failure,
                },
            })
        );
        power.release().done();
    }

    #[test]
    fn cleanup_failure_reports_primary_and_recovery_errors() {
        let primary = ScriptError::new(ErrorKind::Bus);
        let recovery = ScriptError::new(ErrorKind::ArbitrationLoss);
        let bus = ScriptedI2c::new([
            read_word(0x00, 0x0001),
            read_word(0x03, 0x0000),
            write_word(0x03, 0x0000, Err(primary)),
            write_word(0x03, 0x0000, Err(recovery)),
        ]);
        let mut delay = RecordingDelay { elapsed_ns: 0 };
        let mut sensor = Veml7700::new(bus);
        assert_eq!(
            block_on(sensor.measure_once(&mut delay, MeasurementConfig::safe_bright_start(),)),
            Err(MeasureOnceError::RecoveryFailed {
                failed_stage: MeasureStage::DisablePowerSaving,
                source: Error::Bus {
                    operation: Operation::MeasureOnce,
                    context: BusContext::WritePowerSaving,
                    source: primary,
                },
                recovery_stage: MeasureStage::RestorePowerSaving,
                recovery_source: Error::Bus {
                    operation: Operation::MeasureOnce,
                    context: BusContext::WritePowerSaving,
                    source: recovery,
                },
            })
        );
        sensor.release().done();
    }

    #[test]
    fn post_capture_restoration_failure_preserves_the_sample() {
        let failure = ScriptError::new(ErrorKind::Bus);
        let bus = ScriptedI2c::new([
            read_word(0x00, 0x0001),
            read_word(0x03, 0x0000),
            write_word(0x03, 0x0000, Ok(())),
            write_word(0x00, 0x1001, Ok(())),
            write_word(0x00, 0x1000, Ok(())),
            write_word(0x00, 0x1001, Ok(())),
            read_word(0x04, 0x1234),
            read_word(0x05, 0x5678),
            write_word(0x03, 0x0000, Err(failure)),
        ]);
        let mut delay = RecordingDelay { elapsed_ns: 0 };
        let mut sensor = Veml7700::new(bus);
        match block_on(sensor.measure_once(&mut delay, MeasurementConfig::safe_bright_start())) {
            Err(MeasureOnceError::RestoreFailed {
                sample,
                stage,
                source,
            }) => {
                assert_eq!(sample.als, AlsCounts::from_counts(0x1234));
                assert_eq!(sample.white, WhiteCounts::from_counts(0x5678));
                assert_eq!(stage, MeasureStage::RestorePowerSaving);
                assert_eq!(
                    source,
                    Error::Bus {
                        operation: Operation::MeasureOnce,
                        context: BusContext::WritePowerSaving,
                        source: failure,
                    }
                );
            }
            other => panic!("unexpected result: {other:?}"),
        }
        sensor.release().done();
    }

    #[test]
    fn configuration_restoration_failures_preserve_their_stage_and_sample_state() {
        let primary = ScriptError::new(ErrorKind::Bus);
        let recovery = ScriptError::new(ErrorKind::ArbitrationLoss);
        let pre_capture = ScriptedI2c::new([
            read_word(0x00, 0x0001),
            read_word(0x03, 0x0000),
            write_word(0x03, 0x0000, Err(primary)),
            write_word(0x03, 0x0000, Ok(())),
            write_word(0x00, 0x0001, Err(recovery)),
        ]);
        let mut delay = RecordingDelay { elapsed_ns: 0 };
        let mut sensor = Veml7700::new(pre_capture);
        assert_eq!(
            block_on(sensor.measure_once(&mut delay, MeasurementConfig::safe_bright_start(),)),
            Err(MeasureOnceError::RecoveryFailed {
                failed_stage: MeasureStage::DisablePowerSaving,
                source: Error::Bus {
                    operation: Operation::MeasureOnce,
                    context: BusContext::WritePowerSaving,
                    source: primary,
                },
                recovery_stage: MeasureStage::RestoreConfiguration,
                recovery_source: Error::Bus {
                    operation: Operation::MeasureOnce,
                    context: BusContext::WriteConfiguration,
                    source: recovery,
                },
            })
        );
        sensor.release().done();

        let post_capture = ScriptedI2c::new([
            read_word(0x00, 0x0001),
            read_word(0x03, 0x0000),
            write_word(0x03, 0x0000, Ok(())),
            write_word(0x00, 0x1001, Ok(())),
            write_word(0x00, 0x1000, Ok(())),
            write_word(0x00, 0x1001, Ok(())),
            read_word(0x04, 0x1234),
            read_word(0x05, 0x5678),
            write_word(0x03, 0x0000, Ok(())),
            write_word(0x00, 0x0001, Err(recovery)),
        ]);
        let mut sensor = Veml7700::new(post_capture);
        match block_on(sensor.measure_once(&mut delay, MeasurementConfig::safe_bright_start())) {
            Err(MeasureOnceError::RestoreFailed {
                sample,
                stage,
                source,
            }) => {
                assert_eq!(sample.als, AlsCounts::from_counts(0x1234));
                assert_eq!(sample.white, WhiteCounts::from_counts(0x5678));
                assert_eq!(stage, MeasureStage::RestoreConfiguration);
                assert_eq!(
                    source,
                    Error::Bus {
                        operation: Operation::MeasureOnce,
                        context: BusContext::WriteConfiguration,
                        source: recovery,
                    }
                );
            }
            other => panic!("unexpected result: {other:?}"),
        }
        sensor.release().done();
    }

    #[test]
    fn every_threshold_programming_stage_failure_is_identified() {
        let failure = ScriptError::new(ErrorKind::Bus);
        let thresholds =
            Thresholds::new(AlsCounts::from_counts(100), AlsCounts::from_counts(1_000)).unwrap();
        let monitor = ThresholdMonitorConfig::new(
            MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms100),
            thresholds,
            Persistence::Four,
            PowerSavingConfig::new(true, PowerSavingMode::Mode2),
        );
        let stages = [
            (
                ThresholdMonitorStage::ObserveConfiguration,
                BusContext::ReadConfiguration,
            ),
            (
                ThresholdMonitorStage::DisableMonitor,
                BusContext::WriteConfiguration,
            ),
            (
                ThresholdMonitorStage::WriteLowThreshold,
                BusContext::WriteLowThreshold,
            ),
            (
                ThresholdMonitorStage::WriteHighThreshold,
                BusContext::WriteHighThreshold,
            ),
            (
                ThresholdMonitorStage::ApplyPowerSaving,
                BusContext::WritePowerSaving,
            ),
            (
                ThresholdMonitorStage::EnableMonitor,
                BusContext::WriteConfiguration,
            ),
        ];
        for (failed_index, (stage, context)) in stages.into_iter().enumerate() {
            let mut expectations = vec![];
            if failed_index == 0 {
                expectations.push(read_failure(0x00, failure));
            } else {
                expectations.push(read_word(0x00, 0x0001));
                for index in 1..=failed_index {
                    let result = (index != failed_index).then_some(()).ok_or(failure);
                    expectations.push(match index {
                        1 => write_word(0x00, 0x0001, result),
                        2 => write_word(0x02, 100, result),
                        3 => write_word(0x01, 1_000, result),
                        4 => write_word(0x03, 0x0003, result),
                        5 => write_word(0x00, 0x1022, result),
                        _ => unreachable!(),
                    });
                }
            }
            let mut sensor = Veml7700::new(ScriptedI2c::new(expectations));
            assert_eq!(
                block_on(sensor.arm_threshold_monitor(monitor)),
                Err(ThresholdMonitorError {
                    stage,
                    source: Error::Bus {
                        operation: Operation::ThresholdMonitor,
                        context,
                        source: failure,
                    },
                })
            );
            sensor.release().done();
        }
    }
}
