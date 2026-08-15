//! Configuration and power-saving writes, including shutdown-first sequencing.

use super::*;

#[test]
fn configuration_write_is_low_byte_first() {
    let bus = ScriptedI2c::new([read_word(0x00, 0x0001), write_word(0x00, 0x1001, Ok(()))]);
    let mut sensor = Veml7700::new(bus);
    block_on(
        sensor.set_measurement_config(MeasurementConfig::new(Gain::Div8, IntegrationTime::Ms100)),
    )
    .unwrap();
    sensor.release().done();
}

#[test]
fn monitor_blocks_power_and_cadence_changes_before_write() {
    let mut power = Veml7700::new(ScriptedI2c::new([read_word(0x00, 0x0002)]));
    assert_eq!(
        block_on(power.set_power_state(PowerState::Shutdown)),
        Err(Error::Configuration(
            ConfigurationError::ThresholdMonitorOwnsDomain
        ))
    );
    power.release().done();

    let mut cadence = Veml7700::new(ScriptedI2c::new([
        read_word(0x00, 0x0002),
        read_word(0x03, 0x0000),
    ]));
    assert_eq!(
        block_on(cadence.set_power_saving(PowerSavingConfig::new(true, PowerSavingMode::Mode1,))),
        Err(Error::Configuration(
            ConfigurationError::ThresholdMonitorOwnsDomain
        ))
    );
    cadence.release().done();
}

// The driver's `S-19` reaction enters shutdown before reconfiguration. Each test
// below starts from an active device, which is the case every other test in
// this module misses: from shutdown the sequencing writes collapse into
// no-ops, so a shutdown-only suite passes whether or not the rule is
// followed. The exact word order is the assertion — a shutdown write that
// also carried the new domain would satisfy a transaction count but violate
// the contract.

#[test]
fn reconfiguration_from_active_shuts_down_before_changing_the_domain() {
    // 0x0000 active, gain ×1, 100 ms. Target Div8/800 ms is 0x10C0 active.
    let bus = ScriptedI2c::new([
        read_word(0x00, 0x0000),
        // Shutdown carries the *old* domain: bit 0 only.
        write_word(0x00, 0x0001, Ok(())),
        // The new domain lands while shut down.
        write_word(0x00, 0x10C1, Ok(())),
        // Active last.
        write_word(0x00, 0x10C0, Ok(())),
    ]);
    let mut sensor = Veml7700::new(bus);
    block_on(sensor.set_measurement_config(MeasurementConfig::new(
        crate::Gain::Div8,
        crate::IntegrationTime::Ms800,
    )))
    .unwrap();
    sensor.release().done();
}

#[test]
fn reconfiguration_from_shutdown_stays_a_single_write() {
    let bus = ScriptedI2c::new([read_word(0x00, 0x0001), write_word(0x00, 0x10C1, Ok(()))]);
    let mut sensor = Veml7700::new(bus);
    block_on(sensor.set_measurement_config(MeasurementConfig::new(
        crate::Gain::Div8,
        crate::IntegrationTime::Ms800,
    )))
    .unwrap();
    // An already shut-down device needs no shutdown write and must not be
    // woken as a side effect of reconfiguring it.
    sensor.release().done();
}

#[test]
fn cadence_change_from_active_shuts_down_before_writing_power_saving() {
    let bus = ScriptedI2c::new([
        read_word(0x00, 0x0000),
        read_word(0x03, 0x0000),
        write_word(0x00, 0x0001, Ok(())),
        write_word(0x03, 0x0003, Ok(())),
        write_word(0x00, 0x0000, Ok(())),
    ]);
    let mut sensor = Veml7700::new(bus);
    block_on(sensor.set_power_saving(PowerSavingConfig::new(true, PowerSavingMode::Mode2)))
        .unwrap();
    sensor.release().done();
}

#[test]
fn setting_an_unchanged_value_writes_nothing() {
    // Reset default is gain ×1 / 100 ms. Requesting it back must not cycle
    // power: an enabled monitor would lose its active domain for a call
    // that changes no field.
    let mut measurement = Veml7700::new(ScriptedI2c::new([read_word(0x00, 0x0002)]));
    block_on(measurement.set_measurement_config(MeasurementConfig::silicon_reset_default()))
        .unwrap();
    measurement.release().done();

    let mut cadence = Veml7700::new(ScriptedI2c::new([
        read_word(0x00, 0x0002),
        read_word(0x03, 0x0000),
    ]));
    block_on(cadence.set_power_saving(PowerSavingConfig::disabled())).unwrap();
    cadence.release().done();
}
