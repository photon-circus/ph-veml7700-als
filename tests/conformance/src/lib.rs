//! Test glue binding the public driver to the independent model.
//!
//! This package exists so the driver-versus-model boundary is a thing you can
//! point at. It depends on both public artifacts and neither depends on it, so
//! the arrow direction is checkable rather than asserted.
//!
//! # What the adapters must not do
//!
//! They are test glue, **not** the model's behavioral API. The rule that makes
//! the whole layer worth having: a model limitation and the model's
//! address-mismatch reaction to `S-05` must not become the same error. Flattening
//! them would let the driver
//! appear to handle a device response it never saw, and the harness would be
//! manufacturing the evidence it is supposed to collect.
//!
//! [`ModelBusError`] keeps them as separate variants for exactly that reason,
//! and [`map_error`] refuses to invent a mapping for anything it does not
//! recognise.

use core::cell::RefCell;
use std::rc::Rc;

use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::i2c::{
    Error as I2cError, ErrorKind, ErrorType, I2c, NoAcknowledgeSource as HalNack, Operation,
    SevenBitAddress,
};
use ph_veml7700_als::Veml7700;
use ph_veml7700_als_model::{
    NoAcknowledgeSource, RelativeDuration, RetainedInputs, TransportError, Unsupported,
    Veml7700Model,
};

#[derive(Clone)]
pub struct SharedModel(pub Rc<RefCell<Veml7700Model>>);

pub struct ModelI2c(pub SharedModel);
pub struct ModelDelay(pub SharedModel);

impl ModelDelay {
    pub fn set_raw_sample(&self, als: u16, white: u16) {
        self.0.0.borrow_mut().set_raw_sample(als, white);
    }

    pub fn set_white_phase_offset(&self, offset: RelativeDuration) {
        self.0.0.borrow_mut().set_white_phase_offset(offset);
    }

    pub fn advance(&self, elapsed: RelativeDuration) {
        self.0.0.borrow_mut().advance(elapsed);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelBusError {
    /// The model's address-mismatch reaction to `S-05`.
    NoAcknowledge(HalNack),
    /// A limit this model declares. **Not** a device response, and it must
    /// never be presented to the driver as one.
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

pub fn map_error(error: TransportError) -> ModelBusError {
    match error {
        TransportError::NoAcknowledge {
            source: NoAcknowledgeSource::Address,
        } => ModelBusError::NoAcknowledge(HalNack::Address),
        TransportError::Unsupported(reason) => ModelBusError::Unsupported(reason),
        // The model's result types are `#[non_exhaustive]`, so this arm exists
        // for variants added later. It must not guess: mapping an unknown model
        // result onto a device response would be exactly the fabrication this
        // adapter is required to avoid, and mapping it onto `Unsupported` would
        // need an `Unsupported` value nobody has. An unrecognised result means
        // this harness was not updated alongside the model, which is a defect in
        // the test rather than an expected boundary outcome, so it fails loudly.
        other => panic!("model transport result not handled by this harness: {other:?}"),
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

pub fn connected_model(sample_als: u16, sample_white: u16) -> (Veml7700<ModelI2c>, ModelDelay) {
    // Harness input for the undefined `S-11` initial word; not a model claim.
    let model = Veml7700Model::new(RetainedInputs::new(sample_als, sample_white, 0x0000));
    let shared = SharedModel(Rc::new(RefCell::new(model)));
    (Veml7700::new(ModelI2c(shared.clone())), ModelDelay(shared))
}
