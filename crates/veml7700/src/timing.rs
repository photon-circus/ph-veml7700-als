//! Driver timing policy reacting to shared evidence `S-23`, `S-24`, and `S-55`.

use crate::config::IntegrationTime;

/// Minimum wake-up delay before measurement timing begins (`S-23`).
pub const WAKE_UP_DELAY_US: u32 = 2_500;
/// Integration-time tolerance magnitude used by this driver's policy (`S-24`,
/// `S-55`).
///
/// # This is design guidance, not a characterized guarantee
///
/// The 130% wait is this driver's reaction to the guidance in `S-24`; `S-55`
/// keeps the corresponding silicon proposition explicitly undefined. This is
/// not a characterized worst-case bound. If the real spread is wider, the
/// driver can read a retained value before the new conversion completes.
pub const INTEGRATION_TOLERANCE_PERCENT: u32 = 30;
/// Additional software margin beyond wake-up and maximum integration time.
///
/// This is a **driver policy value**, not evidence recorded in the shared
/// registry. It keeps the requested wait beyond the computed guidance boundary
/// but does not turn that guidance into a characterized silicon bound. The wait
/// is a lower-bound request, not measured elapsed time; see
/// [`MeasurementCapture::requested_wait_us`](crate::MeasurementCapture::requested_wait_us).
pub const MEASUREMENT_MARGIN_US: u32 = 1_000;

/// Timing requested by a complete one-shot measurement.
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
    /// minimum; this type cannot represent a shortened policy wait.
    ///
    /// Lengthening does not convert the `S-24` guidance or undefined `S-55`
    /// device proposition into a guarantee. A
    /// caller who expects a wider real spread can add margin here, but no margin
    /// makes the conversion time characterized.
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
