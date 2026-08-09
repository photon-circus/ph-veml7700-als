//! Device-identity decoding.

/// Expected VEML7700 ID word at the fixed write-address option `0x20`.
pub(crate) const EXPECTED_DEVICE_ID: u16 = 0xC481;

/// Decoded VEML7700 ID register.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DeviceId {
    raw: u16,
}

impl DeviceId {
    /// Decode an observed raw ID word.
    pub const fn from_raw(raw: u16) -> Self {
        Self { raw }
    }

    /// Return the complete raw ID word.
    pub const fn raw(self) -> u16 {
        self.raw
    }

    /// Return the fixed low-byte device code.
    pub const fn device_code(self) -> u8 {
        self.raw as u8
    }

    /// Return the address-option code from the high byte.
    pub const fn address_option_code(self) -> u8 {
        (self.raw >> 8) as u8
    }

    /// Return whether this ID matches the supported fixed-address VEML7700.
    pub const fn is_supported(self) -> bool {
        self.raw == EXPECTED_DEVICE_ID
    }
}
