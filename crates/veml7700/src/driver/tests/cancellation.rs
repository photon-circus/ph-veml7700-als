//! Cancellation boundaries: what a dropped future leaves behind.

use super::*;

#[test]
fn dropping_fresh_capture_at_any_transport_boundary_issues_exactly_that_prefix() {
    let total = fresh_capture_script().len();
    for boundary in 0..total {
        let bus = PendingAt::new(ScriptedI2c::new(fresh_capture_script()), boundary);
        let mut delay = CancellableDelay::ready();
        let mut sensor = Veml7700::new(bus);
        let polled = poll_once_then_drop(sensor.measure_once(
            &mut delay,
            MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms100),
        ));
        assert!(
            polled.is_pending(),
            "boundary {boundary} should have parked the operation"
        );
        let remaining = sensor.release().into_inner().remaining();
        assert_eq!(
            total - remaining,
            boundary,
            "boundary {boundary} issued the wrong number of transactions"
        );
    }
}

#[test]
fn dropping_fresh_capture_during_the_measurement_delay_leaves_the_device_shut_down() {
    // The delay is the longest suspension and the one most likely to be
    // cancelled across. Five transactions precede it; the sixth would be the
    // freeze write, so a caller that drops here leaves the device *active*
    // in the temporary measurement domain with power saving disabled.
    let bus = ScriptedI2c::new(fresh_capture_script());
    let mut delay = CancellableDelay::parking();
    let mut sensor = Veml7700::new(bus);
    let polled = poll_once_then_drop(sensor.measure_once(
        &mut delay,
        MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms100),
    ));
    assert!(polled.is_pending());
    let remaining = sensor.release().remaining();
    assert_eq!(
        remaining, 5,
        "the delay should follow exactly the five domain-installing transactions"
    );
    assert_eq!(delay.elapsed_ns(), 0, "a parked delay must not record time");
}

#[test]
fn dropping_a_threshold_arm_at_any_boundary_issues_exactly_that_prefix() {
    let thresholds =
        Thresholds::new(AlsCounts::from_counts(100), AlsCounts::from_counts(1_000)).unwrap();
    let script = || {
        [
            read_word(0x00, 0x0001),
            write_word(0x00, 0x0001, Ok(())),
            write_word(0x02, 100, Ok(())),
            write_word(0x01, 1_000, Ok(())),
            write_word(0x03, 0x0003, Ok(())),
            write_word(0x00, 0x1022, Ok(())),
        ]
    };
    let total = script().len();
    for boundary in 0..total {
        let bus = PendingAt::new(ScriptedI2c::new(script()), boundary);
        let mut sensor = Veml7700::new(bus);
        let polled =
            poll_once_then_drop(sensor.arm_threshold_monitor(ThresholdMonitorConfig::new(
                MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms100),
                thresholds,
                Persistence::Four,
                PowerSavingConfig::new(true, PowerSavingMode::Mode2),
            )));
        assert!(
            polled.is_pending(),
            "boundary {boundary} should have parked the operation"
        );
        let remaining = sensor.release().into_inner().remaining();
        assert_eq!(
            total - remaining,
            boundary,
            "boundary {boundary} issued the wrong number of transactions"
        );
    }
}

#[test]
fn dropping_an_active_start_covers_the_conditional_shutdown_boundaries() {
    // The scripts above start shut down, where `EnterShutdown` is skipped
    // entirely — so on their own they do not reach the boundary that only
    // exists on an active start. These do.
    let fresh_from_active = [
        read_word(0x00, 0x0000),
        read_word(0x03, 0x0000),
        write_word(0x00, 0x0001, Ok(())), // EnterShutdown
        write_word(0x03, 0x0000, Ok(())),
        write_word(0x00, 0x1001, Ok(())),
        write_word(0x00, 0x1000, Ok(())),
        write_word(0x00, 0x1001, Ok(())),
        read_word(0x04, 0x1234),
        read_word(0x05, 0x5678),
        write_word(0x03, 0x0000, Ok(())),
        write_word(0x00, 0x0000, Ok(())),
    ];
    let total = fresh_from_active.len();
    for boundary in 0..total {
        let bus = PendingAt::new(ScriptedI2c::new(fresh_from_active.clone()), boundary);
        let mut delay = CancellableDelay::ready();
        let mut sensor = Veml7700::new(bus);
        let polled = poll_once_then_drop(sensor.measure_once(
            &mut delay,
            MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms100),
        ));
        assert!(polled.is_pending(), "active-start boundary {boundary}");
        let remaining = sensor.release().into_inner().remaining();
        assert_eq!(
            total - remaining,
            boundary,
            "active-start boundary {boundary}"
        );
    }

    // Re-arming an enabled monitor on an active device is the only threshold
    // path with its own `EnterShutdown` write.
    let thresholds =
        Thresholds::new(AlsCounts::from_counts(100), AlsCounts::from_counts(1_000)).unwrap();
    let arm_from_active_enabled = [
        read_word(0x00, 0x0002),
        write_word(0x00, 0x0003, Ok(())), // EnterShutdown, monitor still enabled
        write_word(0x00, 0x0001, Ok(())), // DisableMonitor, now shut down
        write_word(0x02, 100, Ok(())),
        write_word(0x01, 1_000, Ok(())),
        write_word(0x03, 0x0003, Ok(())),
        write_word(0x00, 0x1022, Ok(())),
    ];
    let total = arm_from_active_enabled.len();
    for boundary in 0..total {
        let bus = PendingAt::new(ScriptedI2c::new(arm_from_active_enabled.clone()), boundary);
        let mut sensor = Veml7700::new(bus);
        let polled =
            poll_once_then_drop(sensor.arm_threshold_monitor(ThresholdMonitorConfig::new(
                MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms100),
                thresholds,
                Persistence::Four,
                PowerSavingConfig::new(true, PowerSavingMode::Mode2),
            )));
        assert!(polled.is_pending(), "re-arm boundary {boundary}");
        let remaining = sensor.release().into_inner().remaining();
        assert_eq!(total - remaining, boundary, "re-arm boundary {boundary}");
    }
}

#[test]
fn a_completed_capture_is_unaffected_by_the_cancellation_harness() {
    // The same harness with an out-of-range boundary never parks, so a
    // passing cancellation test cannot be an artefact of the wrapper
    // swallowing transactions.
    let bus = PendingAt::new(ScriptedI2c::new(fresh_capture_script()), usize::MAX);
    let mut delay = CancellableDelay::ready();
    let mut sensor = Veml7700::new(bus);
    let sample = block_on(sensor.measure_once(
        &mut delay,
        MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms100),
    ))
    .unwrap();
    assert_eq!(sample.als, AlsCounts::from_counts(0x1234));
    sensor.release().into_inner().done();
}
