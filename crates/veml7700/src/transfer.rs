//! Source-backed VEML7700 transfer description.
//!
//! Integer and string data only: no floating point, no generated knots, no
//! runtime correction. The family TOML is the smoke-test payload for a future
//! `ph-curves` generator; see `exploration/ph-curves-transfer/`.

#![deny(clippy::float_arithmetic)]
#![cfg_attr(not(test), allow(dead_code))]

use crate::config::{Gain, IntegrationTime, MeasurementConfig};
use crate::illuminance::NominalScale;

/// Vendor polynomial coefficients as written in the application note.
///
/// Stored as source decimal strings so this module never contains `f32`/`f64`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VendorPolynomial {
    /// `a` in `a·u⁴ + b·u³ + c·u² + d·u` (`6.0135e-13`).
    pub a: &'static str,
    /// `b` (`-9.3924e-9`).
    pub b: &'static str,
    /// `c` (`8.1488e-5`).
    pub c: &'static str,
    /// `d` (`1.0023`).
    pub d: &'static str,
}

/// Why a mapping is not generated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferGapStatus {
    /// The sources do not define this mapping.
    Undefined,
}

/// A channel or procedure that must not become a transfer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransferGap {
    /// Stable gap name (`white_channel`, `ir_lux_optimization`).
    pub name: &'static str,
    /// Always [`TransferGapStatus::Undefined`].
    pub status: TransferGapStatus,
    /// Why the sources do not support a transfer here.
    pub reason: &'static str,
}

/// Whether the vendor polynomial applies to this gain × integration pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorrectionApplicability {
    /// Gain ×1/4 or ×1/8: the formula should be used; generate this member.
    Required,
    /// Gain ×1 or ×2: sources confine these to below 100 lx; do not generate
    /// a high-lux table.
    DoNotUseAbove100Lx,
}

/// One of the twenty-four verified resolution pairs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AlsTransferMember {
    /// Analog gain.
    pub gain: Gain,
    /// Integration time.
    pub integration_time: IntegrationTime,
    /// Whether this pair is emitted in the family TOML.
    pub correction: CorrectionApplicability,
}

impl AlsTransferMember {
    /// Nominal scale from the same table `nominal_illuminance` uses.
    pub const fn scale(self) -> NominalScale {
        NominalScale::for_config(MeasurementConfig::new(self.gain, self.integration_time))
    }

    /// Gain encoding bits 12:11, for the encoding-order test.
    pub const fn gain_encoding(self) -> u16 {
        match self.gain {
            Gain::X1 => 0b00,
            Gain::X2 => 0b01,
            Gain::Div8 => 0b10,
            Gain::Div4 => 0b11,
        }
    }
}

/// Complete device transfer description: tables, polynomial, applicability, gaps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Veml7700TransferDescription {
    /// Vendor polynomial as source decimals.
    pub polynomial: VendorPolynomial,
    /// All twenty-four gain × integration pairs.
    pub members: &'static [AlsTransferMember],
    /// Declared non-transfers.
    pub gaps: &'static [TransferGap],
    /// Worked-example ALS counts (5581).
    pub example_counts: u16,
    /// Worked-example uncorrected lux (1500).
    pub example_uncorrected_lux: u32,
    /// Worked-example corrected lux (1658).
    pub example_corrected_lux: u32,
    /// Uncorrected-lux generation window for `correction = required` members.
    pub generated_uncorrected_lux: [u32; 2],
}

/// Source-backed description. Matches `HARDWARE_CONTRACT.md` §8.
pub const DESCRIPTION: Veml7700TransferDescription = Veml7700TransferDescription {
    polynomial: VendorPolynomial {
        a: "6.0135e-13",
        b: "-9.3924e-9",
        c: "8.1488e-5",
        d: "1.0023",
    },
    members: &MEMBERS,
    gaps: &[
        TransferGap {
            name: "white_channel",
            status: TransferGapStatus::Undefined,
            reason: "sources give 16-bit counts only; no lux scale",
        },
        TransferGap {
            name: "ir_lux_optimization",
            status: TransferGapStatus::Undefined,
            reason: "AN states white/ALS ratio ≳ 2 for IR-rich sources; no conversion formula",
        },
    ],
    example_counts: 5581,
    example_uncorrected_lux: 1500,
    example_corrected_lux: 1658,
    generated_uncorrected_lux: [100, 22_000],
};

/// Family TOML consumed by a future `ph-curves` generator.
pub const PH_CURVES_FAMILY_TOML: &str = include_str!("transfer.toml");

const fn member(gain: Gain, integration_time: IntegrationTime) -> AlsTransferMember {
    let correction = match gain {
        Gain::Div4 | Gain::Div8 => CorrectionApplicability::Required,
        Gain::X1 | Gain::X2 => CorrectionApplicability::DoNotUseAbove100Lx,
    };
    AlsTransferMember {
        gain,
        integration_time,
        correction,
    }
}

const MEMBERS: [AlsTransferMember; 24] = [
    member(Gain::X2, IntegrationTime::Ms800),
    member(Gain::X1, IntegrationTime::Ms800),
    member(Gain::Div4, IntegrationTime::Ms800),
    member(Gain::Div8, IntegrationTime::Ms800),
    member(Gain::X2, IntegrationTime::Ms400),
    member(Gain::X1, IntegrationTime::Ms400),
    member(Gain::Div4, IntegrationTime::Ms400),
    member(Gain::Div8, IntegrationTime::Ms400),
    member(Gain::X2, IntegrationTime::Ms200),
    member(Gain::X1, IntegrationTime::Ms200),
    member(Gain::Div4, IntegrationTime::Ms200),
    member(Gain::Div8, IntegrationTime::Ms200),
    member(Gain::X2, IntegrationTime::Ms100),
    member(Gain::X1, IntegrationTime::Ms100),
    member(Gain::Div4, IntegrationTime::Ms100),
    member(Gain::Div8, IntegrationTime::Ms100),
    member(Gain::X2, IntegrationTime::Ms50),
    member(Gain::X1, IntegrationTime::Ms50),
    member(Gain::Div4, IntegrationTime::Ms50),
    member(Gain::Div8, IntegrationTime::Ms50),
    member(Gain::X2, IntegrationTime::Ms25),
    member(Gain::X1, IntegrationTime::Ms25),
    member(Gain::Div4, IntegrationTime::Ms25),
    member(Gain::Div8, IntegrationTime::Ms25),
];

#[cfg(test)]
#[allow(clippy::float_arithmetic)]
mod tests {
    use super::*;
    use crate::illuminance::MicroLux;

    const GAINS: [Gain; 4] = [Gain::X2, Gain::X1, Gain::Div4, Gain::Div8];
    const INTEGRATION_TIMES: [IntegrationTime; 6] = [
        IntegrationTime::Ms800,
        IntegrationTime::Ms400,
        IntegrationTime::Ms200,
        IntegrationTime::Ms100,
        IntegrationTime::Ms50,
        IntegrationTime::Ms25,
    ];

    #[test]
    fn all_twenty_four_pairs_match_nominal_scale() {
        assert_eq!(DESCRIPTION.members.len(), 24);
        for &gain in &GAINS {
            for &integration_time in &INTEGRATION_TIMES {
                let found = DESCRIPTION
                    .members
                    .iter()
                    .find(|member| {
                        member.gain == gain && member.integration_time == integration_time
                    })
                    .expect("every gain × integration pair is in the description");
                assert_eq!(
                    found.scale().micro_lux_per_count(),
                    NominalScale::for_config(MeasurementConfig::new(gain, integration_time))
                        .micro_lux_per_count()
                );
            }
        }
    }

    #[test]
    fn generated_members_are_the_twelve_correction_required_pairs() {
        let mut generated = 0usize;
        for member in DESCRIPTION.members {
            if member.correction != CorrectionApplicability::Required {
                continue;
            }
            generated += 1;
            assert!(matches!(member.gain, Gain::Div4 | Gain::Div8));
            assert!(
                toml_contains_u32(PH_CURVES_FAMILY_TOML, member.scale().micro_lux_per_count()),
                "family TOML missing scale {} for {:?} {:?}",
                member.scale().micro_lux_per_count(),
                member.gain,
                member.integration_time
            );
        }
        assert_eq!(generated, 12);
    }

    #[test]
    fn encoding_order_is_not_magnitude_order() {
        let div8 = DESCRIPTION
            .members
            .iter()
            .find(|member| {
                member.gain == Gain::Div8 && member.integration_time == IntegrationTime::Ms25
            })
            .unwrap();
        let div4 = DESCRIPTION
            .members
            .iter()
            .find(|member| {
                member.gain == Gain::Div4 && member.integration_time == IntegrationTime::Ms25
            })
            .unwrap();
        assert_eq!(div8.gain_encoding(), 0b10);
        assert_eq!(div4.gain_encoding(), 0b11);
        assert!(div8.gain_encoding() < div4.gain_encoding());
        assert_eq!(div8.scale().micro_lux_per_count(), 2_150_400);
        assert_eq!(div4.scale().micro_lux_per_count(), 1_075_200);
        assert!(div8.scale().micro_lux_per_count() > div4.scale().micro_lux_per_count());
    }

    #[test]
    fn worked_example_uncorrected_lux_is_integer() {
        let scale =
            NominalScale::for_config(MeasurementConfig::new(Gain::Div4, IntegrationTime::Ms100));
        let uncorrected = scale.scale_counts(DESCRIPTION.example_counts);
        assert_eq!(
            MicroLux::whole_lux_floor(uncorrected),
            u64::from(DESCRIPTION.example_uncorrected_lux)
        );
    }

    #[test]
    fn worked_example_corrected_lux_rounds_to_1658() {
        let a: f64 = DESCRIPTION.polynomial.a.parse().unwrap();
        let b: f64 = DESCRIPTION.polynomial.b.parse().unwrap();
        let c: f64 = DESCRIPTION.polynomial.c.parse().unwrap();
        let d: f64 = DESCRIPTION.polynomial.d.parse().unwrap();
        let x = f64::from(DESCRIPTION.example_uncorrected_lux);
        let corrected = ((a * x + b) * x + c) * x * x + d * x;
        assert!((corrected - 1658.0).abs() < 0.5, "{corrected}");
    }

    #[test]
    fn white_channel_is_an_undefined_gap() {
        let white = DESCRIPTION
            .gaps
            .iter()
            .find(|gap| gap.name == "white_channel")
            .expect("white_channel gap");
        assert_eq!(white.status, TransferGapStatus::Undefined);
        assert!(PH_CURVES_FAMILY_TOML.contains("[gaps.white_channel]"));
        assert!(PH_CURVES_FAMILY_TOML.contains("[gaps.ir_lux_optimization]"));
    }

    #[test]
    fn generation_window_is_not_full_scale_adc() {
        assert_eq!(DESCRIPTION.generated_uncorrected_lux, [100, 22_000]);
        let full_scale =
            NominalScale::for_config(MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms25))
                .full_scale_micro_lux()
                .whole_lux_floor();
        assert!(u64::from(DESCRIPTION.generated_uncorrected_lux[1]) < full_scale);
        assert!(!PH_CURVES_FAMILY_TOML.contains("65535"));
        assert!(!PH_CURVES_FAMILY_TOML.contains("[curves"));
    }

    #[test]
    fn family_toml_is_not_a_formula_dump() {
        assert!(PH_CURVES_FAMILY_TOML.contains("kind = \"scaled_polynomial\""));
        assert!(PH_CURVES_FAMILY_TOML.contains("interpolate_selectors = false"));
        assert!(PH_CURVES_FAMILY_TOML.contains("max_knots = 64"));
        assert!(!PH_CURVES_FAMILY_TOML.contains("formula ="));
    }

    fn toml_contains_u32(haystack: &str, value: u32) -> bool {
        let mut buf = [b'0'; 16];
        let mut n = value;
        let mut start = buf.len();
        if n == 0 {
            return haystack.contains('0');
        }
        while n > 0 {
            start -= 1;
            buf[start] = b'0' + (n % 10) as u8;
            n /= 10;
        }
        haystack.contains(core::str::from_utf8(&buf[start..]).unwrap())
    }
}
