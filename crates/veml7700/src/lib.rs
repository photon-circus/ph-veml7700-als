//! Async `#![no_std]` driver for the Vishay VEML7700 ambient-light sensor.
//!
//! The driver distinguishes a register snapshot from a fresh measurement it
//! deliberately configured, waited for, froze in shutdown, and read. Nominal
//! illuminance conversion uses integer micro-lux-per-count factors from the
//! vendor resolution table; it does not claim application-specific optical
//! calibration or empirical high-lux correction.
//!
//! The VEML7700 exposes threshold flags but no dedicated interrupt pin. An
//! enabled threshold monitor owns gain, integration time, persistence,
//! thresholds, power state, and power-saving cadence as one monitored domain.
//!
//! Construction performs no I/O. The driver stores no hardware-state cache,
//! allocates nothing, and depends on no MCU HAL or executor.
//!
//! # Status
//!
//! This crate is incubating and unpublished. Pure and scripted-I2C tests cover
//! codec and protocol behavior. A coupled test-only fake exercises autonomous
//! state but is not the independent I2C-level model. That independent model
//! currently covers only `probe` and one successful `measure_once` slice. No
//! reviewed physical or calibrated-optical evidence exists.

#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(test, allow(clippy::std_instead_of_alloc, clippy::std_instead_of_core))]
#![deny(clippy::correctness)]
#![warn(
    clippy::suspicious,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::cloned_instead_of_copied,
    clippy::explicit_iter_loop,
    clippy::implicit_clone,
    clippy::inconsistent_struct_constructor,
    clippy::manual_assert,
    clippy::manual_let_else,
    clippy::match_same_arms,
    clippy::needless_pass_by_value,
    clippy::semicolon_if_nothing_returned,
    clippy::uninlined_format_args,
    clippy::unnested_or_patterns,
    clippy::std_instead_of_core,
    clippy::std_instead_of_alloc,
    clippy::alloc_instead_of_core,
    clippy::undocumented_unsafe_blocks,
    clippy::missing_const_for_fn
)]
#![allow(
    clippy::mod_module_files,
    clippy::self_named_module_files,
    clippy::similar_names,
    clippy::type_complexity,
    clippy::must_use_candidate,
    clippy::module_name_repetitions,
    clippy::wildcard_imports,
    clippy::items_after_statements
)]

mod config;
mod driver;
mod error;
mod id;
mod illuminance;
mod measurement;
mod power;
mod register;
mod threshold;
mod timing;

pub use config::{
    ConfigDecodeError, ConfigurationSnapshot, Gain, IntegrationTime, MeasurementConfig,
    Persistence, PowerState, ThresholdMonitorState,
};
pub use driver::{I2C_ADDRESS, Veml7700};
pub use error::{
    BusContext, ConfigurationError, Error, MeasureOnceError, MeasureStage, Operation, ProbeError,
    ThresholdMonitorError, ThresholdMonitorStage,
};
pub use id::DeviceId;
pub use illuminance::{MicroLux, NominalScale};
pub use measurement::{
    AlsCounts, DeviceSnapshot, FreshMeasurement, MeasurementPairCoherence, SnapshotMeasurement,
    WhiteCounts,
};
pub use power::{PowerSavingConfig, PowerSavingDecodeError, PowerSavingMode, PowerSavingSnapshot};
pub use threshold::{
    ThresholdMonitorConfig, ThresholdStatus, ThresholdStatusDecodeError, Thresholds,
};
pub use timing::{
    INTEGRATION_TOLERANCE_PERCENT, MEASUREMENT_MARGIN_US, MeasurementTiming, WAKE_UP_DELAY_US,
};

#[cfg(test)]
extern crate alloc;
#[cfg(test)]
extern crate std;
#[cfg(test)]
mod testing;
