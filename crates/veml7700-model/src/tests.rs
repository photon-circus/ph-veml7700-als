use crate::{
    DEVICE_ID, I2C_ADDRESS, NoAcknowledgeSource, RelativeDuration, TransportError, Unsupported,
    Veml7700Model,
};

const CONFIG: u8 = 0x00;
const POWER_SAVING: u8 = 0x03;
const ALS: u8 = 0x04;
const WHITE: u8 = 0x05;
const ID: u8 = 0x07;
const THRESHOLD_STATUS: u8 = 0x06;

/// Gain ×1/8, 100 ms, shutdown — the `measure_once` prepare word.
const PREPARED_100MS: u16 = 0x1001;
/// Gain ×1/8, 100 ms, active — the wake word.
const ACTIVE_100MS: u16 = 0x1000;
/// 2.5 ms wake plus 130% of 100 ms.
const BOUND_100MS_US: u64 = 2_500 + 130_000;

fn read_word(model: &mut Veml7700Model, pointer: u8) -> u16 {
    read_word_result(model, pointer).expect("supported register read")
}

fn read_word_result(model: &mut Veml7700Model, pointer: u8) -> Result<u16, TransportError> {
    let mut bytes = [0_u8; 2];
    model
        .write_read(I2C_ADDRESS, &[pointer], &mut bytes)
        .map(|()| u16::from_le_bytes(bytes))
}

fn write_word(model: &mut Veml7700Model, pointer: u8, word: u16) {
    let [low, high] = word.to_le_bytes();
    model
        .write(I2C_ADDRESS, &[pointer, low, high])
        .expect("supported register write");
}

fn wake_100ms(model: &mut Veml7700Model) {
    write_word(model, CONFIG, PREPARED_100MS);
    write_word(model, CONFIG, ACTIVE_100MS);
}

fn freeze_100ms(model: &mut Veml7700Model) {
    write_word(model, CONFIG, PREPARED_100MS);
}

#[test]
fn reset_state_matches_documented_defaults() {
    let mut model = Veml7700Model::new();
    assert_eq!(read_word(&mut model, CONFIG), 0x0001);
    assert_eq!(read_word(&mut model, POWER_SAVING), 0x0000);
    assert_eq!(
        read_word_result(&mut model, ALS),
        Err(TransportError::Unsupported(
            Unsupported::NoCompletedConversion(ALS)
        ))
    );
    assert_eq!(
        read_word_result(&mut model, WHITE),
        Err(TransportError::Unsupported(
            Unsupported::NoCompletedConversion(WHITE)
        ))
    );
    assert_eq!(read_word(&mut model, ID), DEVICE_ID);
    let snapshot = model.inspect();
    assert_eq!(snapshot.configuration, 0x0001);
    assert_eq!(snapshot.als, None);
    assert_eq!(snapshot.white, None);
    assert_eq!(snapshot.remaining, None);
}

#[test]
fn id_register_is_low_byte_first() {
    let mut model = Veml7700Model::new();
    let mut bytes = [0_u8; 2];
    model
        .write_read(I2C_ADDRESS, &[ID], &mut bytes)
        .expect("ID read");
    assert_eq!(bytes, [0x81, 0xC4]);
    assert_eq!(u16::from_le_bytes(bytes), 0xC481);
}

#[test]
fn conversion_does_not_complete_before_the_conservative_bound() {
    let mut model = Veml7700Model::new();
    model.set_raw_sample(0x1234, 0x5678);
    wake_100ms(&mut model);
    model.advance(RelativeDuration::from_micros(BOUND_100MS_US - 1));
    assert_eq!(
        read_word_result(&mut model, ALS),
        Err(TransportError::Unsupported(
            Unsupported::NoCompletedConversion(ALS)
        ))
    );
    assert_eq!(
        read_word_result(&mut model, WHITE),
        Err(TransportError::Unsupported(
            Unsupported::NoCompletedConversion(WHITE)
        ))
    );
    assert!(model.inspect().remaining.is_some());
}

#[test]
fn conversion_latches_the_held_pair_at_the_conservative_bound() {
    let mut model = Veml7700Model::new();
    model.set_raw_sample(0x1234, 0x5678);
    wake_100ms(&mut model);
    model.advance(RelativeDuration::from_micros(BOUND_100MS_US));
    assert_eq!(read_word(&mut model, ALS), 0x1234);
    assert_eq!(read_word(&mut model, WHITE), 0x5678);
    assert_eq!(model.inspect().remaining, None);
}

#[test]
fn changing_the_held_sample_does_not_alter_an_already_latched_pair() {
    let mut model = Veml7700Model::new();
    model.set_raw_sample(11, 22);
    wake_100ms(&mut model);
    model.advance(RelativeDuration::from_micros(BOUND_100MS_US));
    model.set_raw_sample(99, 88);
    assert_eq!(read_word(&mut model, ALS), 11);
    assert_eq!(read_word(&mut model, WHITE), 22);
}

#[test]
fn shutdown_retains_the_last_pair_and_ignores_later_time_and_samples() {
    let mut model = Veml7700Model::new();
    model.set_raw_sample(7, 8);
    wake_100ms(&mut model);
    model.advance(RelativeDuration::from_micros(BOUND_100MS_US));
    freeze_100ms(&mut model);
    model.set_raw_sample(100, 200);
    model.advance(RelativeDuration::from_micros(1_000_000));
    assert_eq!(read_word(&mut model, ALS), 7);
    assert_eq!(read_word(&mut model, WHITE), 8);
    assert_eq!(read_word(&mut model, CONFIG), PREPARED_100MS);
}

#[test]
fn shutdown_before_the_bound_keeps_the_previous_completed_pair() {
    let mut model = Veml7700Model::new();
    model.set_raw_sample(3, 4);
    wake_100ms(&mut model);
    model.advance(RelativeDuration::from_micros(BOUND_100MS_US - 1));
    freeze_100ms(&mut model);
    model.advance(RelativeDuration::from_micros(BOUND_100MS_US));
    assert_eq!(
        read_word_result(&mut model, ALS),
        Err(TransportError::Unsupported(
            Unsupported::NoCompletedConversion(ALS)
        ))
    );
    assert_eq!(
        read_word_result(&mut model, WHITE),
        Err(TransportError::Unsupported(
            Unsupported::NoCompletedConversion(WHITE)
        ))
    );
}

#[test]
fn repeated_active_configuration_is_rejected_without_mutation() {
    let mut model = Veml7700Model::new();
    wake_100ms(&mut model);
    model.advance(RelativeDuration::from_micros(10_000));
    let before = model.inspect();
    let [low, high] = ACTIVE_100MS.to_le_bytes();

    assert_eq!(
        model.write(I2C_ADDRESS, &[CONFIG, low, high]),
        Err(TransportError::Unsupported(
            Unsupported::MidConversionReconfiguration
        ))
    );
    assert_eq!(model.inspect(), before);
}

#[test]
fn repeated_reads_are_stable_at_an_unchanged_frontier() {
    let mut model = Veml7700Model::new();
    model.set_raw_sample(0xABCD, 0xDCBA);
    wake_100ms(&mut model);
    model.advance(RelativeDuration::from_micros(BOUND_100MS_US));
    freeze_100ms(&mut model);
    let first_als = read_word(&mut model, ALS);
    let first_white = read_word(&mut model, WHITE);
    let first_id = read_word(&mut model, ID);
    assert_eq!(read_word(&mut model, ALS), first_als);
    assert_eq!(read_word(&mut model, WHITE), first_white);
    assert_eq!(read_word(&mut model, ID), first_id);
    assert_eq!(model.inspect(), model.inspect());
}

#[test]
fn duration_partitions_are_observationally_equivalent() {
    let mut once = Veml7700Model::new();
    once.set_raw_sample(42, 43);
    wake_100ms(&mut once);
    once.advance(RelativeDuration::from_micros(BOUND_100MS_US));

    let mut split = Veml7700Model::new();
    split.set_raw_sample(42, 43);
    wake_100ms(&mut split);
    split.advance(RelativeDuration::from_micros(40_000));
    split.advance(RelativeDuration::from_micros(50_000));
    split.advance(RelativeDuration::from_nanos(42_500_000));

    assert_eq!(read_word(&mut once, ALS), read_word(&mut split, ALS));
    assert_eq!(read_word(&mut once, WHITE), read_word(&mut split, WHITE));
    assert_eq!(once.inspect().remaining, split.inspect().remaining);
}

#[test]
fn reads_do_not_consume_conversion_time() {
    let mut model = Veml7700Model::new();
    model.set_raw_sample(1, 2);
    wake_100ms(&mut model);
    for _ in 0..8 {
        assert!(matches!(
            read_word_result(&mut model, ALS),
            Err(TransportError::Unsupported(
                Unsupported::NoCompletedConversion(ALS)
            ))
        ));
        assert!(matches!(
            read_word_result(&mut model, WHITE),
            Err(TransportError::Unsupported(
                Unsupported::NoCompletedConversion(WHITE)
            ))
        ));
        let _ = read_word(&mut model, CONFIG);
    }
    model.advance(RelativeDuration::from_micros(BOUND_100MS_US));
    assert_eq!(read_word(&mut model, ALS), 1);
}

#[test]
fn wrong_address_is_a_device_nack_and_does_not_mutate() {
    let mut model = Veml7700Model::new();
    model.set_raw_sample(5, 6);
    let before = model.inspect();
    let write = model.write(0x11, &[CONFIG, 0x00, 0x10]);
    assert_eq!(
        write,
        Err(TransportError::NoAcknowledge {
            source: NoAcknowledgeSource::Address,
        })
    );
    let mut bytes = [0_u8; 2];
    let read = model.write_read(0x11, &[ID], &mut bytes);
    assert_eq!(
        read,
        Err(TransportError::NoAcknowledge {
            source: NoAcknowledgeSource::Address,
        })
    );
    assert_eq!(bytes, [0, 0]);
    assert_eq!(model.inspect(), before);
}

#[test]
fn unsupported_pointer_is_not_a_device_nack() {
    let mut model = Veml7700Model::new();
    let mut bytes = [0_u8; 2];
    let read = model.write_read(I2C_ADDRESS, &[THRESHOLD_STATUS], &mut bytes);
    assert_eq!(
        read,
        Err(TransportError::Unsupported(Unsupported::RegisterPointer(
            THRESHOLD_STATUS
        )))
    );
    assert_ne!(
        read,
        Err(TransportError::NoAcknowledge {
            source: NoAcknowledgeSource::Address,
        })
    );
    let write = model.write(I2C_ADDRESS, &[THRESHOLD_STATUS, 0, 0]);
    assert_eq!(
        write,
        Err(TransportError::Unsupported(Unsupported::RegisterPointer(
            THRESHOLD_STATUS
        )))
    );
}

#[test]
fn unsupported_transaction_shape_is_rejected_without_mutation() {
    let mut model = Veml7700Model::new();
    let before = model.inspect();
    assert_eq!(
        model.write(I2C_ADDRESS, &[CONFIG]),
        Err(TransportError::Unsupported(Unsupported::TransactionShape))
    );
    let mut one = [0_u8; 1];
    assert_eq!(
        model.write_read(I2C_ADDRESS, &[ID], &mut one),
        Err(TransportError::Unsupported(Unsupported::TransactionShape))
    );
    assert_eq!(model.inspect(), before);
}

#[test]
fn words_outside_the_declared_slice_are_rejected_without_mutation() {
    let mut model = Veml7700Model::new();
    let before = model.inspect();

    let [low, high] = 0x0001_u16.to_le_bytes();
    assert_eq!(
        model.write(I2C_ADDRESS, &[POWER_SAVING, low, high]),
        Err(TransportError::Unsupported(Unsupported::PowerSavingWord(
            0x0001
        )))
    );

    let unsupported_configuration = 0x1003_u16;
    let [low, high] = unsupported_configuration.to_le_bytes();
    assert_eq!(
        model.write(I2C_ADDRESS, &[CONFIG, low, high]),
        Err(TransportError::Unsupported(Unsupported::ConfigurationWord(
            unsupported_configuration
        )))
    );
    assert_eq!(model.inspect(), before);
}
