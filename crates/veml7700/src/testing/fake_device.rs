//! Autonomous VEML7700 behavioral-model starting point.
//!
//! This is intentionally not a static register array. Milestone 5 expands it to
//! model wake-up, integration completion, shutdown data retention, power-saving
//! cadence, threshold persistence, and independently refreshing ALS/white data.

#![allow(dead_code)]

use crate::config::ConfigurationSnapshot;
use crate::measurement::{AlsCounts, WhiteCounts};
use crate::power::PowerSavingSnapshot;
use crate::threshold::{ThresholdStatus, Thresholds};

pub(crate) struct FakeVeml7700 {
    pub(crate) configuration: ConfigurationSnapshot,
    pub(crate) power_saving: PowerSavingSnapshot,
    pub(crate) thresholds: Thresholds,
    pub(crate) threshold_status: ThresholdStatus,
    pub(crate) als: AlsCounts,
    pub(crate) white: WhiteCounts,
    pub(crate) now_us: u64,
    pub(crate) next_refresh_us: Option<u64>,
}

impl FakeVeml7700 {
    pub(crate) fn advance_us(&mut self, delta: u64) {
        self.now_us = self.now_us.saturating_add(delta);
        if self
            .next_refresh_us
            .is_some_and(|deadline| self.now_us >= deadline)
        {
            self.complete_measurement();
        }
    }

    fn complete_measurement(&mut self) {
        self.next_refresh_us = None;
        let counts = self.als.counts();
        self.threshold_status = ThresholdStatus {
            low: counts < self.thresholds.low.counts(),
            high: counts > self.thresholds.high.counts(),
        };
        // Milestone 5: source model, integration scheduling, persistence counts,
        // PSM cadence, and separate ALS/white refresh phases.
    }
}
