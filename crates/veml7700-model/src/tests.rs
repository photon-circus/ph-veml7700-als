use crate::{
    DEVICE_ID, I2C_ADDRESS, MAX_ADVANCE, NoAcknowledgeSource, RelativeDuration,
    RetainedInputs as ModelInputs, TransportError, Unsupported, Veml7700Model,
};

const CONFIG: u8 = 0x00;
const HIGH_THRESHOLD: u8 = 0x01;
const LOW_THRESHOLD: u8 = 0x02;
const POWER_SAVING: u8 = 0x03;
const ALS: u8 = 0x04;
const WHITE: u8 = 0x05;
const ID: u8 = 0x07;
const THRESHOLD_STATUS: u8 = 0x06;

/// Gain ×1/8, 100 ms, shutdown — the `measure_once` prepare word.
const PREPARED_100MS: u16 = 0x1001;
/// Gain ×1/8, 100 ms, active — the wake word.
const ACTIVE_100MS: u16 = 0x1000;
/// Ideal-model completion: 2.5 ms wake plus nominal 100 ms integration.
const BOUND_100MS_US: u64 = 2_500 + 100_000;

// One explicit fixture choice for the undefined `S-11` initial word. Keeping it
// outside the model makes the value injected topology rather than device truth.
const fn injected_inputs(als_counts: u16, white_counts: u16) -> ModelInputs {
    ModelInputs::new(als_counts, white_counts, 0x0000)
}

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
fn construction_preserves_the_known_configuration_and_injected_power_word() {
    let mut model = Veml7700Model::new(injected_inputs(0, 0));
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
    assert_eq!(
        read_word_result(&mut model, THRESHOLD_STATUS),
        Err(TransportError::Unsupported(
            Unsupported::StatusReadWhileMonitorDisabled(THRESHOLD_STATUS)
        ))
    );
    assert_eq!(read_word(&mut model, ID), DEVICE_ID);
    let snapshot = model.inspect();
    assert_eq!(snapshot.configuration, 0x0001);
    assert_eq!(snapshot.als, None);
    assert_eq!(snapshot.white, None);
    assert_eq!(snapshot.als_remaining, None);
    assert_eq!(snapshot.white_remaining, None);
}

#[test]
fn id_register_is_low_byte_first() {
    let mut model = Veml7700Model::new(injected_inputs(0, 0));
    let mut bytes = [0_u8; 2];
    model
        .write_read(I2C_ADDRESS, &[ID], &mut bytes)
        .expect("ID read");
    assert_eq!(bytes, [0x81, 0xC4]);
    assert_eq!(u16::from_le_bytes(bytes), 0xC481);
}

#[test]
fn conversion_does_not_complete_before_the_ideal_completion_point() {
    let mut model = Veml7700Model::new(injected_inputs(0, 0));
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
    assert!(model.inspect().als_remaining.is_some());
    assert!(model.inspect().white_remaining.is_some());
}

#[test]
fn conversion_latches_the_held_pair_at_the_ideal_completion_point() {
    let mut model = Veml7700Model::new(injected_inputs(0, 0));
    model.set_raw_sample(0x1234, 0x5678);
    wake_100ms(&mut model);
    model.advance(RelativeDuration::from_micros(BOUND_100MS_US));
    assert_eq!(read_word(&mut model, ALS), 0x1234);
    assert_eq!(read_word(&mut model, WHITE), 0x5678);
    assert_eq!(
        read_word_result(&mut model, THRESHOLD_STATUS),
        Err(TransportError::Unsupported(
            Unsupported::StatusReadWhileMonitorDisabled(THRESHOLD_STATUS)
        ))
    );
    assert!(model.inspect().als_remaining.is_some());
    assert!(model.inspect().white_remaining.is_some());
}

#[test]
fn changing_the_held_sample_does_not_alter_an_already_latched_pair() {
    let mut model = Veml7700Model::new(injected_inputs(11, 22));
    wake_100ms(&mut model);
    model.advance(RelativeDuration::from_micros(BOUND_100MS_US));
    model.set_raw_sample(99, 88);
    assert_eq!(read_word(&mut model, ALS), 11);
    assert_eq!(read_word(&mut model, WHITE), 22);
}

#[test]
fn shutdown_retains_the_last_pair_and_ignores_later_time_and_samples() {
    let mut model = Veml7700Model::new(injected_inputs(7, 8));
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
fn shutdown_before_the_first_bound_completes_no_conversion() {
    // Renamed from `shutdown_before_the_bound_keeps_the_previous_completed_pair`,
    // which is what this body never established: nothing had completed, so there
    // was no previous pair to keep. The retention claim now has its own test
    // below. A test whose name asserts more than its body is worse than a
    // missing test, because a coverage matrix reads the name.
    let mut model = Veml7700Model::new(injected_inputs(3, 4));
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
fn shutdown_before_the_bound_keeps_the_previous_completed_pair() {
    // Model reaction to `S-25`: shutdown retains the last completed pair.
    let mut model = Veml7700Model::new(injected_inputs(3, 4));
    wake_100ms(&mut model);
    model.advance(RelativeDuration::from_micros(BOUND_100MS_US));
    assert_eq!(read_word(&mut model, ALS), 3);
    assert_eq!(read_word(&mut model, WHITE), 4);

    // A second sample is injected, then the device is shut down one microsecond
    // before the next refresh boundary. The new pair never latches, so both
    // outputs must still read the first.
    model.set_raw_sample(9, 10);
    model.advance(RelativeDuration::from_micros(100_000 - 1));
    freeze_100ms(&mut model);
    model.advance(RelativeDuration::from_micros(BOUND_100MS_US));
    assert_eq!(read_word(&mut model, ALS), 3);
    assert_eq!(read_word(&mut model, WHITE), 4);
}

#[test]
fn repeated_active_configuration_is_rejected_without_mutation() {
    let mut model = Veml7700Model::new(injected_inputs(0, 0));
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
    let mut model = Veml7700Model::new(injected_inputs(0, 0));
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
    let mut once = Veml7700Model::new(injected_inputs(0, 0));
    once.set_raw_sample(42, 43);
    wake_100ms(&mut once);
    once.advance(RelativeDuration::from_micros(BOUND_100MS_US));

    let mut split = Veml7700Model::new(injected_inputs(0, 0));
    split.set_raw_sample(42, 43);
    wake_100ms(&mut split);
    split.advance(RelativeDuration::from_micros(40_000));
    split.advance(RelativeDuration::from_micros(50_000));
    split.advance(RelativeDuration::from_nanos(12_500_000));

    assert_eq!(read_word(&mut once, ALS), read_word(&mut split, ALS));
    assert_eq!(read_word(&mut once, WHITE), read_word(&mut split, WHITE));
    assert_eq!(once.inspect().als_remaining, split.inspect().als_remaining);
    assert_eq!(
        once.inspect().white_remaining,
        split.inspect().white_remaining
    );
}

#[test]
fn reads_do_not_consume_conversion_time() {
    let mut model = Veml7700Model::new(injected_inputs(1, 2));
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
    let mut model = Veml7700Model::new(injected_inputs(5, 6));
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
fn addresses_outside_seven_bit_domain_are_model_limitations() {
    for address in [0x80, 0xFF] {
        let mut model = Veml7700Model::new(injected_inputs(0, 0));
        let before = model.inspect();
        assert_eq!(
            model.write(address, &[CONFIG, 0x01, 0x00]),
            Err(TransportError::Unsupported(Unsupported::AddressOutOfRange(
                address
            )))
        );

        let mut bytes = [0xA5_u8; 2];
        assert_eq!(
            model.write_read(address, &[ID], &mut bytes),
            Err(TransportError::Unsupported(Unsupported::AddressOutOfRange(
                address
            )))
        );
        assert_eq!(bytes, [0xA5; 2]);
        assert_eq!(model.inspect(), before);
    }
}

#[test]
fn threshold_status_is_read_only_and_not_a_device_nack() {
    let mut model = Veml7700Model::new(injected_inputs(0, 0));
    assert_eq!(
        read_word_result(&mut model, THRESHOLD_STATUS),
        Err(TransportError::Unsupported(
            Unsupported::StatusReadWhileMonitorDisabled(THRESHOLD_STATUS)
        ))
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
    let mut model = Veml7700Model::new(injected_inputs(0, 0));
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
    let mut model = Veml7700Model::new(injected_inputs(0, 0));
    let before = model.inspect();

    let [low, high] = 0x0008_u16.to_le_bytes();
    assert_eq!(
        model.write(I2C_ADDRESS, &[POWER_SAVING, low, high]),
        Err(TransportError::Unsupported(Unsupported::PowerSavingWord(
            0x0008
        )))
    );

    let unsupported_configuration = 0x1005_u16;
    let [low, high] = unsupported_configuration.to_le_bytes();
    assert_eq!(
        model.write(I2C_ADDRESS, &[CONFIG, low, high]),
        Err(TransportError::Unsupported(Unsupported::ConfigurationWord(
            unsupported_configuration
        )))
    );
    assert_eq!(model.inspect(), before);
}

#[test]
fn threshold_registers_are_unknown_until_programmed_and_use_little_endian_words() {
    let mut model = Veml7700Model::new(injected_inputs(0, 0));
    for pointer in [LOW_THRESHOLD, HIGH_THRESHOLD] {
        assert_eq!(
            read_word_result(&mut model, pointer),
            Err(TransportError::Unsupported(
                Unsupported::NoProgrammedThreshold(pointer)
            ))
        );
    }

    model
        .write(I2C_ADDRESS, &[LOW_THRESHOLD, 0x34, 0x12])
        .expect("low threshold write");
    model
        .write(I2C_ADDRESS, &[HIGH_THRESHOLD, 0xCD, 0xAB])
        .expect("high threshold write");
    assert_eq!(read_word(&mut model, LOW_THRESHOLD), 0x1234);
    assert_eq!(read_word(&mut model, HIGH_THRESHOLD), 0xABCD);
    assert_eq!(model.inspect().low_threshold, Some(0x1234));
    assert_eq!(model.inspect().high_threshold, Some(0xABCD));

    let mut bytes = [0_u8; 2];
    model
        .write_read(I2C_ADDRESS, &[HIGH_THRESHOLD], &mut bytes)
        .expect("high threshold read");
    assert_eq!(bytes, [0xCD, 0xAB]);
}

#[test]
fn every_documented_power_saving_cadence_refreshes_at_the_exact_boundary() {
    let integrations = [
        (0b0000_u16, 100_u64),
        (0b0001, 200),
        (0b0010, 400),
        (0b0011, 800),
    ];
    let modes = [(0_u16, 500_u64), (1, 1_000), (2, 2_000), (3, 4_000)];
    for (integration_field, integration_ms) in integrations {
        for (mode, sleep_ms) in modes {
            let mut model = Veml7700Model::new(injected_inputs(0, 0));
            let power_word = (mode << 1) | 1;
            write_word(&mut model, POWER_SAVING, power_word);
            model.set_raw_sample(1, 1);
            // Gain ×2 is the `S-21` cadence domain; `S-22` remains undefined.
            let shutdown = (0b01 << 11) | (integration_field << 6) | 1;
            let active = (0b01 << 11) | (integration_field << 6);
            write_word(&mut model, CONFIG, shutdown);
            write_word(&mut model, CONFIG, active);
            let first_us = 2_500 + integration_ms * 1_000;
            model.advance(RelativeDuration::from_micros(first_us));
            model.set_raw_sample(2, 2);

            let refresh_us = (integration_ms + sleep_ms) * 1_000;
            model.advance(RelativeDuration::from_micros(refresh_us - 1));
            assert_eq!(read_word(&mut model, ALS), 1);
            model.advance(RelativeDuration::from_micros(1));
            assert_eq!(read_word(&mut model, ALS), 2);
        }
    }
}

#[test]
fn enabled_power_saving_rejects_undocumented_25_and_50_ms_cadence() {
    for integration_field in [0b1100_u16, 0b1000] {
        let mut model = Veml7700Model::new(injected_inputs(0, 0));
        write_word(&mut model, POWER_SAVING, 1);
        let active = (0b01 << 11) | (integration_field << 6);
        let [low, high] = active.to_le_bytes();
        assert_eq!(
            model.write(I2C_ADDRESS, &[CONFIG, low, high]),
            Err(TransportError::Unsupported(
                Unsupported::UnsupportedPowerSavingDomain {
                    configuration: active,
                    power_saving: 1,
                }
            ))
        );
        assert_eq!(model.inspect().configuration, 0x0001);
    }
}

/// Enabled cadence outside the `S-21` gain domain is unsupported.
///
/// The sibling test above holds gain at ×2 and varies integration time, so it
/// cannot fail if the gain restriction is dropped. This one is the reverse: it
/// holds integration time at 100 ms -- squarely inside the `S-21` table -- so
/// the gain field is the only reason the write can be refused. Deleting the
/// gain check in `refresh_interval_ns` must fail exactly here.
///
/// `S-22` is undefined, so the model declines rather than assuming the ×2
/// refresh times carry to other gains.
#[test]
fn enabled_power_saving_rejects_gains_outside_the_documented_cadence_domain() {
    // Gain fields ×1, ×1/8, ×1/4 -- every encoding except the ×2 the table covers.
    for gain_field in [0b00_u16, 0b10, 0b11] {
        let mut model = Veml7700Model::new(injected_inputs(0, 0));
        write_word(&mut model, POWER_SAVING, 1);
        // Integration field 0b0000 is 100 ms, a documented `S-21` column.
        let active = gain_field << 11;
        let [low, high] = active.to_le_bytes();
        assert_eq!(
            model.write(I2C_ADDRESS, &[CONFIG, low, high]),
            Err(TransportError::Unsupported(
                Unsupported::UnsupportedPowerSavingDomain {
                    configuration: active,
                    power_saving: 1,
                }
            ))
        );
        assert_eq!(model.inspect().configuration, 0x0001);
    }

    // Positive control: the same word at gain ×2 is accepted, so the rejections
    // above are the gain domain and not the integration time or the power word.
    let mut model = Veml7700Model::new(injected_inputs(0, 0));
    write_word(&mut model, POWER_SAVING, 1);
    let active = 0b01 << 11;
    write_word(&mut model, CONFIG, active);
    assert_eq!(model.inspect().configuration, active);
}

#[test]
fn injected_channel_skew_preserves_independent_refresh_generations() {
    let mut model = Veml7700Model::new(injected_inputs(0, 0));
    model.set_white_phase_offset(RelativeDuration::from_micros(10));
    model.set_raw_sample(10, 20);
    wake_100ms(&mut model);
    model.advance(RelativeDuration::from_micros(BOUND_100MS_US));
    assert_eq!(read_word(&mut model, ALS), 10);
    assert!(matches!(
        read_word_result(&mut model, WHITE),
        Err(TransportError::Unsupported(
            Unsupported::NoCompletedConversion(WHITE)
        ))
    ));
    model.advance(RelativeDuration::from_micros(10));
    assert_eq!(read_word(&mut model, WHITE), 20);

    model.set_raw_sample(30, 40);
    model.advance(RelativeDuration::from_micros(100_000 - 10));
    assert_eq!(read_word(&mut model, ALS), 30);
    assert_eq!(read_word(&mut model, WHITE), 20);
    model.advance(RelativeDuration::from_micros(10));
    assert_eq!(read_word(&mut model, WHITE), 40);
}

#[test]
fn every_protect_number_leaves_threshold_qualification_undefined() {
    // Model reaction to `S-16`, `S-39`, `S-49`, and `S-50`: accept every field
    // encoding while declining to manufacture a status oracle.
    for persistence_field in [0_u16, 1, 2, 3] {
        let mut model = Veml7700Model::new(injected_inputs(0, 0));
        write_word(&mut model, LOW_THRESHOLD, 100);
        write_word(&mut model, HIGH_THRESHOLD, 200);
        model.set_raw_sample(250, 0);
        let active_monitor = (persistence_field << 4) | 0x0002;
        write_word(&mut model, CONFIG, active_monitor);

        let expected = Err(TransportError::Unsupported(
            Unsupported::UndefinedQualificationRule {
                configuration: active_monitor,
            },
        ));
        assert_eq!(read_word_result(&mut model, THRESHOLD_STATUS), expected);

        // Waiting changes no evidence and therefore cannot resolve the model
        // boundary.
        model.advance(RelativeDuration::from_micros(BOUND_100MS_US + 16 * 100_000));
        assert_eq!(read_word_result(&mut model, THRESHOLD_STATUS), expected);

        // The register the driver actually programs is unaffected: this is a
        // refusal to model qualification, not a rejection of the field.
        assert_eq!(read_word(&mut model, CONFIG), active_monitor);
    }
}

#[test]
#[should_panic(expected = "initial power-saving mode contradicts S-48")]
fn construction_rejects_an_injected_power_saving_mode_that_contradicts_s48() {
    let _ = ModelInputs::new(0, 0, 0b010);
}

#[test]
fn undefined_initial_reserved_bits_are_observable_but_not_interpreted() {
    let mut model = Veml7700Model::new(ModelInputs::new(0, 0, 0x8000));
    assert_eq!(read_word(&mut model, POWER_SAVING), 0x8000);

    let [low, high] = ACTIVE_100MS.to_le_bytes();
    assert_eq!(
        model.write(I2C_ADDRESS, &[CONFIG, low, high]),
        Err(TransportError::Unsupported(
            Unsupported::UnsupportedPowerSavingDomain {
                configuration: ACTIVE_100MS,
                power_saving: 0x8000,
            }
        ))
    );
}

#[test]
fn disabling_the_monitor_does_not_invent_status_history() {
    let mut model = Veml7700Model::new(injected_inputs(0, 0));
    write_word(&mut model, LOW_THRESHOLD, 100);
    write_word(&mut model, HIGH_THRESHOLD, 200);
    model.set_raw_sample(250, 0);
    write_word(&mut model, CONFIG, 0x0002);
    model.advance(RelativeDuration::from_micros(BOUND_100MS_US));
    assert!(matches!(
        read_word_result(&mut model, THRESHOLD_STATUS),
        Err(TransportError::Unsupported(
            Unsupported::UndefinedQualificationRule { .. }
        ))
    ));

    // Disabling is an accepted model transition, but `S-42` does not give the
    // model a value or history to return afterwards.
    write_word(&mut model, CONFIG, 0x0000);
    model.advance(RelativeDuration::from_micros(1_300_000));
    assert_eq!(
        read_word_result(&mut model, THRESHOLD_STATUS),
        Err(TransportError::Unsupported(
            Unsupported::StatusReadWhileMonitorDisabled(THRESHOLD_STATUS)
        ))
    );
}

#[test]
fn a_large_advance_processes_multiple_autonomous_refreshes() {
    let mut model = Veml7700Model::new(injected_inputs(7, 8));
    wake_100ms(&mut model);
    model.advance(RelativeDuration::from_micros(BOUND_100MS_US + 20 * 100_000));
    assert_eq!(read_word(&mut model, ALS), 7);
    assert_eq!(read_word(&mut model, WHITE), 8);
    assert_eq!(
        model.inspect().als_remaining,
        Some(RelativeDuration::from_micros(100_000))
    );
}

/// Every integration time, immediately before and exactly at its first bound.
///
/// Only 100 ms was covered. A conversion boundary computed from the wrong
/// integration constant is exactly the defect this model exists to catch in the
/// driver, so leaving five of six untested left the oracle itself unchecked at
/// the value it is asked about most.
///
/// The words are `ALS_SD = 0` with gain ×1/8 and the integration field from
/// `docs/HARDWARE_CONTRACT.md` `S-15`. Note again that the encoding order is not the
/// magnitude order: `1100` is the *shortest* time.
#[test]
fn every_integration_time_latches_exactly_at_its_ideal_completion_point() {
    // (integration field, nominal microseconds)
    const TIMES: [(u16, u64); 6] = [
        (0b1100, 25_000),
        (0b1000, 50_000),
        (0b0000, 100_000),
        (0b0001, 200_000),
        (0b0010, 400_000),
        (0b0011, 800_000),
    ];

    for (field, nominal_us) in TIMES {
        let prepared = (0b10 << 11) | (field << 6) | 0x0001;
        let active = prepared & !0x0001;
        let bound_us = 2_500 + nominal_us;

        // Immediately before: nothing has completed, and the outputs say so
        // rather than reporting a zero-valued conversion.
        let mut model = Veml7700Model::new(injected_inputs(11, 22));
        write_word(&mut model, CONFIG, prepared);
        write_word(&mut model, CONFIG, active);
        model.advance(RelativeDuration::from_micros(bound_us - 1));
        assert_eq!(
            read_word_result(&mut model, ALS),
            Err(TransportError::Unsupported(
                Unsupported::NoCompletedConversion(ALS)
            )),
            "integration field {field:04b} completed early"
        );

        // Exactly at: both channels latch the held pair.
        let mut model = Veml7700Model::new(injected_inputs(11, 22));
        write_word(&mut model, CONFIG, prepared);
        write_word(&mut model, CONFIG, active);
        model.advance(RelativeDuration::from_micros(bound_us));
        assert_eq!(
            read_word(&mut model, ALS),
            11,
            "integration field {field:04b} did not latch ALS at its bound"
        );
        assert_eq!(
            read_word(&mut model, WHITE),
            22,
            "integration field {field:04b} did not latch white at its bound"
        );
    }
}

#[test]
fn advance_rejects_a_step_beyond_the_bound_without_mutating() {
    let mut model = Veml7700Model::new(injected_inputs(5, 6));
    wake_100ms(&mut model);
    let before = model.clone();

    let excessive = RelativeDuration::from_nanos(MAX_ADVANCE.as_nanos() + 1);
    let result = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| {
        model.advance(excessive);
    }));
    assert!(result.is_err(), "an over-long advance must be rejected");

    // Rejected *before* mutation: the model a caller observes afterwards is the
    // one it had. Checking this is the difference between a bound and a
    // partially applied advance that happens to stop early.
    assert_eq!(model, before);

    // The bound itself is accepted.
    model.advance(MAX_ADVANCE);
}

#[test]
fn microsecond_durations_reject_overflow_instead_of_saturating() {
    // Saturation silently substituted ~584 years for whatever was asked, and
    // every later assertion was then made against a timeline nobody chose.
    assert_eq!(RelativeDuration::try_from_micros(u64::MAX), None);
    assert_eq!(
        RelativeDuration::try_from_micros(1_000),
        Some(RelativeDuration::from_nanos(1_000_000))
    );
    let overflowed = std::panic::catch_unwind(|| RelativeDuration::from_micros(u64::MAX));
    assert!(overflowed.is_err());
}

#[test]
fn construction_carries_the_injected_pair_without_a_separate_call() {
    // The zero-ambient fabrication this construction input exists to prevent:
    // waking without injecting used to yield a conversion of 0, which is a
    // plausible reading and therefore failed nothing.
    let mut model = Veml7700Model::new(injected_inputs(0x1234, 0x5678));
    wake_100ms(&mut model);
    model.advance(RelativeDuration::from_micros(BOUND_100MS_US));
    assert_eq!(read_word(&mut model, ALS), 0x1234);
    assert_eq!(read_word(&mut model, WHITE), 0x5678);
}

#[test]
fn an_injected_white_phase_offset_is_carried_from_construction() {
    let offset = RelativeDuration::from_micros(10);
    let inputs = injected_inputs(7, 8).with_white_phase_offset(offset);
    let mut model = Veml7700Model::new(inputs);
    wake_100ms(&mut model);

    // At the ALS bound the white channel is still short by the offset.
    model.advance(RelativeDuration::from_micros(BOUND_100MS_US));
    assert_eq!(read_word(&mut model, ALS), 7);
    assert_eq!(
        read_word_result(&mut model, WHITE),
        Err(TransportError::Unsupported(
            Unsupported::NoCompletedConversion(WHITE)
        ))
    );
    model.advance(offset);
    assert_eq!(read_word(&mut model, WHITE), 8);
}
