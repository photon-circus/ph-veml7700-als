//! Independent VEML7700 device behavioral model.
//!
//! This crate predicts the datasheet-derived slice used for probe, fresh
//! measurement, autonomous cadence, and threshold-monitor conformance traces.
//! It does not depend on the production driver. The maintained claim, sources,
//! fidelity table, and nonclaims live in the crate README.
//!
//! Passing tests establish agreement with this model only. They do not
//! establish correctness on silicon.

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

mod duration;
mod error;
mod model;
mod registers;

pub use duration::RelativeDuration;
pub use error::{NoAcknowledgeSource, TransportError, Unsupported};
pub use model::{Inspection, MAX_ADVANCE, RetainedInputs, Veml7700Model};
pub use registers::{DEVICE_ID, I2C_ADDRESS};

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests;
