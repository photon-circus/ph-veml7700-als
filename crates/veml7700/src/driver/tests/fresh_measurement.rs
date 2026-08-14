//! Fresh capture, its stage failures, and restoration.

use super::*;

#[test]
fn fresh_measurement_uses_known_wake_edge_and_restores_state() {
    let bus = ScriptedI2c::new([
        read_word(0x00, 0x0001),
        read_word(0x03, 0x0000),
        write_word(0x03, 0x0000, Ok(())),
        write_word(0x00, 0x1001, Ok(())),
        write_word(0x00, 0x1000, Ok(())),
        write_word(0x00, 0x1001, Ok(())),
        read_word(0x04, 0x1234),
        read_word(0x05, 0x5678),
        write_word(0x03, 0x0000, Ok(())),
        write_word(0x00, 0x0001, Ok(())),
    ]);
    let mut delay = CancellableDelay::ready();
    let mut sensor = Veml7700::new(bus);
    let sample = block_on(sensor.measure_once(
        &mut delay,
        MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms100),
    ))
    .unwrap();
    assert_eq!(sample.als, AlsCounts::from_counts(0x1234));
    assert_eq!(sample.white, WhiteCounts::from_counts(0x5678));
    assert_eq!(sample.requested_wait_us, 133_500);
    assert_eq!(delay.elapsed_ns(), 133_500_000);
    sensor.release().done();
}

#[test]
fn explicit_timing_must_match_the_measurement_integration_time() {
    let bus = ScriptedI2c::new([]);
    let mut delay = CancellableDelay::ready();
    let mut sensor = Veml7700::new(bus);
    let result = block_on(sensor.measure_once_with_timing(
        &mut delay,
        MeasurementConfig::new(crate::Gain::Div8, crate::IntegrationTime::Ms800),
        crate::MeasurementTiming::conservative(crate::IntegrationTime::Ms25),
    ));
    assert_eq!(
        result,
        Err(MeasureOnceError::Operation {
            stage: MeasureStage::ValidateTiming,
            source: Error::Configuration(ConfigurationError::TimingIntegrationMismatch {
                measurement: crate::IntegrationTime::Ms800,
                timing: crate::IntegrationTime::Ms25,
            },),
        })
    );
    assert_eq!(delay.elapsed_ns(), 0);
    sensor.release().done();
}

#[test]
fn disable_power_saving_failure_is_followed_by_state_restoration() {
    let failure = ScriptError::new(ErrorKind::Bus);
    let bus = ScriptedI2c::new([
        read_word(0x00, 0x0001),
        read_word(0x03, 0x0000),
        write_word(0x03, 0x0000, Err(failure)),
        write_word(0x03, 0x0000, Ok(())),
        write_word(0x00, 0x0001, Ok(())),
    ]);
    let mut delay = CancellableDelay::ready();
    let mut sensor = Veml7700::new(bus);
    let result = block_on(sensor.measure_once(
        &mut delay,
        MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms100),
    ));
    assert_eq!(
        result,
        Err(MeasureOnceError::Operation {
            stage: MeasureStage::DisablePowerSaving,
            source: Error::Bus {
                operation: Operation::MeasureOnce,
                context: BusContext::WritePowerSaving,
                source: failure,
            },
        })
    );
    assert_eq!(delay.elapsed_ns(), 0);
    sensor.release().done();
}

#[test]
fn fresh_capture_from_active_enters_shutdown_before_touching_the_domain() {
    let bus = ScriptedI2c::new([
        read_word(0x00, 0x0000),
        read_word(0x03, 0x0000),
        // Shutdown in the original domain, before power saving or gain move.
        write_word(0x00, 0x0001, Ok(())),
        write_word(0x03, 0x0000, Ok(())),
        write_word(0x00, 0x1001, Ok(())),
        write_word(0x00, 0x1000, Ok(())),
        write_word(0x00, 0x1001, Ok(())),
        read_word(0x04, 0x1234),
        read_word(0x05, 0x5678),
        write_word(0x03, 0x0000, Ok(())),
        // Restoration returns the device to the active state it started in.
        write_word(0x00, 0x0000, Ok(())),
    ]);
    let mut delay = CancellableDelay::ready();
    let mut sensor = Veml7700::new(bus);
    let sample = block_on(sensor.measure_once(
        &mut delay,
        MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms100),
    ))
    .unwrap();
    assert_eq!(sample.als, AlsCounts::from_counts(0x1234));
    assert_eq!(sample.white, WhiteCounts::from_counts(0x5678));
    sensor.release().done();
}

#[test]
fn a_failed_shutdown_reports_without_attempting_restoration() {
    // Nothing has been mutated when the first write fails, and the device
    // may still be active. Restoring would write power saving to an active
    // device — the exact write this sequence exists to avoid.
    let bus = ScriptedI2c::new([
        read_word(0x00, 0x0000),
        read_word(0x03, 0x0000),
        write_word(0x00, 0x0001, Err(ScriptError::new(ErrorKind::Bus))),
    ]);
    let mut delay = CancellableDelay::ready();
    let mut sensor = Veml7700::new(bus);
    let result = block_on(sensor.measure_once(
        &mut delay,
        MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms100),
    ));
    assert!(matches!(
        result,
        Err(MeasureOnceError::Operation {
            stage: MeasureStage::EnterShutdown,
            ..
        })
    ));
    sensor.release().done();
}

// Cancellation boundaries.
//
// These assert *sequencing*: after dropping the future at boundary k,
// exactly k transactions were issued. That is the whole of what a scripted
// transport can establish. Whether the device physically committed the
// transaction in flight is unknowable here and is not asserted anywhere.

#[test]
fn every_fresh_capture_stage_failure_is_restored_and_identified() {
    let failure = ScriptError::new(ErrorKind::Bus);
    let stages = [
        (
            MeasureStage::DisablePowerSaving,
            BusContext::WritePowerSaving,
        ),
        (
            MeasureStage::PrepareMeasurement,
            BusContext::WriteConfiguration,
        ),
        (
            MeasureStage::ActivateMeasurement,
            BusContext::WriteConfiguration,
        ),
        (MeasureStage::FreezeResult, BusContext::WriteConfiguration),
        (MeasureStage::ReadAls, BusContext::ReadAls),
        (MeasureStage::ReadWhite, BusContext::ReadWhite),
    ];

    for (failed_index, (stage, context)) in stages.into_iter().enumerate() {
        let mut expectations = vec![read_word(0x00, 0x0001), read_word(0x03, 0x0000)];
        for index in 0..=failed_index {
            let result = (index != failed_index).then_some(()).ok_or(failure);
            expectations.push(match index {
                0 => write_word(0x03, 0x0000, result),
                1 | 3 => write_word(0x00, 0x1001, result),
                2 => write_word(0x00, 0x1000, result),
                4 if result.is_err() => read_failure(0x04, failure),
                4 => read_word(0x04, 0x1234),
                5 => read_failure(0x05, failure),
                _ => unreachable!(),
            });
        }
        expectations.push(write_word(0x03, 0x0000, Ok(())));
        expectations.push(write_word(0x00, 0x0001, Ok(())));

        let mut delay = CancellableDelay::ready();
        let mut sensor = Veml7700::new(ScriptedI2c::new(expectations));
        assert_eq!(
            block_on(sensor.measure_once(
                &mut delay,
                MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms100),
            )),
            Err(MeasureOnceError::Operation {
                stage,
                source: Error::Bus {
                    operation: Operation::MeasureOnce,
                    context,
                    source: failure,
                },
            })
        );
        assert_eq!(delay.elapsed_ns() != 0, failed_index >= 3);
        sensor.release().done();
    }
}

#[test]
fn observation_failures_are_identified_without_cleanup_writes() {
    let failure = ScriptError::new(ErrorKind::Bus);
    let mut delay = CancellableDelay::ready();
    let mut configuration = Veml7700::new(ScriptedI2c::new([read_failure(0x00, failure)]));
    assert_eq!(
        block_on(configuration.measure_once(
            &mut delay,
            MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms100),
        )),
        Err(MeasureOnceError::Operation {
            stage: MeasureStage::ObserveConfiguration,
            source: Error::Bus {
                operation: Operation::MeasureOnce,
                context: BusContext::ReadConfiguration,
                source: failure,
            },
        })
    );
    configuration.release().done();

    let mut power = Veml7700::new(ScriptedI2c::new([
        read_word(0x00, 0x0001),
        read_failure(0x03, failure),
    ]));
    assert_eq!(
        block_on(power.measure_once(
            &mut delay,
            MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms100)
        )),
        Err(MeasureOnceError::Operation {
            stage: MeasureStage::ObservePowerSaving,
            source: Error::Bus {
                operation: Operation::MeasureOnce,
                context: BusContext::ReadPowerSaving,
                source: failure,
            },
        })
    );
    power.release().done();
}

#[test]
fn cleanup_failure_reports_primary_and_recovery_errors() {
    let primary = ScriptError::new(ErrorKind::Bus);
    let recovery = ScriptError::new(ErrorKind::ArbitrationLoss);
    let bus = ScriptedI2c::new([
        read_word(0x00, 0x0001),
        read_word(0x03, 0x0000),
        write_word(0x03, 0x0000, Err(primary)),
        write_word(0x03, 0x0000, Err(recovery)),
    ]);
    let mut delay = CancellableDelay::ready();
    let mut sensor = Veml7700::new(bus);
    assert_eq!(
        block_on(sensor.measure_once(
            &mut delay,
            MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms100),
        )),
        Err(MeasureOnceError::RecoveryFailed {
            failed_stage: MeasureStage::DisablePowerSaving,
            source: Error::Bus {
                operation: Operation::MeasureOnce,
                context: BusContext::WritePowerSaving,
                source: primary,
            },
            recovery_stage: MeasureStage::RestorePowerSaving,
            recovery_source: Error::Bus {
                operation: Operation::MeasureOnce,
                context: BusContext::WritePowerSaving,
                source: recovery,
            },
        })
    );
    sensor.release().done();
}

#[test]
fn post_capture_restoration_failure_preserves_the_sample() {
    let failure = ScriptError::new(ErrorKind::Bus);
    let bus = ScriptedI2c::new([
        read_word(0x00, 0x0001),
        read_word(0x03, 0x0000),
        write_word(0x03, 0x0000, Ok(())),
        write_word(0x00, 0x1001, Ok(())),
        write_word(0x00, 0x1000, Ok(())),
        write_word(0x00, 0x1001, Ok(())),
        read_word(0x04, 0x1234),
        read_word(0x05, 0x5678),
        write_word(0x03, 0x0000, Err(failure)),
    ]);
    let mut delay = CancellableDelay::ready();
    let mut sensor = Veml7700::new(bus);
    match block_on(sensor.measure_once(
        &mut delay,
        MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms100),
    )) {
        Err(MeasureOnceError::RestoreFailed {
            sample,
            stage,
            source,
        }) => {
            assert_eq!(sample.als, AlsCounts::from_counts(0x1234));
            assert_eq!(sample.white, WhiteCounts::from_counts(0x5678));
            assert_eq!(stage, MeasureStage::RestorePowerSaving);
            assert_eq!(
                source,
                Error::Bus {
                    operation: Operation::MeasureOnce,
                    context: BusContext::WritePowerSaving,
                    source: failure,
                }
            );
        }
        other => panic!("unexpected result: {other:?}"),
    }
    sensor.release().done();
}

#[test]
fn configuration_restoration_failures_preserve_their_stage_and_sample_state() {
    let primary = ScriptError::new(ErrorKind::Bus);
    let recovery = ScriptError::new(ErrorKind::ArbitrationLoss);
    let pre_capture = ScriptedI2c::new([
        read_word(0x00, 0x0001),
        read_word(0x03, 0x0000),
        write_word(0x03, 0x0000, Err(primary)),
        write_word(0x03, 0x0000, Ok(())),
        write_word(0x00, 0x0001, Err(recovery)),
    ]);
    let mut delay = CancellableDelay::ready();
    let mut sensor = Veml7700::new(pre_capture);
    assert_eq!(
        block_on(sensor.measure_once(
            &mut delay,
            MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms100),
        )),
        Err(MeasureOnceError::RecoveryFailed {
            failed_stage: MeasureStage::DisablePowerSaving,
            source: Error::Bus {
                operation: Operation::MeasureOnce,
                context: BusContext::WritePowerSaving,
                source: primary,
            },
            recovery_stage: MeasureStage::RestoreConfiguration,
            recovery_source: Error::Bus {
                operation: Operation::MeasureOnce,
                context: BusContext::WriteConfiguration,
                source: recovery,
            },
        })
    );
    sensor.release().done();

    let post_capture = ScriptedI2c::new([
        read_word(0x00, 0x0001),
        read_word(0x03, 0x0000),
        write_word(0x03, 0x0000, Ok(())),
        write_word(0x00, 0x1001, Ok(())),
        write_word(0x00, 0x1000, Ok(())),
        write_word(0x00, 0x1001, Ok(())),
        read_word(0x04, 0x1234),
        read_word(0x05, 0x5678),
        write_word(0x03, 0x0000, Ok(())),
        write_word(0x00, 0x0001, Err(recovery)),
    ]);
    let mut sensor = Veml7700::new(post_capture);
    match block_on(sensor.measure_once(
        &mut delay,
        MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms100),
    )) {
        Err(MeasureOnceError::RestoreFailed {
            sample,
            stage,
            source,
        }) => {
            assert_eq!(sample.als, AlsCounts::from_counts(0x1234));
            assert_eq!(sample.white, WhiteCounts::from_counts(0x5678));
            assert_eq!(stage, MeasureStage::RestoreConfiguration);
            assert_eq!(
                source,
                Error::Bus {
                    operation: Operation::MeasureOnce,
                    context: BusContext::WriteConfiguration,
                    source: recovery,
                }
            );
        }
        other => panic!("unexpected result: {other:?}"),
    }
    sensor.release().done();
}
