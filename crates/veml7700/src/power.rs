//! Power-saving configuration and codec.

use crate::config::{Gain, IntegrationTime, MeasurementConfig};

/// Driver power-saving codec reaction to `S-20`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PowerSavingMode {
    /// Mode 1.
    Mode1,
    /// Mode 2.
    Mode2,
    /// Mode 3.
    Mode3,
    /// Mode 4.
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

    /// Return the nominal refresh time in the exact `S-21` cadence domain.
    ///
    /// Gain other than ×2 returns `None` because gain independence is undefined
    /// (`S-22`). The undocumented 25 ms and 50 ms combinations also return
    /// `None` (`S-44`).
    pub const fn nominal_refresh_time_ms(self, measurement: MeasurementConfig) -> Option<u32> {
        if !matches!(measurement.gain(), Gain::X2) {
            return None;
        }
        let base = match measurement.integration_time() {
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
#[non_exhaustive]
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
    // Driver reserved-field reaction to `S-20`.
    let reserved = word & !0b111;
    if reserved != 0 {
        return Err(PowerSavingDecodeError::ReservedBits { observed: reserved });
    }
    Ok(PowerSavingSnapshot {
        enabled: word & 1 != 0,
        mode: PowerSavingMode::from_word(word),
    })
}

impl core::fmt::Display for PowerSavingDecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ReservedBits { observed } => {
                write!(f, "reserved power-saving bits were set: {observed:#06x}")
            }
        }
    }
}

impl core::error::Error for PowerSavingDecodeError {}

#[cfg(test)]
mod tests {
    use super::*;

    /// Literal driver test vectors for `S-20`, not round trips.
    ///
    /// Same reasoning as the configuration vectors: the exhaustive round trip
    /// below proves encoder and decoder agree, which they would continue to do
    /// with `PSM` shifted to the wrong bits. These vectors pin the positions.
    ///
    /// The `PSM_EN` bit sits *below* the mode field, so a word for "mode 2
    /// enabled" is `0b011`, not `0b101`. Getting that backwards is the natural
    /// mistake and the reason the enable case is tested separately from the
    /// mode case.
    #[test]
    fn power_saving_fields_occupy_the_contract_bit_positions() {
        // Mode selection is bits 2:1; with the enable bit clear the word is the
        // mode alone.
        for (mode, bits) in [
            (PowerSavingMode::Mode1, 0b00_u16),
            (PowerSavingMode::Mode2, 0b01),
            (PowerSavingMode::Mode3, 0b10),
            (PowerSavingMode::Mode4, 0b11),
        ] {
            assert_eq!(
                PowerSavingConfig::new(false, mode).encode(),
                bits << 1,
                "{mode:?} must occupy bits 2:1"
            );
        }

        // PSM_EN is bit 0.
        assert_eq!(
            PowerSavingConfig::new(true, PowerSavingMode::Mode1).encode(),
            0b001
        );
        // Mode 4 enabled: mode `11` in bits 2:1, enable in bit 0.
        assert_eq!(
            PowerSavingConfig::new(true, PowerSavingMode::Mode4).encode(),
            0b111
        );
        // The reset word is every field zero: mode 1, cadence disabled.
        assert_eq!(
            decode_power_saving(0x0000).unwrap().as_config(),
            PowerSavingConfig::new(false, PowerSavingMode::Mode1)
        );
    }

    #[test]
    fn every_power_saving_configuration_round_trips() {
        for mode in [
            PowerSavingMode::Mode1,
            PowerSavingMode::Mode2,
            PowerSavingMode::Mode3,
            PowerSavingMode::Mode4,
        ] {
            for enabled in [false, true] {
                let config = PowerSavingConfig::new(enabled, mode);
                let decoded = decode_power_saving(config.encode()).unwrap();
                assert_eq!(decoded.as_config(), config);
            }
        }
    }

    #[test]
    fn every_reserved_power_saving_bit_is_rejected() {
        for bit in 3_u32..16 {
            let observed = 1_u16 << bit;
            assert_eq!(
                decode_power_saving(observed),
                Err(PowerSavingDecodeError::ReservedBits { observed })
            );
        }
    }

    #[test]
    fn documented_refresh_table_is_exact() {
        let integrations = [
            IntegrationTime::Ms100,
            IntegrationTime::Ms200,
            IntegrationTime::Ms400,
            IntegrationTime::Ms800,
        ];
        let rows = [
            (PowerSavingMode::Mode1, [600, 700, 900, 1_300]),
            (PowerSavingMode::Mode2, [1_100, 1_200, 1_400, 1_800]),
            (PowerSavingMode::Mode3, [2_100, 2_200, 2_400, 2_800]),
            (PowerSavingMode::Mode4, [4_100, 4_200, 4_400, 4_800]),
        ];
        for (mode, expected) in rows {
            for (integration, expected_ms) in integrations.into_iter().zip(expected) {
                assert_eq!(
                    mode.nominal_refresh_time_ms(MeasurementConfig::new(Gain::X2, integration)),
                    Some(expected_ms)
                );
            }
            assert_eq!(
                mode.nominal_refresh_time_ms(MeasurementConfig::new(
                    Gain::X2,
                    IntegrationTime::Ms25
                )),
                None
            );
            assert_eq!(
                mode.nominal_refresh_time_ms(MeasurementConfig::new(
                    Gain::X2,
                    IntegrationTime::Ms50
                )),
                None
            );
            for gain in [Gain::X1, Gain::Div4, Gain::Div8] {
                assert_eq!(
                    mode.nominal_refresh_time_ms(MeasurementConfig::new(
                        gain,
                        IntegrationTime::Ms100
                    )),
                    None
                );
            }
        }
    }
}
