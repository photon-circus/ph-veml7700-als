//! Power-saving configuration and codec.

use crate::config::IntegrationTime;

/// Power-saving cadence selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PowerSavingMode {
    /// Mode 1, the shortest documented sleep interval.
    Mode1,
    /// Mode 2.
    Mode2,
    /// Mode 3.
    Mode3,
    /// Mode 4, the longest documented sleep interval.
    Mode4,
}

impl PowerSavingMode {
    pub(crate) const fn bits(self) -> u16 {
        match self {
            Self::Mode1 => 0b00 << 1,
            Self::Mode2 => 0b01 << 1,
            Self::Mode3 => 0b10 << 1,
            Self::Mode4 => 0b11 << 1,
        }
    }

    pub(crate) const fn from_word(word: u16) -> Self {
        match (word >> 1) & 0b11 {
            0 => Self::Mode1,
            1 => Self::Mode2,
            2 => Self::Mode3,
            _ => Self::Mode4,
        }
    }

    /// Return the vendor's nominal refresh time for documented 100–800 ms modes.
    ///
    /// The vendor table does not define 25 ms or 50 ms power-saving cadence, so
    /// those combinations return `None` rather than extrapolating.
    pub const fn nominal_refresh_time_ms(self, integration: IntegrationTime) -> Option<u32> {
        let base = match integration {
            IntegrationTime::Ms100 => 100,
            IntegrationTime::Ms200 => 200,
            IntegrationTime::Ms400 => 400,
            IntegrationTime::Ms800 => 800,
            IntegrationTime::Ms25 | IntegrationTime::Ms50 => return None,
        };
        let sleep = match self {
            Self::Mode1 => 500,
            Self::Mode2 => 1000,
            Self::Mode3 => 2000,
            Self::Mode4 => 4000,
        };
        Some(base + sleep)
    }
}

/// Desired power-saving configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PowerSavingConfig {
    /// Whether power-saving cadence is enabled.
    pub enabled: bool,
    /// Cadence mode retained even while disabled.
    pub mode: PowerSavingMode,
}

impl PowerSavingConfig {
    /// Construct a power-saving configuration.
    pub const fn new(enabled: bool, mode: PowerSavingMode) -> Self {
        Self { enabled, mode }
    }

    /// Disabled power-saving with Mode 1 retained.
    pub const fn disabled() -> Self {
        Self::new(false, PowerSavingMode::Mode1)
    }

    pub(crate) const fn encode(self) -> u16 {
        self.mode.bits() | self.enabled as u16
    }
}

impl Default for PowerSavingConfig {
    fn default() -> Self {
        Self::disabled()
    }
}

/// Decoded power-saving register.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PowerSavingSnapshot {
    /// Whether power saving is enabled.
    pub enabled: bool,
    /// Observed cadence mode.
    pub mode: PowerSavingMode,
}

impl PowerSavingSnapshot {
    pub(crate) const fn as_config(self) -> PowerSavingConfig {
        PowerSavingConfig::new(self.enabled, self.mode)
    }
}

/// Failure decoding the power-saving register.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PowerSavingDecodeError {
    /// Reserved bits were non-zero.
    ReservedBits {
        /// Reserved bits that were observed set.
        observed: u16,
    },
}

pub(crate) const fn decode_power_saving(
    word: u16,
) -> Result<PowerSavingSnapshot, PowerSavingDecodeError> {
    let reserved = word & !0b111;
    if reserved != 0 {
        return Err(PowerSavingDecodeError::ReservedBits { observed: reserved });
    }
    Ok(PowerSavingSnapshot {
        enabled: word & 1 != 0,
        mode: PowerSavingMode::from_word(word),
    })
}
