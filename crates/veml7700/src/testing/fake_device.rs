//! Exploratory sketch of autonomous VEML7700 state.
//!
//! This is **not** the device behavioral model; that is
//! `ph-veml7700-als-model`, which implements the I²C boundary and is derived
//! independently of the driver. This type imports driver semantic types and
//! timing constants, and the tests below drive it directly rather than through
//! `Veml7700`, so they establish nothing about the driver. See
//! `docs/TEST_PLAN.md` Level 3 and issue #9.

#![allow(dead_code)]

use crate::config::{ConfigurationSnapshot, PowerState, ThresholdMonitorState};
use crate::measurement::{AlsCounts, WhiteCounts};
use crate::power::{PowerSavingConfig, PowerSavingSnapshot};
use crate::threshold::{ThresholdStatus, Thresholds};
use crate::timing::{INTEGRATION_TOLERANCE_PERCENT, WAKE_UP_DELAY_US};

pub(crate) struct FakeVeml7700 {
    pub(crate) configuration: ConfigurationSnapshot,
    pub(crate) power_saving: PowerSavingSnapshot,
    pub(crate) thresholds: Thresholds,
    pub(crate) threshold_status: ThresholdStatus,
    pub(crate) als: AlsCounts,
    pub(crate) white: WhiteCounts,
    pub(crate) now_us: u64,
    pub(crate) next_als_refresh_us: Option<u64>,
    pub(crate) next_white_refresh_us: Option<u64>,
    source_als: AlsCounts,
    source_white: WhiteCounts,
    white_phase_offset_us: u64,
    low_streak: u8,
    high_streak: u8,
}

impl FakeVeml7700 {
    pub(crate) fn new() -> Self {
        Self {
            configuration: ConfigurationSnapshot::silicon_reset_default(),
            power_saving: PowerSavingSnapshot {
                enabled: false,
                mode: crate::PowerSavingMode::Mode1,
            },
            thresholds: Thresholds::new(
                AlsCounts::from_counts(0),
                AlsCounts::from_counts(u16::MAX),
            )
            .unwrap(),
            threshold_status: ThresholdStatus {
                low: false,
                high: false,
            },
            als: AlsCounts::from_counts(0),
            white: WhiteCounts::from_counts(0),
            now_us: 0,
            next_als_refresh_us: None,
            next_white_refresh_us: None,
            source_als: AlsCounts::from_counts(0),
            source_white: WhiteCounts::from_counts(0),
            white_phase_offset_us: 0,
            low_streak: 0,
            high_streak: 0,
        }
    }

    pub(crate) fn set_source(&mut self, als: u16, white: u16) {
        self.source_als = AlsCounts::from_counts(als);
        self.source_white = WhiteCounts::from_counts(white);
    }

    pub(crate) fn set_white_phase_offset_us(&mut self, offset_us: u64) {
        self.white_phase_offset_us = offset_us;
    }

    pub(crate) fn apply_configuration(&mut self, next: ConfigurationSnapshot) {
        let wake_edge = self.configuration.power_state == PowerState::Shutdown
            && next.power_state == PowerState::Active;
        self.configuration = next;
        if next.power_state == PowerState::Shutdown {
            self.next_als_refresh_us = None;
            self.next_white_refresh_us = None;
        } else if wake_edge {
            let deadline = self.now_us.saturating_add(self.initial_conversion_us());
            self.next_als_refresh_us = Some(deadline);
            self.next_white_refresh_us = Some(deadline.saturating_add(self.white_phase_offset_us));
        }
    }

    pub(crate) fn apply_power_saving(&mut self, config: PowerSavingConfig) {
        self.power_saving = PowerSavingSnapshot {
            enabled: config.enabled,
            mode: config.mode,
        };
    }

    pub(crate) fn advance_us(&mut self, delta: u64) {
        let target = self.now_us.saturating_add(delta);
        while let Some(deadline) = self.next_event_us().filter(|deadline| *deadline <= target) {
            self.now_us = deadline;
            if self.next_als_refresh_us == Some(deadline) {
                self.complete_als_measurement();
            }
            if self.next_white_refresh_us == Some(deadline) {
                self.complete_white_measurement();
            }
        }
        self.now_us = target;
    }

    pub(crate) fn integration_window_us(&self) -> (u64, u64) {
        let nominal = u64::from(
            self.configuration
                .measurement
                .integration_time()
                .milliseconds(),
        ) * 1_000;
        let tolerance = nominal * u64::from(INTEGRATION_TOLERANCE_PERCENT) / 100;
        (nominal - tolerance, nominal + tolerance)
    }

    fn initial_conversion_us(&self) -> u64 {
        let (_, latest) = self.integration_window_us();
        u64::from(WAKE_UP_DELAY_US) + latest
    }

    fn next_event_us(&self) -> Option<u64> {
        match (self.next_als_refresh_us, self.next_white_refresh_us) {
            (Some(als), Some(white)) => Some(als.min(white)),
            (Some(als), None) => Some(als),
            (None, Some(white)) => Some(white),
            (None, None) => None,
        }
    }

    fn next_refresh_interval_us(&self) -> u64 {
        if self.power_saving.enabled {
            self.power_saving
                .mode
                .nominal_refresh_time_ms(self.configuration.measurement.integration_time())
                .map_or_else(
                    || self.integration_window_us().1,
                    |milliseconds| u64::from(milliseconds) * 1_000,
                )
        } else {
            self.integration_window_us().1
        }
    }

    fn complete_als_measurement(&mut self) {
        self.als = self.source_als;
        self.update_threshold_status();
        self.next_als_refresh_us = self.active_next_refresh();
    }

    fn complete_white_measurement(&mut self) {
        self.white = self.source_white;
        self.next_white_refresh_us = self.active_next_refresh();
    }

    fn active_next_refresh(&self) -> Option<u64> {
        (self.configuration.power_state == PowerState::Active)
            .then(|| self.now_us.saturating_add(self.next_refresh_interval_us()))
    }

    fn update_threshold_status(&mut self) {
        if self.configuration.threshold_monitor == ThresholdMonitorState::Disabled {
            self.low_streak = 0;
            self.high_streak = 0;
            return;
        }
        let counts = self.als.counts();
        self.low_streak = if counts < self.thresholds.low().counts() {
            self.low_streak.saturating_add(1)
        } else {
            0
        };
        self.high_streak = if counts > self.thresholds.high().counts() {
            self.high_streak.saturating_add(1)
        } else {
            0
        };
        let required = self.configuration.persistence.count();
        self.threshold_status.low |= self.low_streak >= required;
        self.threshold_status.high |= self.high_streak >= required;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Gain, IntegrationTime, MeasurementConfig, Persistence, PowerSavingMode};

    fn active_configuration() -> ConfigurationSnapshot {
        ConfigurationSnapshot {
            measurement: MeasurementConfig::new(Gain::X1, IntegrationTime::Ms100),
            persistence: Persistence::One,
            threshold_monitor: ThresholdMonitorState::Disabled,
            power_state: PowerState::Active,
        }
    }

    #[test]
    fn wake_edge_obeys_the_latest_tolerance_bound_and_shutdown_retains_data() {
        let mut fake = FakeVeml7700::new();
        fake.set_source(123, 456);
        assert_eq!(fake.integration_window_us(), (70_000, 130_000));
        fake.apply_configuration(active_configuration());
        fake.advance_us(132_499);
        assert_eq!(fake.als.counts(), 0);
        fake.advance_us(1);
        assert_eq!(fake.als.counts(), 123);
        assert_eq!(fake.white.counts(), 456);

        fake.apply_configuration(ConfigurationSnapshot {
            power_state: PowerState::Shutdown,
            ..active_configuration()
        });
        fake.set_source(999, 999);
        fake.advance_us(1_000_000);
        assert_eq!(fake.als.counts(), 123);
        assert_eq!(fake.white.counts(), 456);
    }

    #[test]
    fn power_saving_uses_the_documented_refresh_cadence() {
        let mut fake = FakeVeml7700::new();
        fake.set_source(1, 1);
        fake.apply_power_saving(PowerSavingConfig::new(true, PowerSavingMode::Mode2));
        fake.apply_configuration(active_configuration());
        fake.advance_us(132_500);
        fake.set_source(2, 2);
        fake.advance_us(1_099_999);
        assert_eq!(fake.als.counts(), 1);
        fake.advance_us(1);
        assert_eq!(fake.als.counts(), 2);
    }

    #[test]
    fn threshold_status_obeys_persistence() {
        let mut fake = FakeVeml7700::new();
        fake.thresholds =
            Thresholds::new(AlsCounts::from_counts(100), AlsCounts::from_counts(200)).unwrap();
        fake.set_source(250, 0);
        fake.apply_configuration(ConfigurationSnapshot {
            persistence: Persistence::Four,
            threshold_monitor: ThresholdMonitorState::Enabled,
            ..active_configuration()
        });
        fake.advance_us(132_500);
        for _ in 0..2 {
            fake.advance_us(130_000);
            assert!(!fake.threshold_status.high);
        }
        fake.advance_us(130_000);
        assert!(fake.threshold_status.high);
    }

    #[test]
    fn channels_can_refresh_at_different_times() {
        let mut fake = FakeVeml7700::new();
        fake.set_source(10, 20);
        fake.set_white_phase_offset_us(10);
        fake.apply_configuration(active_configuration());
        fake.advance_us(132_500);
        assert_eq!(fake.als.counts(), 10);
        assert_eq!(fake.white.counts(), 0);
        fake.advance_us(10);
        assert_eq!(fake.white.counts(), 20);
    }
}
