//! Independently derived VEML7700 register and conversion facts for this slice.
//!
//! Values come from the pinned datasheet and `docs/HARDWARE_CONTRACT.md`. They
//! are not imported from the production driver.

/// Fixed 7-bit I²C address.
pub const I2C_ADDRESS: u8 = 0x10;

/// ID register word at the fixed-address option: bytes `0x81`, `0xC4`.
pub const DEVICE_ID: u16 = 0xC481;

pub(crate) const POINTER_CONFIGURATION: u8 = 0x00;
pub(crate) const POINTER_POWER_SAVING: u8 = 0x03;
pub(crate) const POINTER_ALS: u8 = 0x04;
pub(crate) const POINTER_WHITE: u8 = 0x05;
pub(crate) const POINTER_ID: u8 = 0x07;

pub(crate) const RESET_CONFIGURATION: u16 = 0x0001;
pub(crate) const RESET_POWER_SAVING: u16 = 0x0000;

const SHUTDOWN_BIT: u16 = 1 << 0;
const POWER_SAVING_ENABLE_BIT: u16 = 1 << 0;
const INTEGRATION_FIELD_SHIFT: u16 = 6;
const INTEGRATION_FIELD_MASK: u16 = 0b1111;
const SHUTDOWN_TO_ACTIVE_WAKE_US: u64 = 2_500;
const CONSERVATIVE_INTEGRATION_PERCENT: u64 = 130;

pub(crate) const fn is_shutdown(configuration: u16) -> bool {
    configuration & SHUTDOWN_BIT != 0
}

pub(crate) const fn without_shutdown(configuration: u16) -> u16 {
    configuration & !SHUTDOWN_BIT
}

pub(crate) const fn power_saving_enabled(power_saving: u16) -> bool {
    power_saving & POWER_SAVING_ENABLE_BIT != 0
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
