//! Driver-versus-model tests for the declared behavioral slice.
//!
//! Adapters here are test glue. They are not the model's behavioral API and
//! must not translate model limitations into device NACKs.

use core::cell::RefCell;
use std::rc::Rc;

use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::i2c::{
    Error as I2cError, ErrorKind, ErrorType, I2c, NoAcknowledgeSource as HalNack, Operation,
    SevenBitAddress,
};
use futures::executor::block_on;
use ph_veml7700_als::{
    AlsCounts, ConfigurationSnapshot, Gain, IntegrationTime, MeasurementConfig, Persistence,
    PowerSavingConfig, PowerSavingMode, PowerState, ThresholdMonitorConfig, ThresholdMonitorState,
    Thresholds, Veml7700, WhiteCounts,
};
use ph_veml7700_als_model::{
    NoAcknowledgeSource, RelativeDuration, TransportError, Unsupported, Veml7700Model,
};

#[derive(Clone)]
struct SharedModel(Rc<RefCell<Veml7700Model>>);

struct ModelI2c(SharedModel);
struct ModelDelay(SharedModel);

impl ModelDelay {
    fn set_raw_sample(&self, als: u16, white: u16) {
        self.0.0.borrow_mut().set_raw_sample(als, white);
    }

    fn set_white_phase_offset(&self, offset: RelativeDuration) {
        self.0.0.borrow_mut().set_white_phase_offset(offset);
    }

    fn advance(&self, elapsed: RelativeDuration) {
        self.0.0.borrow_mut().advance(elapsed);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ModelBusError {
    NoAcknowledge(HalNack),
    Unsupported(Unsupported),
}

impl I2cError for ModelBusError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::NoAcknowledge(source) => ErrorKind::NoAcknowledge(*source),
            Self::Unsupported(_) => ErrorKind::Other,
        }
    }
}

fn map_error(error: TransportError) -> ModelBusError {
    match error {
        TransportError::NoAcknowledge {
            source: NoAcknowledgeSource::Address,
        } => ModelBusError::NoAcknowledge(HalNack::Address),
        TransportError::Unsupported(reason) => ModelBusError::Unsupported(reason),
    }
}

impl ErrorType for ModelI2c {
    type Error = ModelBusError;
}

impl I2c<SevenBitAddress> for ModelI2c {
    async fn read(
        &mut self,
        _address: SevenBitAddress,
        _read: &mut [u8],
    ) -> Result<(), Self::Error> {
        Err(ModelBusError::Unsupported(Unsupported::TransactionShape))
    }

    async fn write(&mut self, address: SevenBitAddress, write: &[u8]) -> Result<(), Self::Error> {
        self.0
            .0
            .borrow_mut()
            .write(address, write)
            .map_err(map_error)
    }

    async fn write_read(
        &mut self,
        address: SevenBitAddress,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.0
            .0
            .borrow_mut()
            .write_read(address, write, read)
            .map_err(map_error)
    }

    async fn transaction(
        &mut self,
        _address: SevenBitAddress,
        _operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        Err(ModelBusError::Unsupported(Unsupported::TransactionShape))
    }
}

impl DelayNs for ModelDelay {
    async fn delay_ns(&mut self, ns: u32) {
        self.0
            .0
            .borrow_mut()
            .advance(RelativeDuration::from_nanos(u64::from(ns)));
    }
}

fn connected_model(sample_als: u16, sample_white: u16) -> (Veml7700<ModelI2c>, ModelDelay) {
    let mut model = Veml7700Model::new();
    model.set_raw_sample(sample_als, sample_white);
    let shared = SharedModel(Rc::new(RefCell::new(model)));
    (Veml7700::new(ModelI2c(shared.clone())), ModelDelay(shared))
}

#[test]
fn probe_accepts_the_fixed_address_little_endian_id() {
    let (mut sensor, _delay) = connected_model(0, 0);
    let id = block_on(sensor.probe()).expect("probe against the model");
    assert_eq!(id.raw(), 0xC481);
}

#[test]
fn measure_once_returns_the_injected_pair_after_the_driver_delay_and_restores_state() {
    let als = 0x1234;
    let white = 0x5678;
    let (mut sensor, mut delay) = connected_model(als, white);
    let sample = block_on(sensor.measure_once(&mut delay, MeasurementConfig::safe_bright_start()))
        .expect("measure_once against the model");

    assert_eq!(sample.als, AlsCounts::from_counts(als));
    assert_eq!(sample.white, WhiteCounts::from_counts(white));
    assert_eq!(sample.waited_us, 133_500);

    let configuration = block_on(sensor.read_configuration()).expect("restored configuration");
    let power_saving = block_on(sensor.read_power_saving()).expect("restored power saving");
    assert_eq!(
        configuration,
        ConfigurationSnapshot::silicon_reset_default()
    );
    assert!(!power_saving.enabled);
    assert_eq!(power_saving.mode, PowerSavingMode::Mode1);
}

#[test]
fn public_power_operations_observe_the_documented_mode_2_refresh_boundary() {
    let (mut sensor, model) = connected_model(1, 1);
    block_on(sensor.set_power_saving(PowerSavingConfig::new(true, PowerSavingMode::Mode2)))
        .expect("configure modeled cadence");
    block_on(sensor.set_power_state(PowerState::Active)).expect("wake modeled device");
    model.advance(RelativeDuration::from_micros(132_500));
    model.set_raw_sample(2, 2);

    model.advance(RelativeDuration::from_micros(1_099_999));
    assert_eq!(
        block_on(sensor.read_als_snapshot()).expect("pre-boundary ALS"),
        AlsCounts::from_counts(1)
    );
    model.advance(RelativeDuration::from_micros(1));
    assert_eq!(
        block_on(sensor.read_als_snapshot()).expect("at-boundary ALS"),
        AlsCounts::from_counts(2)
    );
}

#[test]
fn threshold_monitor_public_operations_qualify_after_configured_persistence() {
    let (mut sensor, model) = connected_model(250, 0);
    let thresholds = Thresholds::new(AlsCounts::from_counts(100), AlsCounts::from_counts(200))
        .expect("ordered thresholds");
    let monitor = ThresholdMonitorConfig::new(
        MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms100),
        thresholds,
        Persistence::Four,
        PowerSavingConfig::disabled(),
    );
    block_on(sensor.arm_threshold_monitor(monitor)).expect("arm against the model");

    let observed = block_on(sensor.read_thresholds()).expect("threshold readback");
    assert_eq!(observed, thresholds);
    model.advance(RelativeDuration::from_micros(132_500));
    for _ in 0..2 {
        model.advance(RelativeDuration::from_micros(130_000));
        assert!(
            !block_on(sensor.read_threshold_status())
                .expect("pre-qualification status")
                .high
        );
    }
    model.advance(RelativeDuration::from_micros(130_000));
    let status = block_on(sensor.read_threshold_status()).expect("qualified status");
    assert!(!status.low);
    assert!(status.high);

    block_on(sensor.disable_threshold_monitor()).expect("disable monitor");
    assert_eq!(
        block_on(sensor.read_configuration())
            .expect("configuration after disable")
            .threshold_monitor,
        ThresholdMonitorState::Disabled
    );
}

#[test]
fn public_channel_reads_can_observe_independently_refreshed_generations() {
    let (mut sensor, model) = connected_model(10, 20);
    model.set_white_phase_offset(RelativeDuration::from_micros(10));
    block_on(sensor.set_power_state(PowerState::Active)).expect("wake modeled device");
    model.advance(RelativeDuration::from_micros(132_510));
    assert_eq!(
        block_on(sensor.read_als_snapshot()).expect("initial ALS"),
        AlsCounts::from_counts(10)
    );
    assert_eq!(
        block_on(sensor.read_white_snapshot()).expect("initial white"),
        WhiteCounts::from_counts(20)
    );

    model.set_raw_sample(30, 40);
    model.advance(RelativeDuration::from_micros(129_990));
    assert_eq!(
        block_on(sensor.read_als_snapshot()).expect("refreshed ALS"),
        AlsCounts::from_counts(30)
    );
    assert_eq!(
        block_on(sensor.read_white_snapshot()).expect("retained white"),
        WhiteCounts::from_counts(20)
    );
    model.advance(RelativeDuration::from_micros(10));
    assert_eq!(
        block_on(sensor.read_white_snapshot()).expect("refreshed white"),
        WhiteCounts::from_counts(40)
    );
}
