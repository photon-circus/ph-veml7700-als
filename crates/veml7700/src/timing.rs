//! Named timing policy derived from vendor documentation.

use crate::config::IntegrationTime;

/// Minimum wake-up delay before measurement timing begins.
pub const WAKE_UP_DELAY_US: u32 = 2_500;
/// Maximum documented integration-time tolerance magnitude.
pub const INTEGRATION_TOLERANCE_PERCENT: u32 = 30;
/// Additional software margin beyond wake-up and maximum integration time.
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
            integration_us: nominal_us
                + nominal_us * INTEGRATION_TOLERANCE_PERCENT / 100,
            margin_us: MEASUREMENT_MARGIN_US,
        }
    }

    /// Construct conservative timing plus an additional caller-selected margin.
    ///
    /// The resulting timing can only be equal to or longer than the documented
    /// conservative minimum; this type cannot represent a shortened fresh wait.
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
        let extended = MeasurementTiming::with_additional_margin_us(
            IntegrationTime::Ms100,
            10_000,
        );
        assert_eq!(base.total_us(), 133_500);
        assert_eq!(extended.total_us(), 143_500);
        assert_eq!(extended.wake_up_us(), WAKE_UP_DELAY_US);
    }
}
