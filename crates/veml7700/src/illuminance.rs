//! Integer nominal illuminance scaling.

use crate::config::{Gain, IntegrationTime, MeasurementConfig};

/// Nominal illuminance represented in micro-lux.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MicroLux(u64);

impl MicroLux {
    /// Construct from a raw micro-lux value.
    pub const fn from_micro_lux(value: u64) -> Self {
        Self(value)
    }

    /// Return the raw micro-lux value.
    pub const fn as_micro_lux(self) -> u64 {
        self.0
    }

    /// Return whole lux, rounded down.
    pub const fn whole_lux_floor(self) -> u64 {
        self.0 / 1_000_000
    }

    /// Return milli-lux rounded to the nearest milli-lux.
    pub const fn milli_lux_rounded(self) -> u64 {
        self.0 / 1_000 + (self.0 % 1_000 + 500) / 1_000
    }
}

/// Nominal scale for one gain/integration-time pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NominalScale {
    micro_lux_per_count: u32,
}

impl NominalScale {
    /// Construct the vendor-table nominal scale for a measurement configuration.
    pub const fn for_config(config: MeasurementConfig) -> Self {
        let base_gain2_800 = 4_200_u32;
        let integration_multiplier = match config.integration_time() {
            IntegrationTime::Ms800 => 1,
            IntegrationTime::Ms400 => 2,
            IntegrationTime::Ms200 => 4,
            IntegrationTime::Ms100 => 8,
            IntegrationTime::Ms50 => 16,
            IntegrationTime::Ms25 => 32,
        };
        let gain_multiplier = match config.gain() {
            Gain::X2 => 1,
            Gain::X1 => 2,
            Gain::Div4 => 8,
            Gain::Div8 => 16,
        };
        Self {
            micro_lux_per_count: base_gain2_800 * integration_multiplier * gain_multiplier,
        }
    }

    /// Return nominal micro-lux per ADC count.
    pub const fn micro_lux_per_count(self) -> u32 {
        self.micro_lux_per_count
    }

    /// Scale an ALS count using the nominal vendor-table ratio.
    pub const fn scale_counts(self, counts: u16) -> MicroLux {
        MicroLux::from_micro_lux(counts as u64 * self.micro_lux_per_count as u64)
    }

    /// Nominal illuminance at maximum ADC code — the point this domain
    /// saturates.
    ///
    /// This is where a reading stops being a measurement. At `u16::MAX` counts
    /// the conversion has clipped: the light exceeded what this domain can
    /// encode, and the reported number is the domain's ceiling rather than an
    /// observation.
    ///
    /// It is **not** a bound on the scene's actual illuminance. This is a
    /// nominal figure — the vendor count-to-lux ratio, uncorrected for the
    /// non-linearity the sources describe above roughly 1 000 lx and never
    /// calibrated for optics or source spectrum — so it bounds nothing outside
    /// its own nominal domain. What a clipped reading establishes is that the
    /// configuration was too narrow, not how much light there was. See
    /// [`AlsCounts::is_saturated`](crate::AlsCounts::is_saturated).
    ///
    /// The spread is four orders of magnitude, which is why the starting
    /// configuration matters more than any other choice a caller makes:
    ///
    /// | Configuration | Full scale |
    /// | --- | --- |
    /// | ×2, 800 ms | ~275 lx |
    /// | ×1, 100 ms — silicon reset | ~4 404 lx |
    /// | ×1/8, 100 ms | ~35 232 lx |
    /// | ×1/8, 25 ms — [`maximum_range_start`](crate::MeasurementConfig::maximum_range_start) | ~140 926 lx |
    ///
    /// Direct sunlight exceeds every row but the last.
    pub const fn full_scale_micro_lux(self) -> MicroLux {
        self.scale_counts(u16::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_documented_gain_and_integration_pairs_match_the_nominal_table() {
        let rows = [
            (IntegrationTime::Ms800, [4_200, 8_400, 33_600, 67_200]),
            (IntegrationTime::Ms400, [8_400, 16_800, 67_200, 134_400]),
            (IntegrationTime::Ms200, [16_800, 33_600, 134_400, 268_800]),
            (IntegrationTime::Ms100, [33_600, 67_200, 268_800, 537_600]),
            (IntegrationTime::Ms50, [67_200, 134_400, 537_600, 1_075_200]),
            (
                IntegrationTime::Ms25,
                [134_400, 268_800, 1_075_200, 2_150_400],
            ),
        ];
        let gains = [Gain::X2, Gain::X1, Gain::Div4, Gain::Div8];

        for (integration_time, expected) in rows {
            for (gain, expected_micro_lux) in gains.into_iter().zip(expected) {
                let scale =
                    NominalScale::for_config(MeasurementConfig::new(gain, integration_time));
                assert_eq!(scale.micro_lux_per_count(), expected_micro_lux);
            }
        }
    }

    #[test]
    fn milli_lux_rounds_to_nearest_and_survives_the_whole_input_range() {
        for (micro_lux, expected_milli_lux) in [
            (0_u64, 0_u64),
            (499, 0),
            (500, 1),
            (1_499, 1),
            (1_500, 2),
            // u64::MAX is 18_446_744_073_709_551_615 µlx: 18_446_744_073_709_551
            // whole milli-lux with a 615 µlx remainder, which rounds up.
            (u64::MAX, 18_446_744_073_709_552),
        ] {
            assert_eq!(
                MicroLux::from_micro_lux(micro_lux).milli_lux_rounded(),
                expected_milli_lux
            );
        }
    }

    #[test]
    fn maximum_nominal_range_fits_u64() {
        let scale =
            NominalScale::for_config(MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms25));
        assert_eq!(
            scale.scale_counts(u16::MAX).as_micro_lux(),
            140926464000_u64
        );
    }
}
