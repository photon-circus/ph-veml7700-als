//! Named timing policy derived from vendor documentation.

use crate::config::IntegrationTime;

/// Minimum wake-up delay before measurement timing begins.
pub const WAKE_UP_DELAY_US: u32 = 2_500;
/// Assumed maximum integration-time tolerance magnitude.
///
/// # This is an assumption about silicon, not a documented figure
///
/// No vendor document states an integration-time tolerance, and none is
/// expected to. Integration intervals are counted off the part's internal
/// oscillator, so their spread is that oscillator's tolerance — a
/// process-dependent silicon characteristic rather than a specified timing
/// parameter. Vishay publishes no oscillator accuracy for this part.
///
/// 30 % is a conservative stand-in for that unpublished tolerance, third-party
/// in origin and not adopted as source-backed. It is why a conservative wait is
/// 130 % of the selected integration time, which makes that margin conservative
/// *given this assumption* rather than in general.
///
/// If the real spread is wider, the driver can read an output register before
/// the conversion behind it has completed, and the freshness guarantee fails
/// silently — a stale value is indistinguishable from a new one.
///
/// `docs/HARDWARE_CONTRACT.md` §7 records this as an **Assumption** under D-029,
/// with the measurement that would settle it.
pub const INTEGRATION_TOLERANCE_PERCENT: u32 = 30;
/// Additional software margin beyond wake-up and maximum integration time.
///
/// # Why 1 ms, and what it does not cover
///
/// This is a **driver policy value, not a source-derived one.** No vendor
/// document specifies it. It exists so the total wait does not land exactly on
/// the computed worst-case boundary, where a value equal to the bound is
/// indistinguishable from one just past it.
///
/// 1 ms was chosen as the smallest round figure that is negligible against the
/// shortest integration time — 4 % of 25 ms, and under 0.1 % of 800 ms — so it
/// costs nothing measurable while removing exact-boundary equality. A larger
/// margin would buy no additional correctness, because the real uncertainty is
/// already carried by [`INTEGRATION_TOLERANCE_PERCENT`].
///
/// It does **not** cover, and must not be relied on for:
///
/// - integration-time error beyond the assumed ±30 %, which is what
///   [`INTEGRATION_TOLERANCE_PERCENT`] is for;
/// - I²C transaction time, which is the caller's bus speed and is unbounded from
///   this driver's perspective;
/// - executor scheduling latency, which
///   `embedded_hal_async::delay::DelayNs` may add without limit; or
/// - any silicon behavior. Waiting longer cannot make an undocumented
///   conversion time documented.
///
/// The wait is a lower bound on request, never a guarantee about elapsed time —
/// see
/// [`FreshMeasurement::requested_wait_us`](crate::FreshMeasurement::requested_wait_us).
pub const MEASUREMENT_MARGIN_US: u32 = 1_000;

/// Timing applied by a complete fresh measurement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MeasurementTiming {
    integration_time: IntegrationTime,
    wake_up_us: u32,
    integration_us: u32,
    margin_us: u32,
}

impl MeasurementTiming {
    /// Construct the default conservative timing for an integration-time setting.
    pub const fn conservative(integration_time: IntegrationTime) -> Self {
        let nominal_us = integration_time.milliseconds() * 1_000;
        Self {
            integration_time,
            wake_up_us: WAKE_UP_DELAY_US,
            integration_us: nominal_us + nominal_us * INTEGRATION_TOLERANCE_PERCENT / 100,
            margin_us: MEASUREMENT_MARGIN_US,
        }
    }

    /// Construct conservative timing plus an additional caller-selected margin.
    ///
    /// The resulting timing can only be equal to or longer than the conservative
    /// minimum; this type cannot represent a shortened fresh wait.
    ///
    /// Lengthening does not convert an assumption into a guarantee. The minimum
    /// is partly built on [`INTEGRATION_TOLERANCE_PERCENT`], so a caller who
    /// suspects a wider real spread can add margin here — but no margin makes
    /// the conversion time source-specified.
    pub const fn with_additional_margin_us(
        integration_time: IntegrationTime,
        additional_margin_us: u32,
    ) -> Self {
        let base = Self::conservative(integration_time);
        Self {
            integration_time: base.integration_time,
            wake_up_us: base.wake_up_us,
            integration_us: base.integration_us,
            margin_us: base.margin_us.saturating_add(additional_margin_us),
        }
    }

    /// Return the integration-time selection this timing was derived for.
    pub const fn integration_time(self) -> IntegrationTime {
        self.integration_time
    }

    /// Return the wake-up delay in microseconds.
    pub const fn wake_up_us(self) -> u32 {
        self.wake_up_us
    }

    /// Return the conservative integration interval in microseconds.
    pub const fn integration_us(self) -> u32 {
        self.integration_us
    }

    /// Return the software margin in microseconds.
    pub const fn margin_us(self) -> u32 {
        self.margin_us
    }

    /// Return the total delay in microseconds, saturating for extreme margins.
    pub const fn total_us(self) -> u32 {
        self.wake_up_us
            .saturating_add(self.integration_us)
            .saturating_add(self.margin_us)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_timing_can_only_extend_the_conservative_wait() {
        let base = MeasurementTiming::conservative(IntegrationTime::Ms100);
        let extended = MeasurementTiming::with_additional_margin_us(IntegrationTime::Ms100, 10_000);
        assert_eq!(base.total_us(), 133_500);
        assert_eq!(extended.total_us(), 143_500);
        assert_eq!(extended.wake_up_us(), WAKE_UP_DELAY_US);
    }

    #[test]
    fn every_integration_time_has_conservative_provenance() {
        for (integration_time, integration_us, total_us) in [
            (IntegrationTime::Ms25, 32_500, 36_000),
            (IntegrationTime::Ms50, 65_000, 68_500),
            (IntegrationTime::Ms100, 130_000, 133_500),
            (IntegrationTime::Ms200, 260_000, 263_500),
            (IntegrationTime::Ms400, 520_000, 523_500),
            (IntegrationTime::Ms800, 1_040_000, 1_043_500),
        ] {
            let timing = MeasurementTiming::conservative(integration_time);
            assert_eq!(timing.integration_time(), integration_time);
            assert_eq!(timing.wake_up_us(), WAKE_UP_DELAY_US);
            assert_eq!(timing.integration_us(), integration_us);
            assert_eq!(timing.margin_us(), MEASUREMENT_MARGIN_US);
            assert_eq!(timing.total_us(), total_us);
        }
    }

    #[test]
    fn extreme_additional_margin_saturates_without_shortening() {
        let timing = MeasurementTiming::with_additional_margin_us(IntegrationTime::Ms800, u32::MAX);
        assert_eq!(timing.margin_us(), u32::MAX);
        assert_eq!(timing.total_us(), u32::MAX);
    }
}
