//! Independently derived VEML7700 register and conversion facts for this slice.
//!
//! Each data-derived value cites the shared registry proposition it reacts to.
//! Values are not imported from the production driver.

/// Model address selection (`S-05`).
pub const I2C_ADDRESS: u8 = 0x10;

/// Model identity response (`S-43`).
pub const DEVICE_ID: u16 = 0xC481;

// Model register-map reaction to `S-09`.
pub(crate) const POINTER_CONFIGURATION: u8 = 0x00;
pub(crate) const POINTER_HIGH_THRESHOLD: u8 = 0x01;
pub(crate) const POINTER_LOW_THRESHOLD: u8 = 0x02;
pub(crate) const POINTER_POWER_SAVING: u8 = 0x03;
pub(crate) const POINTER_ALS: u8 = 0x04;
pub(crate) const POINTER_WHITE: u8 = 0x05;
pub(crate) const POINTER_THRESHOLD_STATUS: u8 = 0x06;
pub(crate) const POINTER_ID: u8 = 0x07;

/// Model reset configuration selected from `S-12`.
pub(crate) const RESET_CONFIGURATION: u16 = 0x0001;

const SHUTDOWN_BIT: u16 = 1 << 0;
const GAIN_FIELD_MASK: u16 = 0b11 << 11;
const INTEGRATION_FIELD_SHIFT: u16 = 6;
const INTEGRATION_FIELD_MASK: u16 = 0b1111;
const PERSISTENCE_FIELD_SHIFT: u16 = 4;
const PERSISTENCE_FIELD_MASK: u16 = 0b11;
const THRESHOLD_MONITOR_BIT: u16 = 1 << 1;
// Model configuration-codec reaction to `S-13`, `S-14`, `S-15`, `S-16`,
// `S-17`, and `S-18`.
const SUPPORTED_CONFIGURATION_MASK: u16 = GAIN_FIELD_MASK
    | (INTEGRATION_FIELD_MASK << INTEGRATION_FIELD_SHIFT)
    | (PERSISTENCE_FIELD_MASK << PERSISTENCE_FIELD_SHIFT)
    | THRESHOLD_MONITOR_BIT
    | SHUTDOWN_BIT;
// Ideal-model wake reaction to `S-23`.
const SHUTDOWN_TO_ACTIVE_WAKE_US: u64 = 2_500;

pub(crate) const fn is_shutdown(configuration: u16) -> bool {
    configuration & SHUTDOWN_BIT != 0
}

pub(crate) const fn without_shutdown(configuration: u16) -> u16 {
    configuration & !SHUTDOWN_BIT
}

pub(crate) const fn without_monitor(configuration: u16) -> u16 {
    configuration & !THRESHOLD_MONITOR_BIT
}

pub(crate) const fn threshold_monitor_is_enabled(configuration: u16) -> bool {
    configuration & THRESHOLD_MONITOR_BIT != 0
}

pub(crate) const fn configuration_fields_are_supported(configuration: u16) -> bool {
    configuration & !SUPPORTED_CONFIGURATION_MASK == 0
}

pub(crate) const fn power_saving_is_supported(power_saving: u16) -> bool {
    // Model power-saving codec reaction to `S-20`.
    power_saving & !0b111 == 0
}

pub(crate) const fn power_saving_is_enabled(power_saving: u16) -> bool {
    power_saving & 1 != 0
}

pub(crate) const fn integration_field(configuration: u16) -> u16 {
    (configuration >> INTEGRATION_FIELD_SHIFT) & INTEGRATION_FIELD_MASK
}

/// Model integration-codec reaction to `S-15`.
pub(crate) const fn documented_integration_us(configuration: u16) -> Option<u64> {
    let milliseconds = match integration_field(configuration) {
        0b1100 => 25,
        0b1000 => 50,
        0b0000 => 100,
        0b0001 => 200,
        0b0010 => 400,
        0b0011 => 800,
        _ => return None,
    };
    Some(milliseconds * 1_000)
}

/// Ideal-model conversion completion after a shutdown-to-active wake, in nanoseconds.
///
/// Model reaction to `S-15` and `S-23`: use the nominal integration point. The
/// driver's separate timing policy reacts to `S-24`; the ideal model does not.
pub(crate) const fn conversion_completion_ns(configuration: u16) -> Option<u64> {
    let Some(integration_us) = documented_integration_us(configuration) else {
        return None;
    };
    Some((SHUTDOWN_TO_ACTIVE_WAKE_US + integration_us) * 1_000)
}

/// Deterministic interval between recurring refreshes, in nanoseconds.
///
/// With power saving disabled, use the ideal nominal integration interval
/// (`S-15`). Enabled power saving is limited to the represented `S-21` domain;
/// `S-22` is not completed by assuming gain independence.
pub(crate) const fn refresh_interval_ns(configuration: u16, power_saving: u16) -> Option<u64> {
    let Some(integration_us) = documented_integration_us(configuration) else {
        return None;
    };
    // Unknown initial reserved bits are injectable under `S-11`, but the model
    // does not invent their effect on autonomous behavior.
    if power_saving & !0b111 != 0 {
        return None;
    }
    if !power_saving_is_enabled(power_saving) {
        return Some(integration_us * 1_000);
    }

    // `S-21` records the enabled cadence only for gain ×2. Other gains remain
    // unsupported until `S-22` is resolved or a scoped input supplies them.
    if configuration & GAIN_FIELD_MASK != 0b01 << 11 {
        return None;
    }

    let integration_ms = integration_us / 1_000;
    let sleep_ms = match (power_saving >> 1) & 0b11 {
        0 => 500,
        1 => 1_000,
        2 => 2_000,
        _ => 4_000,
    };
    match integration_ms {
        100 | 200 | 400 | 800 => Some((integration_ms + sleep_ms) * 1_000_000),
        _ => None,
    }
}
