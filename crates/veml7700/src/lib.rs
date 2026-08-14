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
//!     // Start at the widest range. Unknown light can be direct sunlight, and a
//!     // narrower domain would saturate without saying so.
//!     let fresh = sensor
//!         .measure_once(delay, MeasurementConfig::maximum_range_start())
//!         .await
//!         .expect("fresh measurement failed");
//!
//!     // Saturation is not an error, so it must be checked. At maximum code the
//!     // conversion clipped: `nominal_illuminance` is the domain's ceiling, not an
//!     // observation, and it bounds nothing about the actual light.
//!     if fresh.als.is_saturated() {
//!         // Nothing wider exists — this is already the maximum range. Attenuate
//!         // optically, or record that the reading clipped rather than treating the
//!         // number as a value.
//!     } else {
//!         // Once the ambient range is known, a longer integration time or higher
//!         // gain gives a finer reading over a narrower span.
//!         let _micro_lux = fresh.nominal_illuminance.as_micro_lux();
//!     }
//!
//!     let _counts = (snapshot.als.counts(), fresh.als.counts());
//! }
//! ```
//!
//! ## Model conformance coverage
//!
//! Every positive claim below maps to one named test that drives the **public
//! driver API** through an abstract I²C boundary against an independently derived
//! device model. Nothing here establishes behavior on silicon.
//!
//! Two other test layers exist and must not be confused with this one. *Model-only*
//! tests exercise the model's own declared behavior, not the driver. *Scripted-I²C*
//! tests assert exact transactions, not device behavior. Neither is conformance.
//!
//! ### Covered
//!
//! | Public operation | Accepted initial state | Configuration exercised | Conformance test |
//! | --- | --- | --- | --- |
//! | `probe` | any | — | `probe_accepts_the_fixed_address_little_endian_id` |
//! | `measure_once` | reset / shut down | ×1/8, 25 ms, cadence disabled | `measure_once_returns_the_injected_pair_after_the_driver_delay_and_restores_state` |
//! | `measure_once` | active | ×1/8, 25 ms, cadence disabled | `measure_once_from_an_active_start_agrees_with_the_model` |
//! | `arm_threshold_monitor` | reset / shut down | ×1/8, 100 ms, persistence 4, cadence disabled | `threshold_monitor_public_operations_qualify_after_configured_persistence` |
//! | `arm_threshold_monitor` | active, monitor disabled | ×1/8, 100 ms, persistence 4, Mode 2 | `arming_the_monitor_from_an_active_start_agrees_with_the_model` |
//! | `arm_threshold_monitor` | active, monitor **enabled** | ×1/8, 100 ms, persistence 4, Mode 2 | `re_arming_an_enabled_active_monitor_agrees_with_the_model` |
//! | `read_threshold_status` | armed | **high direction only** | `threshold_monitor_public_operations_qualify_after_configured_persistence` |
//! | `read_thresholds` | armed | — | same, and `re_arming_an_enabled_active_monitor_agrees_with_the_model` |
//! | `disable_threshold_monitor` | armed, active | — | `threshold_monitor_public_operations_qualify_after_configured_persistence` |
//! | `set_power_saving` | shut down | Mode 2 enabled | `public_power_operations_observe_the_documented_mode_2_refresh_boundary` |
//! | `set_power_state` | reset / shut down | requests active only; no trace shuts an active device down | four traces |
//! | `read_als_snapshot` | active | — | `public_power_operations_observe_the_documented_mode_2_refresh_boundary`, `public_channel_reads_can_observe_independently_refreshed_generations` |
//! | `read_white_snapshot` | active | — | `public_channel_reads_can_observe_independently_refreshed_generations` |
//! | `read_configuration` | various | — | four traces |
//! | `read_power_saving` | after restoration | — | `measure_once_returns_the_injected_pair_after_the_driver_delay_and_restores_state` |
//!
//! ### Not covered
//!
//! These public operations have **no** driver-versus-model trace. They are tested by
//! other layers, which establish different and weaker things.
//!
//! | Public operation | Why it is absent |
//! | --- | --- |
//! | `read_device_id` | `probe` reads the same register internally, so the codec is exercised — but this operation is never called in a conformance trace. |
//! | `inspect` | Never called. |
//! | `snapshot` | Never called. Its component reads are covered separately. |
//! | `set_measurement_config` | Never called. Reconfiguration is exercised only as part of `measure_once` and `arm_threshold_monitor`. |
//! | `measure_once_with_timing` | Never called with caller-supplied timing. `measure_once` delegates to it with conservative timing only, so the custom-timing surface is unexercised. |
//!
//! ### Configuration domain not exercised
//!
//! Coverage above is narrower than the operations imply. Within the covered
//! operations, conformance traces exercise only:
//!
//! | Domain | Exercised | Not exercised |
//! | --- | --- | --- |
//! | Gain | ×1/8 | ×1, ×2, ×1/4 |
//! | Integration time | 25 ms (fresh capture), 100 ms (threshold) | 50, 200, 400, 800 ms |
//! | Persistence | 4 | 1, 2, 8 |
//! | Power-saving mode | Mode 1, Mode 2 | Mode 3, Mode 4 |
//! | Threshold direction | high qualification | **low qualification is never exercised** |
//!
//! A claim about a gain, integration time, persistence value, cadence mode, or
//! threshold direction outside this table has no conformance evidence behind it.
//!
//! Threshold traces deliberately use 100 ms rather than the
//! [`maximum_range_start`] preset: 25 ms has no vendor-documented power-saving
//! refresh time, so pairing it with an enabled cadence would ask for behavior no
//! source establishes.
//!
//! [`maximum_range_start`]: https://docs.rs/ph-veml7700-als/latest/ph_veml7700_als/struct.MeasurementConfig.html#method.maximum_range_start
//!
//! ### Boundaries
//!
//! - `ph-veml7700-als-model` is a **repository-only, unpublished** test artifact. It
//!   is not part of this package and cannot be depended on.
//! - Conformance runs in the repository only. `tests/device_model.rs` and the
//!   path-only model dependency are excluded from the published package, so tests
//!   run against the unpacked archive establish that the crate builds and passes its
//!   own tests standalone — never model conformance.
//! - Model agreement establishes that two independent derivations of the same
//!   documents agree. It does not establish that either matches silicon.
//!
//! # Status
//!
//! **Lifecycle:** Incubating.
//!
//! **Distribution:** Unpublished; the candidate version is
//! `0.1.0-incubating.1` and the manifest retains `publish = false`.
//!
//! **Model conformance:** An independent I²C-level model covers twelve public
//! operations at gain ×1/8 and 100 ms only, from shut-down and active starts,
//! with high-threshold qualification only. `read_device_id`, `inspect`,
//! `snapshot`, `set_measurement_config`, and custom-timing
//! `measure_once_with_timing` have no conformance trace. See the coverage matrix
//! for the exact domain.
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
