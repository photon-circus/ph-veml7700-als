//! Quiescent VEML7700 state machine for the declared behavioral slice.

use crate::duration::RelativeDuration;
use crate::error::{NoAcknowledgeSource, TransportError, Unsupported};
use crate::registers::{
    DEVICE_ID, I2C_ADDRESS, POINTER_ALS, POINTER_CONFIGURATION, POINTER_HIGH_THRESHOLD, POINTER_ID,
    POINTER_LOW_THRESHOLD, POINTER_POWER_SAVING, POINTER_THRESHOLD_STATUS, POINTER_WHITE,
    RESET_CONFIGURATION, RESET_POWER_SAVING, configuration_fields_are_supported,
    conversion_bound_ns, integration_field, is_shutdown, persistence_count,
    power_saving_is_supported, refresh_interval_ns, threshold_monitor_is_enabled, without_monitor,
    without_shutdown,
};

const STATUS_LOW_BIT: u16 = 1 << 15;
const STATUS_HIGH_BIT: u16 = 1 << 14;

/// Independent VEML7700 predictor for the declared I²C behavioral slice.
///
/// The model remains quiescent between explicit inputs. Bus operations do not
/// consume invented duration, and [`inspect`](Self::inspect) does not mutate state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Veml7700Model {
    configuration: u16,
    power_saving: u16,
    low_threshold: Option<u16>,
    high_threshold: Option<u16>,
    threshold_status: Option<u16>,
    held_als: u16,
    held_white: u16,
    completed_als: Option<u16>,
    completed_white: Option<u16>,
    als_remaining_ns: Option<u64>,
    white_remaining_ns: Option<u64>,
    white_phase_offset_ns: u64,
}

/// Frozen observation of model state. Calling this method does not mutate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Inspection {
    /// Configuration register word.
    pub configuration: u16,
    /// Power-saving register word.
    pub power_saving: u16,
    /// Programmed low-threshold register word, if observed.
    pub low_threshold: Option<u16>,
    /// Programmed high-threshold register word, if observed.
    pub high_threshold: Option<u16>,
    /// Threshold-status register word, if a monitored ALS refresh established it.
    pub threshold_status: Option<u16>,
    /// Last completed ALS output word, if this model has completed a conversion.
    pub als: Option<u16>,
    /// Last completed white output word, if this model has completed a conversion.
    pub white: Option<u16>,
    /// Currently held injected ALS sample.
    pub held_als: u16,
    /// Currently held injected white sample.
    pub held_white: u16,
    /// Remaining progress toward the next ALS refresh while active.
    pub als_remaining: Option<RelativeDuration>,
    /// Remaining progress toward the next white refresh while active.
    pub white_remaining: Option<RelativeDuration>,
}

/// Largest single [`Veml7700Model::advance`] step: one hour of virtual time.
///
/// Generous against every cadence this model can select — the longest is 800 ms
/// integration plus a 4 s Mode 4 refresh, so an hour is roughly 750 refreshes
/// and about 110,000 loop iterations at the shortest cadence. Both are trivial.
///
/// The bound exists because the alternative to a bound is a hang. It is not a
/// claim that an hour is meaningful to the device; it is the point past which a
/// single step is more likely a units mistake than a scenario.
///
/// **This is a model-domain constraint, not a performance guard.** D-031 records
/// why the loop rejects rather than batching event-free recurrence, and what a
/// maintainer has to keep intact when changing this value — notably that the
/// same constant is what makes the white-channel wake edge provably
/// non-overflowing.
pub const MAX_ADVANCE: RelativeDuration = RelativeDuration::from_nanos(3_600_000_000_000);

/// Stimuli a harness must supply before the model can produce a conversion.
///
/// # Why these are required rather than defaulted
///
/// `Veml7700Model::new()` used to zero all three. A harness that woke the model
/// without calling `set_raw_sample` then received a conversion reporting zero
/// ambient light — a reading it never supplied and the model had no basis for.
///
/// Nothing failed, which is the problem. Zero is a plausible ALS value, so the
/// fabricated sample flowed through conversions, threshold comparisons and
/// driver-versus-model traces looking exactly like an injected one. This is the
/// same shape as the persistence rule removed in #56: **an invented value does
/// not produce a failure, it produces agreement.**
///
/// Requiring them at construction makes the omission unrepresentable rather
/// than merely discouraged. A harness that has not decided what the device is
/// looking at cannot build a model, which is the correct outcome — that
/// decision is the harness's to make, and defaulting it silently made it the
/// model's.
///
/// `Default` is deliberately **not** implemented. It would reintroduce the
/// zeroed construction through a different name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetainedInputs {
    /// Raw ALS counts available to a completed conversion. Not ambient lux.
    pub als_counts: u16,
    /// Raw white-channel counts available to a completed conversion.
    pub white_counts: u16,
    /// Offset applied to white-channel wake edges.
    ///
    /// Test scheduling topology, not a claim about silicon phase relationship.
    /// [`RelativeDuration::ZERO`] is the ordinary choice and says so explicitly.
    pub white_phase_offset: RelativeDuration,
}

impl RetainedInputs {
    /// Construct with no white-channel phase offset.
    ///
    /// The common case, written so a harness that does not care about channel
    /// skew still states the sample it is injecting.
    #[must_use]
    pub const fn new(als_counts: u16, white_counts: u16) -> Self {
        Self {
            als_counts,
            white_counts,
            white_phase_offset: RelativeDuration::ZERO,
        }
    }

    /// Return these inputs with an injected white-channel phase offset.
    #[must_use]
    pub const fn with_white_phase_offset(mut self, offset: RelativeDuration) -> Self {
        self.white_phase_offset = offset;
        self
    }
}

impl Veml7700Model {
    /// Construct the documented reset/default state needed by this slice.
    ///
    /// The retained inputs are **required**, not defaulted. See
    /// [`RetainedInputs`] for why.
    ///
    /// # Panics
    ///
    /// If the white-channel phase offset exceeds [`MAX_ADVANCE`].
    #[must_use]
    pub const fn new(retained: RetainedInputs) -> Self {
        assert!(
            retained.white_phase_offset.as_nanos() <= MAX_ADVANCE.as_nanos(),
            "white-channel phase offset exceeds the maximum advance bound"
        );
        Self {
            configuration: RESET_CONFIGURATION,
            power_saving: RESET_POWER_SAVING,
            low_threshold: None,
            high_threshold: None,
            threshold_status: None,
            held_als: retained.als_counts,
            held_white: retained.white_counts,
            completed_als: None,
            completed_white: None,
            als_remaining_ns: None,
            white_remaining_ns: None,
            white_phase_offset_ns: retained.white_phase_offset.as_nanos(),
        }
    }

    /// Replace the persistent raw ALS/white pair available to later refreshes.
    ///
    /// This does not latch either output register.
    pub const fn set_raw_sample(&mut self, als_counts: u16, white_counts: u16) {
        self.held_als = als_counts;
        self.held_white = white_counts;
    }

    /// Set an injected white-channel offset for future shutdown-to-active edges.
    ///
    /// The offset is test scheduling topology, not a claim about silicon phase.
    /// Changing it while active does not alter the already scheduled refresh.
    ///
    /// # Panics
    ///
    /// If `offset` exceeds [`MAX_ADVANCE`]. The bound is what makes the white
    /// wake edge provably non-overflowing rather than saturating.
    pub const fn set_white_phase_offset(&mut self, offset: RelativeDuration) {
        assert!(
            offset.as_nanos() <= MAX_ADVANCE.as_nanos(),
            "white-channel phase offset exceeds the maximum advance bound"
        );
        self.white_phase_offset_ns = offset.as_nanos();
    }

    /// Advance autonomous refresh progress by a non-negative relative duration.
    ///
    /// # Panics
    ///
    /// If `elapsed` exceeds [`MAX_ADVANCE`].
    ///
    /// The loop below runs once per refresh event, and the smallest recurrence
    /// this model can produce is 130 % of 25 ms — about 32.5 ms. A `u64::MAX`
    /// nanosecond input therefore implies on the order of **568 billion**
    /// iterations: not an error, just a hang, which is the worst way for a test
    /// suite to report a bad argument.
    ///
    /// The input is rejected **before any mutation**, so a caller that catches
    /// the panic observes an unchanged model rather than a partially advanced
    /// one.
    pub fn advance(&mut self, elapsed: RelativeDuration) {
        assert!(
            elapsed <= MAX_ADVANCE,
            "advance of {} ns exceeds the {} ns bound; a single step longer than an hour of virtual time is a defect in the harness, not a scenario",
            elapsed.as_nanos(),
            MAX_ADVANCE.as_nanos()
        );
        let mut step = elapsed.as_nanos();
        // Terminates because refresh_interval_ns never returns Some(0); smallest interval is 130% of 25 ms.
        loop {
            let Some(next) = Self::next_event(self.als_remaining_ns, self.white_remaining_ns)
            else {
                return;
            };
            if step < next {
                self.als_remaining_ns = Self::subtract(self.als_remaining_ns, step);
                self.white_remaining_ns = Self::subtract(self.white_remaining_ns, step);
                return;
            }

            self.als_remaining_ns = Self::subtract(self.als_remaining_ns, next);
            self.white_remaining_ns = Self::subtract(self.white_remaining_ns, next);
            step -= next;

            if self.als_remaining_ns == Some(0) {
                self.complete_als_refresh();
            }
            if self.white_remaining_ns == Some(0) {
                self.complete_white_refresh();
            }
            if step == 0 {
                return;
            }
        }
    }

    /// I²C write of a complete register word: `[pointer, low, high]`.
    pub fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), TransportError> {
        Self::require_address(address)?;
        let [pointer, low, high] = <[u8; 3]>::try_from(bytes)
            .map_err(|_| TransportError::Unsupported(Unsupported::TransactionShape))?;
        let word = u16::from_le_bytes([low, high]);
        match pointer {
            POINTER_CONFIGURATION => self.write_configuration(word),
            POINTER_HIGH_THRESHOLD | POINTER_LOW_THRESHOLD => self.write_threshold(pointer, word),
            POINTER_POWER_SAVING => self.write_power_saving(word),
            other => Err(TransportError::Unsupported(Unsupported::RegisterPointer(
                other,
            ))),
        }
    }

    /// Combined write of a register pointer and read of a 16-bit little-endian word.
    pub fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), TransportError> {
        Self::require_address(address)?;
        let [pointer] = <[u8; 1]>::try_from(write)
            .map_err(|_| TransportError::Unsupported(Unsupported::TransactionShape))?;
        if read.len() != 2 {
            return Err(TransportError::Unsupported(Unsupported::TransactionShape));
        }
        let word = self.read_word(pointer)?;
        read.copy_from_slice(&word.to_le_bytes());
        Ok(())
    }

    /// Return a non-mutating snapshot of retained state.
    #[must_use]
    pub const fn inspect(&self) -> Inspection {
        Inspection {
            configuration: self.configuration,
            power_saving: self.power_saving,
            low_threshold: self.low_threshold,
            high_threshold: self.high_threshold,
            threshold_status: self.threshold_status,
            als: self.completed_als,
            white: self.completed_white,
            held_als: self.held_als,
            held_white: self.held_white,
            als_remaining: Self::duration(self.als_remaining_ns),
            white_remaining: Self::duration(self.white_remaining_ns),
        }
    }

    const fn duration(nanos: Option<u64>) -> Option<RelativeDuration> {
        match nanos {
            Some(value) => Some(RelativeDuration::from_nanos(value)),
            None => None,
        }
    }

    const fn subtract(remaining: Option<u64>, elapsed: u64) -> Option<u64> {
        match remaining {
            Some(value) => Some(value.saturating_sub(elapsed)),
            None => None,
        }
    }

    const fn next_event(als: Option<u64>, white: Option<u64>) -> Option<u64> {
        match (als, white) {
            (Some(als), Some(white)) => Some(if als < white { als } else { white }),
            (Some(als), None) => Some(als),
            (None, Some(white)) => Some(white),
            (None, None) => None,
        }
    }

    const fn require_address(address: u8) -> Result<(), TransportError> {
        if address > 0x7F {
            Err(TransportError::Unsupported(Unsupported::AddressOutOfRange(
                address,
            )))
        } else if address == I2C_ADDRESS {
            Ok(())
        } else {
            Err(TransportError::NoAcknowledge {
                source: NoAcknowledgeSource::Address,
            })
        }
    }

    const fn read_word(&self, pointer: u8) -> Result<u16, TransportError> {
        match pointer {
            POINTER_CONFIGURATION => Ok(self.configuration),
            POINTER_HIGH_THRESHOLD => self.read_threshold(self.high_threshold, pointer),
            POINTER_LOW_THRESHOLD => self.read_threshold(self.low_threshold, pointer),
            POINTER_POWER_SAVING => Ok(self.power_saving),
            POINTER_ALS => self.read_completed(self.completed_als, pointer),
            POINTER_WHITE => self.read_completed(self.completed_white, pointer),
            POINTER_THRESHOLD_STATUS => self.read_status(self.threshold_status, pointer),
            POINTER_ID => Ok(DEVICE_ID),
            other => Err(TransportError::Unsupported(Unsupported::RegisterPointer(
                other,
            ))),
        }
    }

    const fn read_threshold(&self, value: Option<u16>, pointer: u8) -> Result<u16, TransportError> {
        match value {
            Some(value) => Ok(value),
            None => Err(TransportError::Unsupported(
                Unsupported::NoProgrammedThreshold(pointer),
            )),
        }
    }

    const fn read_completed(&self, value: Option<u16>, pointer: u8) -> Result<u16, TransportError> {
        match value {
            Some(value) => Ok(value),
            None => Err(TransportError::Unsupported(
                Unsupported::NoCompletedConversion(pointer),
            )),
        }
    }

    const fn read_status(&self, value: Option<u16>, pointer: u8) -> Result<u16, TransportError> {
        // Checked before the `None` arm on purpose. Both would report "no status
        // available", but they mean opposite things to a caller: NoQualifiedStatus
        // says *not yet*, and waiting resolves it. This says the rule that would
        // decide it was never written down, so waiting never does.
        if persistence_count(self.configuration) > 1 {
            return Err(TransportError::Unsupported(
                Unsupported::UndefinedQualificationRule {
                    configuration: self.configuration,
                },
            ));
        }
        match value {
            Some(value) => Ok(value),
            None => Err(TransportError::Unsupported(Unsupported::NoQualifiedStatus(
                pointer,
            ))),
        }
    }

    const fn write_threshold(&mut self, pointer: u8, word: u16) -> Result<(), TransportError> {
        if threshold_monitor_is_enabled(self.configuration) {
            return Err(TransportError::Unsupported(
                Unsupported::ThresholdWriteWhileMonitoring(pointer),
            ));
        }
        if pointer == POINTER_HIGH_THRESHOLD {
            self.high_threshold = Some(word);
        } else {
            self.low_threshold = Some(word);
        }
        Ok(())
    }

    const fn write_configuration(&mut self, word: u16) -> Result<(), TransportError> {
        if !configuration_fields_are_supported(word) {
            return Err(TransportError::Unsupported(Unsupported::ConfigurationWord(
                word,
            )));
        }
        if conversion_bound_ns(word).is_none() {
            return Err(TransportError::Unsupported(
                Unsupported::ReservedIntegrationTime(integration_field(word)),
            ));
        }
        if !is_shutdown(word) && refresh_interval_ns(word, self.power_saving).is_none() {
            return Err(TransportError::Unsupported(
                Unsupported::UndocumentedPowerSavingCadence {
                    configuration: word,
                    power_saving: self.power_saving,
                },
            ));
        }
        if threshold_monitor_is_enabled(word)
            && (self.low_threshold.is_none() || self.high_threshold.is_none())
        {
            return Err(TransportError::Unsupported(
                Unsupported::IncompleteThresholdDomain,
            ));
        }

        if is_shutdown(self.configuration) {
            self.write_configuration_from_shutdown(word)
        } else {
            self.write_configuration_while_active(word)
        }
    }

    const fn write_configuration_from_shutdown(&mut self, word: u16) -> Result<(), TransportError> {
        if !is_shutdown(word) {
            let Some(first) = conversion_bound_ns(word) else {
                return Err(TransportError::Unsupported(
                    Unsupported::ReservedIntegrationTime(integration_field(word)),
                ));
            };
            self.als_remaining_ns = Some(first);
            // Not saturating. Saturation here would silently reschedule the
            // white channel to a boundary nobody chose, and the test asserting
            // against it would still pass. The sum cannot overflow because the
            // offset is bounded by MAX_ADVANCE wherever it enters the model and
            // `first` is at most the wake allowance plus 130 % of 800 ms -- so
            // this panic is unreachable today, and stays loud if that bound is
            // ever loosened.
            let Some(white_first) = first.checked_add(self.white_phase_offset_ns) else {
                panic!("white-channel wake edge overflows nanosecond resolution")
            };
            self.white_remaining_ns = Some(white_first);
        }
        self.configuration = word;
        Ok(())
    }

    const fn write_configuration_while_active(&mut self, word: u16) -> Result<(), TransportError> {
        if is_shutdown(word) && without_shutdown(word) == without_shutdown(self.configuration) {
            self.als_remaining_ns = None;
            self.white_remaining_ns = None;
            self.configuration = word;
            return Ok(());
        }

        let disabling_monitor = threshold_monitor_is_enabled(self.configuration)
            && !threshold_monitor_is_enabled(word)
            && !is_shutdown(word)
            && without_monitor(word) == without_monitor(self.configuration);
        if disabling_monitor {
            self.configuration = word;
            return Ok(());
        }

        Err(TransportError::Unsupported(
            Unsupported::MidConversionReconfiguration,
        ))
    }

    const fn write_power_saving(&mut self, word: u16) -> Result<(), TransportError> {
        if !power_saving_is_supported(word) {
            return Err(TransportError::Unsupported(Unsupported::PowerSavingWord(
                word,
            )));
        }
        if !is_shutdown(self.configuration) && word != self.power_saving {
            return Err(TransportError::Unsupported(
                Unsupported::ActivePowerSavingReconfiguration,
            ));
        }
        self.power_saving = word;
        Ok(())
    }

    const fn complete_als_refresh(&mut self) {
        self.completed_als = Some(self.held_als);
        self.update_threshold_status();
        self.als_remaining_ns = refresh_interval_ns(self.configuration, self.power_saving);
    }

    const fn complete_white_refresh(&mut self) {
        self.completed_white = Some(self.held_white);
        self.white_remaining_ns = refresh_interval_ns(self.configuration, self.power_saving);
    }

    /// Qualify threshold status for a protect number of one only.
    ///
    /// Above one, the sources give the counting condition but not a complete
    /// rule. The application note says a flag is set *only when* `ALS_PERS`
    /// measurements stay above or below the threshold — necessary, not stated
    /// to be sufficient — and says nothing about what a non-qualifying
    /// measurement does to a partial run. Counting requires both. This model
    /// therefore establishes nothing above one, and `read_status` reports
    /// `UndefinedQualificationRule` rather than a value derived from the half
    /// nobody wrote down. See D-030 and `docs/HARDWARE_CONTRACT.md` §9.
    const fn update_threshold_status(&mut self) {
        if !threshold_monitor_is_enabled(self.configuration) {
            return;
        }
        if persistence_count(self.configuration) > 1 {
            return;
        }
        let (Some(low), Some(high)) = (self.low_threshold, self.high_threshold) else {
            return;
        };
        let Some(als) = self.completed_als else {
            return;
        };
        let mut status = match self.threshold_status {
            Some(value) => value,
            None => 0,
        };
        if als < low {
            status |= STATUS_LOW_BIT;
        }
        if als > high {
            status |= STATUS_HIGH_BIT;
        }
        self.threshold_status = Some(status);
    }
}

// `Default` is deliberately not implemented. It would have to invent the
// retained sample, which is the fabrication `RetainedInputs` exists to prevent
// -- and it would do so under a name that reads like a safe convenience.
