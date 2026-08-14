//! Read-only observation and strict decoding.

use super::*;

#[test]
fn snapshot_reads_provenance_before_sequential_channels() {
    let bus = ScriptedI2c::new([
        read_word(0x00, 0x0001),
        read_word(0x03, 0x0000),
        read_word(0x04, 0x1234),
        read_word(0x05, 0x5678),
    ]);
    let mut sensor = Veml7700::new(bus);
    let snapshot = block_on(sensor.snapshot()).unwrap();
    assert_eq!(snapshot.als, AlsCounts::from_counts(0x1234));
    assert_eq!(snapshot.white, WhiteCounts::from_counts(0x5678));
    assert_eq!(
        snapshot.configuration,
        crate::ConfigurationSnapshot::silicon_reset_default()
    );
    assert_eq!(
        snapshot.coherence,
        crate::MeasurementPairCoherence::SequentialRegisters
    );
    sensor.release().done();
}

#[test]
fn inspect_reads_the_complete_diagnostic_register_set() {
    let bus = ScriptedI2c::new([
        read_word(0x07, 0xC481),
        read_word(0x00, 0x0001),
        read_word(0x03, 0x0000),
        read_word(0x02, 100),
        read_word(0x01, 1_000),
        read_word(0x06, 0x4000),
    ]);
    let mut sensor = Veml7700::new(bus);
    let snapshot = block_on(sensor.inspect()).unwrap();
    assert_eq!(snapshot.id.raw(), 0xC481);
    assert_eq!(snapshot.thresholds.low().counts(), 100);
    assert_eq!(snapshot.thresholds.high().counts(), 1_000);
    assert!(!snapshot.threshold_status.low);
    assert!(snapshot.threshold_status.high);
    sensor.release().done();
}

#[test]
fn strict_decode_errors_preserve_semantic_context() {
    let mut configuration = Veml7700::new(ScriptedI2c::new([read_word(0x00, 0x0004)]));
    assert_eq!(
        block_on(configuration.read_configuration()),
        Err(Error::Configuration(
            ConfigurationError::ConfigurationDecode(crate::ConfigDecodeError::ReservedBits {
                observed: 4
            })
        ))
    );
    configuration.release().done();

    let mut power = Veml7700::new(ScriptedI2c::new([read_word(0x03, 0x0008)]));
    assert_eq!(
        block_on(power.read_power_saving()),
        Err(Error::Configuration(ConfigurationError::PowerSavingDecode(
            crate::PowerSavingDecodeError::ReservedBits { observed: 8 }
        )))
    );
    power.release().done();

    let mut status = Veml7700::new(ScriptedI2c::new([read_word(0x06, 0x0001)]));
    assert_eq!(
        block_on(status.read_threshold_status()),
        Err(Error::Configuration(
            ConfigurationError::ThresholdStatusDecode(
                crate::ThresholdStatusDecodeError::ReservedBits { observed: 1 }
            )
        ))
    );
    status.release().done();

    let mut thresholds = Veml7700::new(ScriptedI2c::new([read_word(0x02, 2), read_word(0x01, 1)]));
    assert_eq!(
        block_on(thresholds.read_thresholds()),
        Err(Error::Configuration(ConfigurationError::ReversedThresholds))
    );
    thresholds.release().done();
}
