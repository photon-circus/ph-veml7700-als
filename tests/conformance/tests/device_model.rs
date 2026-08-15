//! Driver-versus-model traces for the declared behavioral slice.
//!
//! Every trace drives the **public** driver API against the independent model
//! through the adapters in this package's library. What a passing trace
//! establishes is agreement between two independent readings of the same
//! documents -- not device correctness, and not physical qualification.
//!
//! The maintained list of what is and is not covered lives in the packaged
//! `crates/veml7700/README.md`, because that surface ships to a consumer. The
//! canonical gate compares it against this file in both directions.

use futures::executor::block_on;
use ph_veml7700_als::{
    AlsCounts, BusContext, ConfigurationSnapshot, Error, Gain, IntegrationTime, MeasurementConfig,
    Operation, Persistence, PowerSavingConfig, PowerSavingMode, PowerState, ThresholdMonitorConfig,
    ThresholdMonitorState, Thresholds, WhiteCounts,
};
use ph_veml7700_als_conformance::{ModelBusError, connected_model};
use ph_veml7700_als_model::{RelativeDuration, Unsupported};

#[test]
fn probe_accepts_the_fixed_address_little_endian_id() {
    let (mut sensor, _delay) = connected_model(0, 0);
    let id = block_on(sensor.probe()).expect("probe against the model");
    assert_eq!(id.raw(), 0xC481);
}

#[test]
fn measure_once_returns_the_injected_pair_after_the_driver_delay_and_restores_state() {
    let als = 0x1234;
    let white = 0x5678;
    let (mut sensor, mut delay) = connected_model(als, white);
    let sample =
        block_on(sensor.measure_once(&mut delay, MeasurementConfig::maximum_range_start()))
            .expect("measure_once against the model");

    assert_eq!(sample.als, AlsCounts::from_counts(als));
    assert_eq!(sample.white, WhiteCounts::from_counts(white));
    assert_eq!(sample.requested_wait_us, 36_000);

    let configuration = block_on(sensor.read_configuration()).expect("restored configuration");
    let power_saving = block_on(sensor.read_power_saving()).expect("restored power saving");
    assert_eq!(
        configuration,
        ConfigurationSnapshot::silicon_reset_default()
    );
    assert!(!power_saving.enabled);
    assert_eq!(power_saving.mode, PowerSavingMode::Mode1);
}

// Every other trace in this file starts from model reset, which is shutdown.
// These two start active. That initial state is newly inside the claim: the
// driver now enters shutdown before reconfiguring, so the model accepts a
// sequence it would previously have rejected as mid-conversion reconfiguration.
// They are the driver-versus-model evidence for the sequence change, and they
// would fail against the old driver rather than merely cover more ground.

#[test]
fn measure_once_from_an_active_start_agrees_with_the_model() {
    let als = 0x0BEE;
    let white = 0x0C0F;
    let (mut sensor, mut delay) = connected_model(als, white);
    block_on(sensor.set_power_state(PowerState::Active)).expect("activate before measuring");

    let sample =
        block_on(sensor.measure_once(&mut delay, MeasurementConfig::maximum_range_start()))
            .expect("measure_once from an active start");

    assert_eq!(sample.als, AlsCounts::from_counts(als));
    assert_eq!(sample.white, WhiteCounts::from_counts(white));

    // Restoration returns the device to the state it started in, which was
    // active — not to the reset default.
    let configuration = block_on(sensor.read_configuration()).expect("restored configuration");
    assert_eq!(configuration.power_state, PowerState::Active);
    assert_eq!(
        configuration.measurement,
        MeasurementConfig::silicon_reset_default()
    );
}

#[test]
fn arming_the_monitor_from_an_active_start_agrees_with_the_model() {
    let (mut sensor, _delay) = connected_model(500, 500);
    block_on(sensor.set_power_state(PowerState::Active)).expect("activate before arming");

    let thresholds =
        Thresholds::new(AlsCounts::from_counts(100), AlsCounts::from_counts(1_000)).unwrap();
    block_on(sensor.arm_threshold_monitor(ThresholdMonitorConfig::new(
        MeasurementConfig::new(Gain::X2, IntegrationTime::Ms100),
        thresholds,
        Persistence::Four,
        PowerSavingConfig::new(true, PowerSavingMode::Mode2),
    )))
    .expect("arm from an active start");

    let configuration = block_on(sensor.read_configuration()).expect("armed configuration");
    assert_eq!(
        configuration.threshold_monitor,
        ThresholdMonitorState::Enabled
    );
    assert_eq!(configuration.power_state, PowerState::Active);
    assert_eq!(configuration.persistence, Persistence::Four);
}

#[test]
fn re_arming_an_enabled_active_monitor_agrees_with_the_model() {
    let (mut sensor, _delay) = connected_model(500, 500);
    let thresholds =
        Thresholds::new(AlsCounts::from_counts(100), AlsCounts::from_counts(1_000)).unwrap();
    let monitor = ThresholdMonitorConfig::new(
        MeasurementConfig::new(Gain::X2, IntegrationTime::Ms100),
        thresholds,
        Persistence::Four,
        PowerSavingConfig::new(true, PowerSavingMode::Mode2),
    );
    block_on(sensor.arm_threshold_monitor(monitor)).expect("first arm");

    // The device is now active with the monitor enabled. Re-arming from here
    // must move the shutdown and monitor bits in separate writes: the model
    // accepts either alone as a transition, and both together as
    // MidConversionReconfiguration.
    let retarget = ThresholdMonitorConfig::new(
        MeasurementConfig::new(Gain::X2, IntegrationTime::Ms100),
        Thresholds::new(AlsCounts::from_counts(200), AlsCounts::from_counts(2_000)).unwrap(),
        Persistence::Four,
        PowerSavingConfig::new(true, PowerSavingMode::Mode2),
    );
    block_on(sensor.arm_threshold_monitor(retarget)).expect("re-arm an enabled active monitor");

    let programmed = block_on(sensor.read_thresholds()).expect("re-armed thresholds");
    assert_eq!(programmed.low(), AlsCounts::from_counts(200));
    assert_eq!(programmed.high(), AlsCounts::from_counts(2_000));
}

#[test]
fn public_power_operations_observe_the_documented_mode_2_refresh_boundary() {
    let (mut sensor, model) = connected_model(1, 1);
    block_on(
        sensor.set_measurement_config(MeasurementConfig::new(Gain::X2, IntegrationTime::Ms100)),
    )
    .expect("select the `S-21` gain domain");
    block_on(sensor.set_power_saving(PowerSavingConfig::new(true, PowerSavingMode::Mode2)))
        .expect("configure modeled cadence");
    block_on(sensor.set_power_state(PowerState::Active)).expect("wake modeled device");
    model.advance(RelativeDuration::from_micros(102_500));
    model.set_raw_sample(2, 2);

    model.advance(RelativeDuration::from_micros(1_099_999));
    assert_eq!(
        block_on(sensor.read_als_snapshot()).expect("pre-boundary ALS"),
        AlsCounts::from_counts(1)
    );
    model.advance(RelativeDuration::from_micros(1));
    assert_eq!(
        block_on(sensor.read_als_snapshot()).expect("at-boundary ALS"),
        AlsCounts::from_counts(2)
    );
}

#[test]
fn threshold_monitor_public_operations_program_read_back_and_disable() {
    // This trace covers the shared programming surface (`S-16`, `S-38`) without
    // manufacturing qualification behavior from the undefined `S-49`/`S-50`
    // device propositions.
    let (mut sensor, _model) = connected_model(250, 0);
    let thresholds = Thresholds::new(AlsCounts::from_counts(100), AlsCounts::from_counts(200))
        .expect("ordered thresholds");
    let monitor = ThresholdMonitorConfig::new(
        MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms100),
        thresholds,
        Persistence::One,
        PowerSavingConfig::disabled(),
    );
    block_on(sensor.arm_threshold_monitor(monitor)).expect("arm against the model");

    let observed = block_on(sensor.read_thresholds()).expect("threshold readback");
    assert_eq!(observed, thresholds);

    block_on(sensor.disable_threshold_monitor()).expect("disable monitor");
    assert_eq!(
        block_on(sensor.read_configuration())
            .expect("configuration after disable")
            .threshold_monitor,
        ThresholdMonitorState::Disabled
    );
}

#[test]
fn arming_programs_the_field_but_yields_no_modeled_status() {
    // The driver programs the protect number and promises nothing about
    // assertion timing. The model declares the qualification rule undefined
    // instead of inventing one.
    //
    // What this trace establishes is exactly the intersection -- the write path
    // is unaffected -- and it deliberately establishes nothing about when a flag
    // asserts. `S-39`, `S-49`, and `S-50` do not support deriving one.
    let (mut sensor, model) = connected_model(250, 0);
    let thresholds = Thresholds::new(AlsCounts::from_counts(100), AlsCounts::from_counts(200))
        .expect("ordered thresholds");
    let monitor = ThresholdMonitorConfig::new(
        MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms100),
        thresholds,
        Persistence::Four,
        PowerSavingConfig::disabled(),
    );
    block_on(sensor.arm_threshold_monitor(monitor)).expect("arm against the model");

    // Programming the `S-16` encoding must keep working.
    assert_eq!(
        block_on(sensor.read_configuration())
            .expect("armed configuration")
            .persistence,
        Persistence::Four
    );
    assert_eq!(
        block_on(sensor.read_thresholds()).expect("threshold readback"),
        thresholds
    );

    // Qualification is not. Advancing repeatedly does not turn the model's
    // undefined reaction to `S-39`, `S-49`, and `S-50` into one it can support.
    model.advance(RelativeDuration::from_micros(102_500 + 16 * 100_000));
    assert!(matches!(
        block_on(sensor.read_threshold_status()),
        Err(Error::Bus {
            operation: Operation::Inspect,
            context: BusContext::ReadThresholdStatus,
            source: ModelBusError::Unsupported(Unsupported::UndefinedQualificationRule { .. }),
        })
    ));
}

#[test]
fn public_channel_reads_can_observe_independently_refreshed_generations() {
    let (mut sensor, model) = connected_model(10, 20);
    model.set_white_phase_offset(RelativeDuration::from_micros(10));
    block_on(sensor.set_power_state(PowerState::Active)).expect("wake modeled device");
    model.advance(RelativeDuration::from_micros(102_510));
    assert_eq!(
        block_on(sensor.read_als_snapshot()).expect("initial ALS"),
        AlsCounts::from_counts(10)
    );
    assert_eq!(
        block_on(sensor.read_white_snapshot()).expect("initial white"),
        WhiteCounts::from_counts(20)
    );

    model.set_raw_sample(30, 40);
    model.advance(RelativeDuration::from_micros(99_990));
    assert_eq!(
        block_on(sensor.read_als_snapshot()).expect("refreshed ALS"),
        AlsCounts::from_counts(30)
    );
    assert_eq!(
        block_on(sensor.read_white_snapshot()).expect("retained white"),
        WhiteCounts::from_counts(20)
    );
    model.advance(RelativeDuration::from_micros(10));
    assert_eq!(
        block_on(sensor.read_white_snapshot()).expect("refreshed white"),
        WhiteCounts::from_counts(40)
    );
}
