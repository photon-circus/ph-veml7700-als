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

/// Threshold persistence protect number (`ALS_PERS`).
///
/// # What this selects, and what it does not promise
///
/// The four values are source-backed: Table 1 defines `ALS_PERS` and its
/// encodings as a *persistence protect number*, and that is what this driver
/// programs.
///
/// **The counting condition is source-backed; assertion timing is not.** The
/// vendor's application note states that a flag is set *only when* the threshold
/// is exceeded and `ALS_PERS` measurements stay above or below it. That makes
/// the condition **necessary**. It does not state that meeting it is
/// sufficient, and it does not say what a measurement that fails to qualify does
/// to a partial run.
///
/// This driver therefore promises nothing about *when*
/// [`read_threshold_status`](crate::Veml7700::read_threshold_status) will report
/// a flag for any value above [`Persistence::One`].
///
/// Poll the status. Do not compute an expected assertion time from the count and
/// the refresh cadence — that calculation needs sufficiency and a reset rule,
/// and the sources give neither.
///
/// `docs/HARDWARE_CONTRACT.md` `S-39` / `S-40` record both halves; D-030 says why the
/// driver stays silent here rather than assuming.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Persistence {
    /// Protect number 1 — one qualifying measurement.
    ///
    /// The only value with no rule ambiguity: there is no sequence to count.
    One,
    /// Protect number 2.
    Two,
    /// Protect number 4.
    Four,
    /// Protect number 8.
    Eight,
}

impl Persistence {
    /// Return the programmed protect number.
    ///
    /// This is the encoded field value, not an input to any timing calculation
    /// the driver performs — nothing in this driver reads it.
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
    ///
    /// This is what the device powers up in, not a recommendation. The sources
    /// confine gain ×1 to illumination below 100 lx, so this domain saturates at
    /// 4 404 lx — well under office daylight. It exists so a caller can name the
    /// reset state, not so they can start from it.
    pub const fn silicon_reset_default() -> Self {
        Self::new(Gain::X1, IntegrationTime::Ms100)
    }

    /// Widest range the part offers: gain ×1/8 and 25 ms, saturating at
    /// ~140 926 lx.
    ///
    /// This is the starting point for unknown brightness. The sources say to
    /// begin at the lowest gain — ×1/8 or ×1/4 — so strong sunlight cannot
    /// overload the sensor, and that an integration time below 100 ms may be
    /// needed to show such a value. Both are recorded in
    /// `docs/HARDWARE_CONTRACT.md` `S-34`.
    ///
    /// The cost is resolution: 2.1504 lx per count, the coarsest the part
    /// offers. Once the ambient range is known, a longer integration time or
    /// higher gain gives a finer reading — see
    /// [`NominalScale::full_scale_micro_lux`](crate::NominalScale::full_scale_micro_lux)
    /// for what each pair reaches.
    ///
    /// # Not usable with power-saving cadence
    ///
    /// The vendor publishes refresh times for 100, 200, 400 and 800 ms only, so
    /// 25 ms has no documented cadence. Pairing this preset with an enabled
    /// [`PowerSavingConfig`](crate::PowerSavingConfig) in
    /// [`arm_threshold_monitor`](crate::Veml7700::arm_threshold_monitor) asks for
    /// behavior no source establishes. Use 100 ms or longer when monitoring with
    /// cadence enabled.
    pub const fn maximum_range_start() -> Self {
        Self::new(Gain::Div8, IntegrationTime::Ms25)
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
    /// This crate's software policy, **not** the device's reset state.
    ///
    /// Returns [`maximum_range_start`](Self::maximum_range_start): the widest
    /// range the part offers, so an unconfigured first measurement has the best
    /// chance of landing on scale. It does **not** make saturation impossible —
    /// light beyond ~140 926 lx still clips, still without an error — so
    /// [`AlsCounts::is_saturated`](crate::AlsCounts::is_saturated) must be
    /// checked regardless of configuration. The device's own reset domain is
    /// [`silicon_reset_default`](Self::silicon_reset_default) and is different —
    /// a caller who wants what the hardware powers up in must ask for it by
    /// name.
    ///
    /// The two are deliberately distinct. Conflating them is how a caller ends
    /// up believing `Default` describes the device.
    fn default() -> Self {
        Self::maximum_range_start()
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

impl core::fmt::Display for ConfigDecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ReservedBits { observed } => {
                write!(f, "reserved configuration bits were set: {observed:#06x}")
            }
            Self::ReservedIntegrationTime { observed } => {
                write!(f, "undocumented integration-time encoding {observed:#06b}")
            }
        }
    }
}

impl core::error::Error for ConfigDecodeError {}

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

    /// Literal words from `docs/HARDWARE_CONTRACT.md` `S-12` / `S-14` / `S-15`, not round trips.
    ///
    /// The exhaustive round-trip test below proves the encoder and decoder agree
    /// with each other. It cannot detect them agreeing on the *wrong* bit
    /// position: shift both fields by one and every round trip still passes.
    /// These vectors are the only tests here that would fail.
    ///
    /// Each field is placed alone so a failure names the field rather than the
    /// word. The encodings deliberately include the two cases where bit order
    /// and magnitude order disagree — gain `10` is ×1/8 while `11` is ×1/4, and
    /// integration `1100` is the *shortest* time — because a plausible-looking
    /// table sorted by magnitude would encode both backwards.
    #[test]
    fn configuration_fields_occupy_the_contract_bit_positions() {
        let base = ConfigurationSnapshot {
            measurement: MeasurementConfig::new(Gain::X1, IntegrationTime::Ms100),
            persistence: Persistence::One,
            threshold_monitor: ThresholdMonitorState::Disabled,
            power_state: PowerState::Active,
        };
        // Every field at its zero encoding is the all-zero word.
        assert_eq!(base.encode(), 0x0000);

        // Gain, bits 12:11.
        for (gain, bits) in [
            (Gain::X1, 0b00_u16),
            (Gain::X2, 0b01),
            (Gain::Div8, 0b10),
            (Gain::Div4, 0b11),
        ] {
            let word = ConfigurationSnapshot {
                measurement: MeasurementConfig::new(gain, IntegrationTime::Ms100),
                ..base
            }
            .encode();
            assert_eq!(word, bits << 11, "gain {gain:?} must occupy bits 12:11");
        }

        // Integration time, bits 9:6.
        for (integration_time, bits) in [
            (IntegrationTime::Ms25, 0b1100_u16),
            (IntegrationTime::Ms50, 0b1000),
            (IntegrationTime::Ms100, 0b0000),
            (IntegrationTime::Ms200, 0b0001),
            (IntegrationTime::Ms400, 0b0010),
            (IntegrationTime::Ms800, 0b0011),
        ] {
            let word = ConfigurationSnapshot {
                measurement: MeasurementConfig::new(Gain::X1, integration_time),
                ..base
            }
            .encode();
            assert_eq!(
                word,
                bits << 6,
                "integration time {integration_time:?} must occupy bits 9:6"
            );
        }

        // Persistence, bits 5:4.
        for (persistence, bits) in [
            (Persistence::One, 0b00_u16),
            (Persistence::Two, 0b01),
            (Persistence::Four, 0b10),
            (Persistence::Eight, 0b11),
        ] {
            let word = ConfigurationSnapshot {
                persistence,
                ..base
            }
            .encode();
            assert_eq!(
                word,
                bits << 4,
                "persistence {persistence:?} must occupy bits 5:4"
            );
        }

        // Monitor enable is bit 1; shutdown is bit 0.
        assert_eq!(
            ConfigurationSnapshot {
                threshold_monitor: ThresholdMonitorState::Enabled,
                ..base
            }
            .encode(),
            1 << 1
        );
        assert_eq!(
            ConfigurationSnapshot {
                power_state: PowerState::Shutdown,
                ..base
            }
            .encode(),
            1 << 0
        );

        // One word carrying every field at once, decoded back. ×1/4 gain,
        // 800 ms, persistence 8, monitor enabled, shut down:
        // 0b0001_1000_1111_0011.
        let combined = (0b11 << 11) | (0b0011 << 6) | (0b11 << 4) | (1 << 1) | 1;
        assert_eq!(combined, 0x18F3);
        assert_eq!(
            ConfigWord(combined).decode(),
            Ok(ConfigurationSnapshot {
                measurement: MeasurementConfig::new(Gain::Div4, IntegrationTime::Ms800),
                persistence: Persistence::Eight,
                threshold_monitor: ThresholdMonitorState::Enabled,
                power_state: PowerState::Shutdown,
            })
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
