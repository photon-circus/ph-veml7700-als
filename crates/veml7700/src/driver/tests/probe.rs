//! Construction, probe, and identity.

use super::*;

#[test]
fn probe_reads_little_endian_id() {
    let bus = ScriptedI2c::new([read_word(0x07, 0xC481)]);
    let mut sensor = Veml7700::new(bus);
    let id = block_on(sensor.probe()).unwrap();
    assert_eq!(id.raw(), 0xC481);
    sensor.release().done();
}

#[test]
fn probe_preserves_non_address_bus_error() {
    let failure = ScriptError::new(ErrorKind::Bus);
    let bus = ScriptedI2c::new([read_failure(0x07, failure)]);
    let mut sensor = Veml7700::new(bus);
    assert_eq!(block_on(sensor.probe()), Err(ProbeError::Bus(failure)));
    sensor.release().done();
}

#[test]
fn probe_classifies_address_nack_and_wrong_identity() {
    let nack = ScriptError::new(ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address));
    let mut absent = Veml7700::new(ScriptedI2c::new([read_failure(0x07, nack)]));
    assert_eq!(block_on(absent.probe()), Err(ProbeError::NotPresent));
    absent.release().done();

    let mut wrong = Veml7700::new(ScriptedI2c::new([read_word(0x07, 0x0081)]));
    assert_eq!(
        block_on(wrong.probe()),
        Err(ProbeError::WrongDevice { observed: 0x0081 })
    );
    wrong.release().done();
}
