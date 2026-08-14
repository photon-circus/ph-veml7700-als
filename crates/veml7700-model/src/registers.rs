//! Independently derived VEML7700 register and conversion facts for this slice.
//!
//! Values come from the pinned datasheet and `docs/HARDWARE_CONTRACT.md`. They
//! are not imported from the production driver.

/// Fixed 7-bit I²C address.
pub const I2C_ADDRESS: u8 = 0x10;

/// ID register word at the fixed-address option: bytes `0x81`, `0xC4`.
pub const DEVICE_ID: u16 = 0xC481;

pub(crate) const POINTER_CONFIGURATION: u8 = 0x00;
pub(crate) const POINTER_HIGH_THRESHOLD: u8 = 0x01;
pub(crate) const POINTER_LOW_THRESHOLD: u8 = 0x02;
pub(crate) const POINTER_POWER_SAVING: u8 = 0x03;
pub(crate) const POINTER_ALS: u8 = 0x04;
pub(crate) const POINTER_WHITE: u8 = 0x05;
pub(crate) const POINTER_THRESHOLD_STATUS: u8 = 0x06;
pub(crate) const POINTER_ID: u8 = 0x07;

/// Source-declared: the register-format note states `0x0001` for `0x00`.
pub(crate) const RESET_CONFIGURATION: u16 = 0x0001;
/// **Assumed, not declared.** No passage states a power-on value for `0x03`;
/// Table 4 constrains bits 15:3 to zero, which is a write-validity rule rather
/// than a reset value. This is the value every defined field takes at zero, and
/// the model needs *some* word to represent a device that has never had power
/// saving written.
///
/// The naming deliberately matches `RESET_CONFIGURATION` above, which is why
/// this comment exists: the two constants look like peers and are not. See the
/// Assumption row in `docs/HARDWARE_CONTRACT.md` §4 and D-030 — the driver
/// carries no equivalent, because it reads `0x03` before acting on it.
pub(crate) const RESET_POWER_SAVING: u16 = 0x0000;

const SHUTDOWN_BIT: u16 = 1 << 0;
const GAIN_FIELD_MASK: u16 = 0b11 << 11;
const INTEGRATION_FIELD_SHIFT: u16 = 6;
const INTEGRATION_FIELD_MASK: u16 = 0b1111;
const PERSISTENCE_FIELD_SHIFT: u16 = 4;
const PERSISTENCE_FIELD_MASK: u16 = 0b11;
const THRESHOLD_MONITOR_BIT: u16 = 1 << 1;
const SUPPORTED_CONFIGURATION_MASK: u16 = GAIN_FIELD_MASK
    | (INTEGRATION_FIELD_MASK << INTEGRATION_FIELD_SHIFT)
    | (PERSISTENCE_FIELD_MASK << PERSISTENCE_FIELD_SHIFT)
    | THRESHOLD_MONITOR_BIT
    | SHUTDOWN_BIT;
const SHUTDOWN_TO_ACTIVE_WAKE_US: u64 = 2_500;
const CONSERVATIVE_INTEGRATION_PERCENT: u64 = 130;

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

pub(crate) const fn persistence_count(configuration: u16) -> u8 {
    1 << ((configuration >> PERSISTENCE_FIELD_SHIFT) & PERSISTENCE_FIELD_MASK)
}

pub(crate) const fn configuration_fields_are_supported(configuration: u16) -> bool {
    configuration & !SUPPORTED_CONFIGURATION_MASK == 0
}

pub(crate) const fn power_saving_is_supported(power_saving: u16) -> bool {
    power_saving & !0b111 == 0
}

pub(crate) const fn power_saving_is_enabled(power_saving: u16) -> bool {
    power_saving & 1 != 0
}

pub(crate) const fn integration_field(configuration: u16) -> u16 {
    (configuration >> INTEGRATION_FIELD_SHIFT) & INTEGRATION_FIELD_MASK
}

/// Nominal integration time in microseconds for a documented encoding.
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

/// Conservative conversion bound after a shutdown-to-active wake, in nanoseconds.
///
/// Bound is 2.5 ms plus 130% of the selected integration time. This is not the
/// driver's longer wait, which also adds software margin.
pub(crate) const fn conversion_bound_ns(configuration: u16) -> Option<u64> {
    let Some(integration_us) = documented_integration_us(configuration) else {
        return None;
    };
    let conservative_us = integration_us.saturating_mul(CONSERVATIVE_INTEGRATION_PERCENT) / 100;
    Some(
        SHUTDOWN_TO_ACTIVE_WAKE_US
            .saturating_add(conservative_us)
            .saturating_mul(1_000),
    )
}

/// Deterministic interval between recurring refreshes, in nanoseconds.
pub(crate) const fn refresh_interval_ns(configuration: u16, power_saving: u16) -> Option<u64> {
    let Some(integration_us) = documented_integration_us(configuration) else {
        return None;
    };
    if !power_saving_is_enabled(power_saving) {
        return Some(
            integration_us
                .saturating_mul(CONSERVATIVE_INTEGRATION_PERCENT)
                .saturating_mul(1_000)
                / 100,
        );
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
