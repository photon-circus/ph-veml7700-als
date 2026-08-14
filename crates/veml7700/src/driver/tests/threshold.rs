//! Threshold-monitor programming and partial commit.

use super::*;

#[test]
fn threshold_monitor_is_enabled_only_after_its_domain_is_programmed() {
    let thresholds =
        Thresholds::new(AlsCounts::from_counts(100), AlsCounts::from_counts(1_000)).unwrap();
    let monitor = ThresholdMonitorConfig::new(
        MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms100),
        thresholds,
        Persistence::Four,
        PowerSavingConfig::new(true, PowerSavingMode::Mode2),
    );
    let bus = ScriptedI2c::new([
        read_word(0x00, 0x0001),
        write_word(0x00, 0x0001, Ok(())),
        write_word(0x02, 100, Ok(())),
        write_word(0x01, 1_000, Ok(())),
        write_word(0x03, 0x0003, Ok(())),
        write_word(0x00, 0x1022, Ok(())),
    ]);
    let mut sensor = Veml7700::new(bus);
    block_on(sensor.arm_threshold_monitor(monitor)).unwrap();
    sensor.release().done();
}

#[test]
fn threshold_monitor_blocks_domain_retarget_before_write() {
    let bus = ScriptedI2c::new([read_word(0x00, 0x0002)]);
    let mut sensor = Veml7700::new(bus);
    let result = block_on(
        sensor.set_measurement_config(MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms100)),
    );
    assert_eq!(
        result,
        Err(Error::Configuration(
            ConfigurationError::ThresholdMonitorOwnsDomain
        ))
    );
    sensor.release().done();
}

#[test]
fn arming_from_active_disables_the_monitor_and_shuts_down_together() {
    let thresholds =
        Thresholds::new(AlsCounts::from_counts(100), AlsCounts::from_counts(1_000)).unwrap();
    let monitor = ThresholdMonitorConfig::new(
        MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms100),
        thresholds,
        Persistence::Four,
        PowerSavingConfig::new(true, PowerSavingMode::Mode2),
    );
    let bus = ScriptedI2c::new([
        read_word(0x00, 0x0000),
        // Monitor-disable and shutdown in one write, still the old domain.
        write_word(0x00, 0x0001, Ok(())),
        write_word(0x02, 100, Ok(())),
        write_word(0x01, 1_000, Ok(())),
        write_word(0x03, 0x0003, Ok(())),
        // The monitored domain is installed and activated last.
        write_word(0x00, 0x1022, Ok(())),
    ]);
    let mut sensor = Veml7700::new(bus);
    block_on(sensor.arm_threshold_monitor(monitor)).unwrap();
    sensor.release().done();
}

#[test]
fn re_arming_an_active_monitor_shuts_down_before_disabling_it() {
    let thresholds =
        Thresholds::new(AlsCounts::from_counts(100), AlsCounts::from_counts(1_000)).unwrap();
    let monitor = ThresholdMonitorConfig::new(
        MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms100),
        thresholds,
        Persistence::Four,
        PowerSavingConfig::new(true, PowerSavingMode::Mode2),
    );
    // 0x0002 is active with the monitor enabled. The shutdown and monitor
    // bits cannot move together, so this is the one starting state needing
    // an extra write.
    let bus = ScriptedI2c::new([
        read_word(0x00, 0x0002),
        // Shutdown with the monitored domain intact: only bit 0 moves.
        write_word(0x00, 0x0003, Ok(())),
        // Monitor disabled while shut down: only bit 1 moves.
        write_word(0x00, 0x0001, Ok(())),
        write_word(0x02, 100, Ok(())),
        write_word(0x01, 1_000, Ok(())),
        write_word(0x03, 0x0003, Ok(())),
        write_word(0x00, 0x1022, Ok(())),
    ]);
    let mut sensor = Veml7700::new(bus);
    block_on(sensor.arm_threshold_monitor(monitor)).unwrap();
    sensor.release().done();
}

#[test]
fn every_threshold_programming_stage_failure_is_identified() {
    let failure = ScriptError::new(ErrorKind::Bus);
    let thresholds =
        Thresholds::new(AlsCounts::from_counts(100), AlsCounts::from_counts(1_000)).unwrap();
    let monitor = ThresholdMonitorConfig::new(
        MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms100),
        thresholds,
        Persistence::Four,
        PowerSavingConfig::new(true, PowerSavingMode::Mode2),
    );
    // The third column is what the driver must report as *confirmed* when
    // that stage fails: the last write that actually returned success. The
    // failing stage is never in it, because its commit status is precisely
    // what an I2C error cannot establish. This starts from shutdown, so
    // `EnterShutdown` is skipped and `DisableMonitor` is the first write.
    let stages = [
        (
            ThresholdMonitorStage::ObserveConfiguration,
            BusContext::ReadConfiguration,
            None,
        ),
        (
            ThresholdMonitorStage::DisableMonitor,
            BusContext::WriteConfiguration,
            None,
        ),
        (
            ThresholdMonitorStage::WriteLowThreshold,
            BusContext::WriteLowThreshold,
            Some(ThresholdMonitorStage::DisableMonitor),
        ),
        (
            ThresholdMonitorStage::WriteHighThreshold,
            BusContext::WriteHighThreshold,
            Some(ThresholdMonitorStage::WriteLowThreshold),
        ),
        (
            ThresholdMonitorStage::ApplyPowerSaving,
            BusContext::WritePowerSaving,
            Some(ThresholdMonitorStage::WriteHighThreshold),
        ),
        (
            ThresholdMonitorStage::EnableMonitor,
            BusContext::WriteConfiguration,
            Some(ThresholdMonitorStage::ApplyPowerSaving),
        ),
    ];
    for (failed_index, (stage, context, confirmed)) in stages.into_iter().enumerate() {
        let mut expectations = vec![];
        if failed_index == 0 {
            expectations.push(read_failure(0x00, failure));
        } else {
            expectations.push(read_word(0x00, 0x0001));
            for index in 1..=failed_index {
                let result = (index != failed_index).then_some(()).ok_or(failure);
                expectations.push(match index {
                    1 => write_word(0x00, 0x0001, result),
                    2 => write_word(0x02, 100, result),
                    3 => write_word(0x01, 1_000, result),
                    4 => write_word(0x03, 0x0003, result),
                    5 => write_word(0x00, 0x1022, result),
                    _ => unreachable!(),
                });
            }
        }
        let mut sensor = Veml7700::new(ScriptedI2c::new(expectations));
        assert_eq!(
            block_on(sensor.arm_threshold_monitor(monitor)),
            Err(ThresholdMonitorError {
                stage,
                confirmed,
                source: Error::Bus {
                    operation: Operation::ThresholdMonitor,
                    context,
                    source: failure,
                },
            })
        );
        sensor.release().done();
    }
}
