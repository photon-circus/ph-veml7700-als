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
//! # Example
//!
//! The application supplies its platform's async I²C bus and delay provider:
//!
//! ```rust,no_run
//! use embedded_hal_async::{delay::DelayNs, i2c::I2c};
//! use ph_veml7700_als::{MeasurementConfig, Veml7700};
//!
//! async fn sample<I2C, D>(i2c: I2C, delay: &mut D)
//! where
//!     I2C: I2c,
//!     D: DelayNs,
//! {
//!     // Construction is inert: it performs no I²C transaction.
//!     let mut sensor = Veml7700::new(i2c);
//!     let _device_id = sensor.probe().await.expect("VEML7700 probe failed");
//!
//!     // A snapshot may contain retained or stale data. ALS and white are read
//!     // sequentially and may straddle an autonomous refresh.
//!     let snapshot = sensor.snapshot().await.expect("snapshot failed");
//!
//!     // A fresh measurement is deliberately configured, timed, and frozen
//!     // before the ALS and white registers are read.
//!     let fresh = sensor
//!         .measure_once(delay, MeasurementConfig::safe_bright_start())
//!         .await
//!         .expect("fresh measurement failed");
//!
//!     let _counts = (snapshot.als.counts(), fresh.als.counts());
//! }
//! ```
//!
//! # Status
//!
//! **Lifecycle:** Incubating.
//!
//! **Distribution:** Unpublished; the candidate version is
//! `0.1.0-incubating.1` and the manifest retains `publish = false`.
//!
//! **Model conformance:** An independent I²C-level model covers `probe` and one
//! successful `measure_once` path only. All other public operations are outside
//! the current model claim.
//!
//! **Physical evidence:** None. No reviewed physical or calibrated-optical
//! evidence exists. Evidence applies only to the named operations, and eventual
//! publication would not imply hardware qualification.

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
