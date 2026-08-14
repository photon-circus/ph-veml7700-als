# Public API contract

Target v0.1 public surface. Additions require this document and `DECISIONS.md` to
change first.

Crate policy: `#![no_std]`, `#![forbid(unsafe_code)]`,
`#![deny(missing_docs)]`, async-first `embedded-hal-async`.

Distribution policy: the Incubating candidate is
`0.1.0-incubating.1`. It can be built and inspected locally, but its Cargo
manifest retains `publish = false`. Repository visibility and registry
publication require separate recorded maintainer decisions.

## Constants and facade

```rust
pub const I2C_ADDRESS: u8 = 0x10;
pub const WAKE_UP_DELAY_US: u32 = 2_500;
pub const INTEGRATION_TOLERANCE_PERCENT: u32 = 30;
pub const MEASUREMENT_MARGIN_US: u32 = 1_000;

pub struct Veml7700<I2C> { /* i2c only */ }

impl<I2C> Veml7700<I2C> {
    pub const fn new(i2c: I2C) -> Self;
    pub fn release(self) -> I2C;
}
```

## Value constructors and accessors

The contract includes these public functions:

```text
DeviceId::from_raw, raw, device_code, address_option_code, is_supported
IntegrationTime::milliseconds
Persistence::count
MeasurementConfig::new, silicon_reset_default, safe_bright_start, gain, integration_time
ConfigurationSnapshot::silicon_reset_default
PowerSavingMode::nominal_refresh_time_ms
PowerSavingConfig::new, disabled
MicroLux::from_micro_lux, as_micro_lux, whole_lux_floor, milli_lux_rounded
NominalScale::for_config, micro_lux_per_count, scale_counts
AlsCounts::from_counts, counts, is_saturated, nominal_micro_lux
WhiteCounts::from_counts, counts
Thresholds::new
ThresholdMonitorConfig::new
MeasurementTiming::conservative, with_additional_margin_us,
integration_time, wake_up_us, integration_us, margin_us, total_us
```

Public enums/structs are re-exported from crate root: `Gain`, `IntegrationTime`,
`Persistence`, `PowerState`, `ThresholdMonitorState`, `MeasurementConfig`,
`ConfigurationSnapshot`, `PowerSavingMode`, `PowerSavingConfig`,
`PowerSavingSnapshot`, `DeviceId`, `AlsCounts`, `WhiteCounts`, `MicroLux`,
`NominalScale`, `SnapshotMeasurement`, `FreshMeasurement`,
`MeasurementPairCoherence`, `Thresholds`, `ThresholdStatus`,
`ThresholdMonitorConfig`, `ThresholdStatusDecodeError`, `DeviceSnapshot`,
`MeasurementTiming`, and the error
and stage types.

## Driver operations

```rust
impl<I2C: embedded_hal_async::i2c::I2c> Veml7700<I2C> {
    pub async fn probe(&mut self) -> Result<DeviceId, ProbeError<I2C::Error>>;
    pub async fn read_device_id(&mut self) -> Result<DeviceId, Error<I2C::Error>>;
    pub async fn read_configuration(&mut self)
        -> Result<ConfigurationSnapshot, Error<I2C::Error>>;
    pub async fn read_power_saving(&mut self)
        -> Result<PowerSavingSnapshot, Error<I2C::Error>>;
    pub async fn read_als_snapshot(&mut self)
        -> Result<AlsCounts, Error<I2C::Error>>;
    pub async fn read_white_snapshot(&mut self)
        -> Result<WhiteCounts, Error<I2C::Error>>;
    pub async fn read_threshold_status(&mut self)
        -> Result<ThresholdStatus, Error<I2C::Error>>;
    pub async fn read_thresholds(&mut self)
        -> Result<Thresholds, Error<I2C::Error>>;
    pub async fn inspect(&mut self)
        -> Result<DeviceSnapshot, Error<I2C::Error>>;
    pub async fn snapshot(&mut self)
        -> Result<SnapshotMeasurement, Error<I2C::Error>>;

    pub async fn set_measurement_config(&mut self, config: MeasurementConfig)
        -> Result<(), Error<I2C::Error>>;
    pub async fn set_power_state(&mut self, state: PowerState)
        -> Result<(), Error<I2C::Error>>;
    pub async fn set_power_saving(&mut self, config: PowerSavingConfig)
        -> Result<(), Error<I2C::Error>>;

    pub async fn measure_once<D: embedded_hal_async::delay::DelayNs>(
        &mut self,
        delay: &mut D,
        config: MeasurementConfig,
    ) -> Result<FreshMeasurement, MeasureOnceError<I2C::Error>>;

    pub async fn measure_once_with_timing<D: embedded_hal_async::delay::DelayNs>(
        &mut self,
        delay: &mut D,
        config: MeasurementConfig,
        timing: MeasurementTiming,
    ) -> Result<FreshMeasurement, MeasureOnceError<I2C::Error>>;

    pub async fn arm_threshold_monitor(&mut self, config: ThresholdMonitorConfig)
        -> Result<(), ThresholdMonitorError<I2C::Error>>;
    pub async fn disable_threshold_monitor(&mut self)
        -> Result<(), Error<I2C::Error>>;
}
```

## Semantic contracts

- `probe` validates exact ID `0xC481`; it does not claim package or calibration.
- `snapshot` never claims freshness or atomic ALS/white pairing.
- `measure_once` creates an explicit shutdown-to-active wake edge and restores
  the prior device state or returns explicit uncertainty.
- `MeasurementTiming` cannot represent a shorter-than-conservative wait and is
  rejected when derived for a different integration-time selection.
- monitor configuration is disable-first, enable-last.
- no operation exposes a raw register pointer or owns an interrupt GPIO.
- `MicroLux` values are nominal, not calibrated.
