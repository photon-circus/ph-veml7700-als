//! Driver-versus-model tests for the declared probe and `measure_once` slice.
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
    AlsCounts, ConfigurationSnapshot, MeasurementConfig, PowerSavingMode, Veml7700, WhiteCounts,
};
use ph_veml7700_als_model::{
    NoAcknowledgeSource, RelativeDuration, TransportError, Unsupported, Veml7700Model,
};

#[derive(Clone)]
struct SharedModel(Rc<RefCell<Veml7700Model>>);

struct ModelI2c(SharedModel);
struct ModelDelay(SharedModel);

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
