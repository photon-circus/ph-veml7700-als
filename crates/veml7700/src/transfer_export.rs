//! Host export against a **future** `ph-curves` gen-lib API.
//!
//! This module is the smoke test. It does not compile on `ph-curves` 0.2.1
//! because [`DefinitionsFile::transfer_families`] and [`DefinitionsFile::gaps`]
//! do not exist there. That is intentional: a `formula` dump that 0.2.1 would
//! accept would drop applicability, gaps, and saturation.
//!
//! Apply `exploration/ph-curves-transfer/ph-curves-0.2.1.patch` on `ph-curves`
//! before expecting this module to type-check.

use ph_curves::r#gen::{DefinitionsFile, Error, GenerateOptions, generate};

/// Family TOML from [`crate::transfer::PH_CURVES_FAMILY_TOML`].
pub const FAMILY_TOML: &str = crate::transfer::PH_CURVES_FAMILY_TOML;

/// Parse the VEML7700 family and emit sparse integer transfers.
///
/// Requires the patched `ph-curves` generator. On 0.2.1 this function does
/// not compile.
pub fn generate_als_family() -> Result<String, Error> {
    let defs = DefinitionsFile::from_toml_str(FAMILY_TOML).map_err(Error::Toml)?;
    if defs.transfer_families().is_empty() {
        return Err(Error::Validation(
            "expected als_scaled_polynomial transfer family".into(),
        ));
    }
    if defs.gaps().get("white_channel").is_none() {
        return Err(Error::Validation("expected white_channel gap".into()));
    }
    if defs.gaps().get("ir_lux_optimization").is_none() {
        return Err(Error::Validation("expected ir_lux_optimization gap".into()));
    }
    generate(&defs, &GenerateOptions::default())
}
