//! Configuration-register value types and codec.

/// Analog gain selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Gain {
    /// Gain ×1.
    X1,
    /// Gain ×2.
    X2,
    /// Gain ×1/8.
    Div8,
    /// Gain ×1/4.
    Div4,
}

impl Gain {
    pub(crate) const fn bits(self) -> u16 {
        match self {
            Self::X1 => 0b00 << 11,
            Self::X2 => 0b01 << 11,
            Self::Div8 => 0b10 << 11,
            Self::Div4 => 0b11 << 11,
        }
    }

    pub(crate) const fn from_bits(bits: u16) -> Self {
        match (bits >> 11) & 0b11 {
            0b00 => Self::X1,
            0b01 => Self::X2,
            0b10 => Self::Div8,
            _ => Self::Div4,
        }
    }
}

/// Integration-time selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum IntegrationTime {
    /// 25 ms.
    Ms25,
    /// 50 ms.
    Ms50,
    /// 100 ms.
    Ms100,
    /// 200 ms.
    Ms200,
    /// 400 ms.
    Ms400,
    /// 800 ms.
    Ms800,
}

impl IntegrationTime {
    /// Return the nominal integration time in milliseconds.
    pub const fn milliseconds(self) -> u32 {
        match self {
            Self::Ms25 => 25,
            Self::Ms50 => 50,
            Self::Ms100 => 100,
            Self::Ms200 => 200,
            Self::Ms400 => 400,
            Self::Ms800 => 800,
        }
    }

    pub(crate) const fn bits(self) -> u16 {
        match self {
            Self::Ms25 => 0b1100 << 6,
            Self::Ms50 => 0b1000 << 6,
            Self::Ms100 => 0b0000 << 6,
            Self::Ms200 => 0b0001 << 6,
            Self::Ms400 => 0b0010 << 6,
            Self::Ms800 => 0b0011 << 6,
        }
    }

    pub(crate) const fn from_bits(bits: u16) -> Result<Self, ConfigDecodeError> {
        match (bits >> 6) & 0b1111 {
            0b1100 => Ok(Self::Ms25),
            0b1000 => Ok(Self::Ms50),
            0b0000 => Ok(Self::Ms100),
            0b0001 => Ok(Self::Ms200),
            0b0010 => Ok(Self::Ms400),
            0b0011 => Ok(Self::Ms800),
            observed => Err(ConfigDecodeError::ReservedIntegrationTime { observed }),
        }
    }
}

/// Number of consecutive qualifying measurements required by the threshold monitor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Persistence {
    /// One qualifying measurement.
    One,
    /// Two consecutive qualifying measurements.
    Two,
    /// Four consecutive qualifying measurements.
    Four,
    /// Eight consecutive qualifying measurements.
    Eight,
}

impl Persistence {
    /// Return the qualifying-measurement count.
    pub const fn count(self) -> u8 {
        match self {
            Self::One => 1,
            Self::Two => 2,
            Self::Four => 4,
            Self::Eight => 8,
        }
    }

    pub(crate) const fn bits(self) -> u16 {
        match self {
            Self::One => 0b00 << 4,
            Self::Two => 0b01 << 4,
            Self::Four => 0b10 << 4,
            Self::Eight => 0b11 << 4,
        }
    }

    pub(crate) const fn from_bits(bits: u16) -> Self {
        match (bits >> 4) & 0b11 {
            0b00 => Self::One,
            0b01 => Self::Two,
            0b10 => Self::Four,
            _ => Self::Eight,
        }
    }
}

/// Sensor conversion power state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PowerState {
    /// Conversions are enabled.
    Active,
    /// Conversion circuitry is shut down and the last data remains readable.
    Shutdown,
}

impl PowerState {
    pub(crate) const fn bit(self) -> u16 {
        match self {
            Self::Active => 0,
            Self::Shutdown => 1,
        }
    }

    pub(crate) const fn from_word(word: u16) -> Self {
        if word & 1 == 0 {
            Self::Active
        } else {
            Self::Shutdown
        }
    }
}

/// Whether threshold monitoring is enabled in the configuration register.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ThresholdMonitorState {
    /// Threshold monitoring is disabled.
    Disabled,
    /// Threshold monitoring is enabled; status is available only by polling.
    Enabled,
}

impl ThresholdMonitorState {
    pub(crate) const fn bit(self) -> u16 {
        match self {
            Self::Disabled => 0,
            Self::Enabled => 1 << 1,
        }
    }

    pub(crate) const fn from_word(word: u16) -> Self {
        if word & (1 << 1) == 0 {
            Self::Disabled
        } else {
            Self::Enabled
        }
    }
}

/// Gain and integration-time pair defining one measurement domain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MeasurementConfig {
    gain: Gain,
    integration_time: IntegrationTime,
}

impl MeasurementConfig {
    /// Construct a measurement configuration.
    pub const fn new(gain: Gain, integration_time: IntegrationTime) -> Self {
        Self {
            gain,
            integration_time,
        }
    }

    /// Vendor silicon reset-domain measurement fields: gain ×1 and 100 ms.
    pub const fn silicon_reset_default() -> Self {
        Self::new(Gain::X1, IntegrationTime::Ms100)
    }

    /// Conservative unknown-light starting point: gain ×1/8 and 100 ms.
    pub const fn safe_bright_start() -> Self {
        Self::new(Gain::Div8, IntegrationTime::Ms100)
    }

    /// Return the selected gain.
    pub const fn gain(self) -> Gain {
        self.gain
    }

    /// Return the selected integration time.
    pub const fn integration_time(self) -> IntegrationTime {
        self.integration_time
    }

    pub(crate) const fn bits(self) -> u16 {
        self.gain.bits() | self.integration_time.bits()
    }
}

impl Default for MeasurementConfig {
    fn default() -> Self {
        Self::safe_bright_start()
    }
}

/// Decoded configuration-register snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ConfigurationSnapshot {
    /// Observed measurement domain.
    pub measurement: MeasurementConfig,
    /// Observed threshold persistence.
    pub persistence: Persistence,
    /// Observed threshold-monitor enable state.
    pub threshold_monitor: ThresholdMonitorState,
    /// Observed sensor power state.
    pub power_state: PowerState,
}

impl ConfigurationSnapshot {
    /// Return the documented reset value decoded as a snapshot.
    pub const fn silicon_reset_default() -> Self {
        Self {
            measurement: MeasurementConfig::silicon_reset_default(),
            persistence: Persistence::One,
            threshold_monitor: ThresholdMonitorState::Disabled,
            power_state: PowerState::Shutdown,
        }
    }

    pub(crate) const fn encode(self) -> u16 {
        self.measurement.bits()
            | self.persistence.bits()
            | self.threshold_monitor.bit()
            | self.power_state.bit()
    }

    pub(crate) const fn with_measurement(mut self, measurement: MeasurementConfig) -> Self {
        self.measurement = measurement;
        self
    }

    pub(crate) const fn with_persistence(mut self, persistence: Persistence) -> Self {
        self.persistence = persistence;
        self
    }

    pub(crate) const fn with_monitor(mut self, state: ThresholdMonitorState) -> Self {
        self.threshold_monitor = state;
        self
    }

    pub(crate) const fn with_power_state(mut self, state: PowerState) -> Self {
        self.power_state = state;
        self
    }
}

/// Failure decoding a configuration register.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ConfigDecodeError {
    /// A reserved bit was observed set.
    ReservedBits {
        /// Reserved bits that were observed set.
        observed: u16,
    },
    /// An undocumented integration-time encoding was observed.
    ReservedIntegrationTime {
        /// Undocumented integration-time field value.
        observed: u16,
    },
}

pub(crate) struct ConfigWord(u16);

impl ConfigWord {
    pub(crate) const fn from_raw(raw: u16) -> Self {
        Self(raw)
    }

    pub(crate) const fn from_snapshot(snapshot: ConfigurationSnapshot) -> Self {
        Self(snapshot.encode())
    }

    pub(crate) const fn raw(self) -> u16 {
        self.0
    }

    pub(crate) fn decode(self) -> Result<ConfigurationSnapshot, ConfigDecodeError> {
        let reserved = self.0 & 0b1110_0100_0000_1100;
        if reserved != 0 {
            return Err(ConfigDecodeError::ReservedBits { observed: reserved });
        }
        Ok(ConfigurationSnapshot {
            measurement: MeasurementConfig::new(
                Gain::from_bits(self.0),
                IntegrationTime::from_bits(self.0)?,
            ),
            persistence: Persistence::from_bits(self.0),
            threshold_monitor: ThresholdMonitorState::from_word(self.0),
            power_state: PowerState::from_word(self.0),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_word_decodes() {
        assert_eq!(
            ConfigWord(0x0001).decode(),
            Ok(ConfigurationSnapshot::silicon_reset_default())
        );
    }

    #[test]
    fn every_documented_configuration_field_combination_round_trips() {
        let gains = [Gain::X1, Gain::X2, Gain::Div8, Gain::Div4];
        let times = [
            IntegrationTime::Ms25,
            IntegrationTime::Ms50,
            IntegrationTime::Ms100,
            IntegrationTime::Ms200,
            IntegrationTime::Ms400,
            IntegrationTime::Ms800,
        ];
        let persistence_values = [
            Persistence::One,
            Persistence::Two,
            Persistence::Four,
            Persistence::Eight,
        ];
        let monitor_states = [
            ThresholdMonitorState::Disabled,
            ThresholdMonitorState::Enabled,
        ];
        let power_states = [PowerState::Active, PowerState::Shutdown];

        for gain in gains {
            for integration_time in times {
                for persistence in persistence_values {
                    for threshold_monitor in monitor_states {
                        for power_state in power_states {
                            let expected = ConfigurationSnapshot {
                                measurement: MeasurementConfig::new(gain, integration_time),
                                persistence,
                                threshold_monitor,
                                power_state,
                            };
                            assert_eq!(ConfigWord(expected.encode()).decode(), Ok(expected));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn every_reserved_configuration_bit_is_rejected() {
        for bit in [2_u32, 3, 10, 13, 14, 15] {
            let raw = 1_u16 << bit;
            assert_eq!(
                ConfigWord(raw).decode(),
                Err(ConfigDecodeError::ReservedBits { observed: raw })
            );
        }
    }

    #[test]
    fn every_reserved_integration_encoding_is_rejected() {
        for observed in [4_u16, 5, 6, 7, 9, 10, 11, 13, 14, 15] {
            assert_eq!(
                ConfigWord(observed << 6).decode(),
                Err(ConfigDecodeError::ReservedIntegrationTime { observed })
            );
        }
    }

    #[test]
    fn public_configuration_accessors_match_the_selected_domain() {
        let config = MeasurementConfig::new(Gain::X2, IntegrationTime::Ms800);
        assert_eq!(config.gain(), Gain::X2);
        assert_eq!(config.integration_time(), IntegrationTime::Ms800);
        assert_eq!(IntegrationTime::Ms25.milliseconds(), 25);
        assert_eq!(IntegrationTime::Ms800.milliseconds(), 800);
        assert_eq!(Persistence::One.count(), 1);
        assert_eq!(Persistence::Eight.count(), 8);
    }
}
