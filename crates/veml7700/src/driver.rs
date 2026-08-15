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
    AlsCounts, DeviceSnapshot, MeasurementCapture, MeasurementPairCoherence, SnapshotMeasurement,
    WhiteCounts,
};
use crate::power::{PowerSavingConfig, PowerSavingSnapshot, decode_power_saving};
use crate::register::Register;
use crate::threshold::{ThresholdMonitorConfig, ThresholdStatus, Thresholds};
use crate::timing::MeasurementTiming;

/// Driver address selection (`S-05`).
pub const I2C_ADDRESS: u8 = 0x10;

/// Async VEML7700 driver owning one I²C resource.
///
/// # Write and cancellation uncertainty
///
/// A failed I²C write establishes only that the operation did not complete. The
/// write may not have arrived, or it may have taken effect before an
/// acknowledgement was lost. Dropping a future during a write has the same
/// uncertainty without returning an error. Read device state back when commit
/// status matters; this driver never claims rollback it cannot establish.
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

    /// Read one raw observation of the polled threshold flags.
    ///
    /// See [`ThresholdStatus`] for the clearing, history, and qualification
    /// limits carried by this result.
    pub async fn read_threshold_status(&mut self) -> Result<ThresholdStatus, Error<I2C::Error>> {
        self.read_threshold_status_for(Operation::Inspect).await
    }

    /// Read and validate the ordered threshold-register pair.
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
    /// In reaction to `S-19`, an active device is shut down first, reconfigured
    /// while shut down, and returned to active last — three writes rather than
    /// one. A device that is already shut down takes the single-write path and
    /// stays shut down.
    ///
    /// # State after a failure
    ///
    /// Because shutdown comes first, a failure part way through can leave an
    /// originally active device shut down, with the measurement domain either
    /// old or new. Read the configuration back to establish which. This is the
    /// cost of following the shutdown-first sequence `S-19`.
    ///
    /// A returned error also does **not** establish that the failing write was
    /// rejected; see the type-level write uncertainty on [`Veml7700`].
    ///
    /// Dropping this future has the same effect as a failure at that point,
    /// without an error to inspect.
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
    ///
    /// This is a single write and not a reconfiguration, so it needs no shutdown
    /// sequencing and has no partial state.
    ///
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
    /// establish which. As everywhere, an error does not establish that the
    /// failing write was rejected — see [`set_power_state`](Self::set_power_state).
    ///
    /// # This future is not cancellation-safe
    ///
    /// Two reads then up to three writes, and dropping at any of them leaves the
    /// device where that boundary reached:
    ///
    /// | Dropped at | Device is left |
    /// | --- | --- |
    /// | Either read | Unchanged; nothing was written |
    /// | Entering shutdown | Active or shut down, cadence unchanged |
    /// | Writing the cadence | Shut down; cadence old or new |
    /// | Returning to active | Shut down or active, cadence new |
    ///
    /// The middle rows leave an originally active device asleep. Recover by
    /// reading [`read_configuration`](Self::read_configuration) and
    /// [`read_power_saving`](Self::read_power_saving), then reinstating what you
    /// want; both are idempotent, so repeating the call is safe.
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

    /// Capture one measurement using the default conditional timing policy.
    ///
    /// The wait is 2.5 ms plus 130% of the selected integration time (`S-23`,
    /// `S-24`, `S-55`) plus a 1 ms driver-policy margin. The corresponding device
    /// bound remains undefined. If the real spread exceeds it, this can return a
    /// value from the previous conversion, indistinguishable from a new one.
    pub async fn measure_once<D>(
        &mut self,
        delay: &mut D,
        measurement: MeasurementConfig,
    ) -> Result<MeasurementCapture, MeasureOnceError<I2C::Error>>
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

    /// Capture one measurement using an explicit policy-compliant wait.
    ///
    /// [`MeasurementTiming`] cannot represent a wait shorter than the
    /// conservative minimum for its selected integration time. That minimum is
    /// the driver's conditional reaction to `S-23`, `S-24`, and `S-55` — see
    /// [`measure_once`] and
    /// [`INTEGRATION_TOLERANCE_PERCENT`](crate::INTEGRATION_TOLERANCE_PERCENT).
    ///
    /// [`measure_once`]: Self::measure_once
    ///
    /// The operation disables power-saving cadence, installs the requested
    /// measurement domain while shut down, creates a known shutdown-to-active
    /// wake edge, waits, enters shutdown again to freeze data, reads ALS and
    /// white, then restores the original configuration and power-saving register
    /// — **when polled to completion.**
    ///
    /// # This future is not cancellation-safe
    ///
    /// Dropping it does not undo what it has already done. The driver is not an
    /// executor and cannot run cleanup during a drop, so restoration only
    /// happens on paths that return.
    ///
    /// Every `await` is a point where a caller may drop, and the device is left
    /// in the state that boundary reached:
    ///
    /// | Dropped at | Device is left |
    /// | --- | --- |
    /// | Observing configuration or power saving | Unchanged; nothing was written |
    /// | Entering shutdown | Active or shut down, in the original domain |
    /// | Disabling power saving | Shut down; cadence old or new |
    /// | Installing the domain | Shut down, in the original or requested domain, cadence disabled |
    /// | Activating | Shut down or **active and converting**, in the requested domain |
    /// | **The measurement delay** | **Active and converting**, requested domain, cadence disabled |
    /// | Freezing the result | Active or shut down, requested domain |
    /// | Reading ALS or white | Shut down, requested domain; the sample is lost |
    /// | Restoring power saving | Shut down; cadence old or new |
    /// | Restoring configuration | Shut down or restored |
    ///
    /// The delay row is the one that matters in practice: it is by far the
    /// longest suspension, so a timeout or `select!` is most likely to land
    /// there, and it leaves the sensor **awake and drawing current** in a domain
    /// the caller did not ask to persist.
    ///
    /// Each row spans two possibilities wherever a write was in flight, because
    /// an interrupted I²C write may or may not have reached the device. That is
    /// a property of the bus, not a gap in this description.
    ///
    /// # Recovering after a drop
    ///
    /// Do not infer the state — read it. This procedure is deterministic and
    /// uses only public operations:
    ///
    /// 1. [`read_configuration`](Self::read_configuration) and
    ///    [`read_power_saving`](Self::read_power_saving) to observe what is
    ///    actually installed.
    /// 2. [`set_power_state`](Self::set_power_state) with
    ///    [`PowerState::Shutdown`] to stop conversion and current draw.
    /// 3. [`set_power_saving`](Self::set_power_saving) and
    ///    [`set_measurement_config`](Self::set_measurement_config) to reinstate
    ///    the domain you want.
    ///
    /// Step 2 first: it is the only step that bounds how long an abandoned
    /// conversion keeps running.
    ///
    /// A caller that cannot tolerate this should not race this future against a
    /// timeout. Bound the operation by choosing a shorter integration time
    /// instead, which shortens the delay rather than abandoning it.
    pub async fn measure_once_with_timing<D>(
        &mut self,
        delay: &mut D,
        measurement: MeasurementConfig,
        timing: MeasurementTiming,
    ) -> Result<MeasurementCapture, MeasureOnceError<I2C::Error>>
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
        let sample = MeasurementCapture {
            als,
            white,
            configuration: measurement,
            nominal_illuminance: als.nominal_micro_lux(measurement),
            requested_wait_us: timing.total_us(),
            coherence: MeasurementPairCoherence::FrozenAfterRequestedWait,
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
    /// The driver reaches shutdown before changing the domain, confirms the
    /// monitor disabled, writes thresholds and cadence, then installs the final
    /// domain active and enabled. Re-arming an active enabled monitor needs an
    /// initial write that enters shutdown while preserving the enabled bit; only
    /// the following confirmed write establishes disabled-and-shut-down state.
    ///
    /// [`ThresholdMonitorError`] identifies the actual branch, last confirmed
    /// write, and failing stage. A failed or cancelled write may have committed;
    /// read configuration, thresholds, and power saving back before recovery.
    /// Repeating this complete operation is idempotent in effect.
    ///
    /// This operation performs no explicit status-clearing action and gives no
    /// status-history or assertion-time guarantee. See [`ThresholdStatus`].
    pub async fn arm_threshold_monitor(
        &mut self,
        monitor: ThresholdMonitorConfig,
    ) -> Result<(), ThresholdMonitorError<I2C::Error>> {
        let current = self
            .read_configuration_for(Operation::ThresholdMonitor)
            .await
            .map_err(|source| ThresholdMonitorError {
                stage: ThresholdMonitorStage::ObserveConfiguration,
                confirmed: None,
                source,
            })?;

        // A single write may move the shutdown bit or the monitor bit, not both:
        // the device accepts either as a transition, but the two together are a
        // reconfiguration, which the `S-19` driver reaction performs only in
        // shutdown. That
        // only bites when re-arming an enabled monitor on an active device —
        // every other starting state needs one write.
        // Tracks what definitely reached the device. Each write advances it only
        // after returning success, so the failing stage is never counted as
        // confirmed -- its commit status is exactly what nobody can establish.
        let mut confirmed: Option<ThresholdMonitorStage> = None;
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
                confirmed: None,
                source,
            })?;
            confirmed = Some(ThresholdMonitorStage::EnterShutdown);
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
                confirmed,
                source,
            })?;
        confirmed = Some(ThresholdMonitorStage::DisableMonitor);
        self.write_word(
            Register::LowThreshold,
            monitor.thresholds.low().counts(),
            Operation::ThresholdMonitor,
            BusContext::WriteLowThreshold,
        )
        .await
        .map_err(|source| ThresholdMonitorError {
            stage: ThresholdMonitorStage::WriteLowThreshold,
            confirmed,
            source,
        })?;
        confirmed = Some(ThresholdMonitorStage::WriteLowThreshold);
        self.write_word(
            Register::HighThreshold,
            monitor.thresholds.high().counts(),
            Operation::ThresholdMonitor,
            BusContext::WriteHighThreshold,
        )
        .await
        .map_err(|source| ThresholdMonitorError {
            stage: ThresholdMonitorStage::WriteHighThreshold,
            confirmed,
            source,
        })?;
        confirmed = Some(ThresholdMonitorStage::WriteHighThreshold);
        self.write_power_saving_for(monitor.power_saving, Operation::ThresholdMonitor)
            .await
            .map_err(|source| ThresholdMonitorError {
                stage: ThresholdMonitorStage::ApplyPowerSaving,
                confirmed,
                source,
            })?;
        confirmed = Some(ThresholdMonitorStage::ApplyPowerSaving);

        let enabled = disabled
            .with_measurement(monitor.measurement)
            .with_persistence(monitor.persistence)
            .with_power_state(PowerState::Active)
            .with_monitor(ThresholdMonitorState::Enabled);
        self.write_configuration_for(enabled, Operation::ThresholdMonitor)
            .await
            .map_err(|source| ThresholdMonitorError {
                stage: ThresholdMonitorStage::EnableMonitor,
                confirmed,
                source,
            })
    }

    /// Disable threshold monitoring while preserving all other configuration fields.
    ///
    /// This clears the monitor bit only. It does **not** restore whatever power
    /// state preceded arming: a device armed from shutdown stays active after
    /// disabling. Follow with [`set_power_state`](Self::set_power_state) if you
    /// want it asleep.
    ///
    /// It performs no explicit threshold-status clearing action. See
    /// [`ThresholdStatus`] for the resulting history limits.
    ///
    /// # This future is not cancellation-safe
    ///
    /// One read then one write. Dropping at the read changes nothing; dropping
    /// at the write leaves the monitor either enabled or disabled, and a
    /// returned error does not distinguish those either — see the type-level
    /// write uncertainty on [`Veml7700`].
    ///
    /// Recover by reading [`read_configuration`](Self::read_configuration) and
    /// repeating the call, which is idempotent.
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
        // Driver reaction to `S-08`.
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
        // Driver reaction to `S-08`.
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
    //! Driver tests, split by responsibility.
    //!
    //! These stay `#[cfg(test)]` submodules rather than becoming integration
    //! tests because they exercise private sequencing. Promoting them would
    //! either lose that access or force the internals public to keep it.
    //!
    //! Shared imports and the delay stub live here; the exact-transaction
    //! builders live in `crate::testing::scripted_i2c`, so the wire format has
    //! one definition.

    use alloc::vec;
    use embedded_hal_async::i2c::{ErrorKind, NoAcknowledgeSource};
    use futures::executor::block_on;

    use super::Veml7700;
    use crate::testing::cancellation::{CancellableDelay, PendingAt, poll_once_then_drop};
    use crate::testing::scripted_i2c::{
        Expectation, ScriptError, ScriptedI2c, read_failure, read_word, write_word,
    };
    use crate::{
        AlsCounts, BusContext, ConfigurationError, Error, Gain, IntegrationTime, MeasureOnceError,
        MeasureStage, MeasurementConfig, Operation, Persistence, PowerSavingConfig,
        PowerSavingMode, PowerState, ProbeError, ThresholdMonitorConfig, ThresholdMonitorError,
        ThresholdMonitorStage, Thresholds, WhiteCounts,
    };

    /// Complete `measure_once` script from a shut-down device, in order.
    fn fresh_capture_script() -> [Expectation; 10] {
        [
            read_word(0x00, 0x0001),
            read_word(0x03, 0x0000),
            write_word(0x03, 0x0000, Ok(())),
            write_word(0x00, 0x1001, Ok(())),
            write_word(0x00, 0x1000, Ok(())),
            // the measurement delay sits here
            write_word(0x00, 0x1001, Ok(())),
            read_word(0x04, 0x1234),
            read_word(0x05, 0x5678),
            write_word(0x03, 0x0000, Ok(())),
            write_word(0x00, 0x0001, Ok(())),
        ]
    }

    mod cancellation;
    mod configuration;
    mod fresh_measurement;
    mod observation;
    mod probe;
    mod threshold;
}
