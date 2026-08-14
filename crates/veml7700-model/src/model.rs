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
    threshold_status: u16,
    held_als: u16,
    held_white: u16,
    completed_als: Option<u16>,
    completed_white: Option<u16>,
    als_remaining_ns: Option<u64>,
    white_remaining_ns: Option<u64>,
    white_phase_offset_ns: u64,
    low_streak: u8,
    high_streak: u8,
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
    /// Threshold-status register word.
    pub threshold_status: u16,
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

impl Veml7700Model {
    /// Construct the documented reset/default state needed by this slice.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            configuration: RESET_CONFIGURATION,
            power_saving: RESET_POWER_SAVING,
            low_threshold: None,
            high_threshold: None,
            threshold_status: 0,
            held_als: 0,
            held_white: 0,
            completed_als: None,
            completed_white: None,
            als_remaining_ns: None,
            white_remaining_ns: None,
            white_phase_offset_ns: 0,
            low_streak: 0,
            high_streak: 0,
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
    pub const fn set_white_phase_offset(&mut self, offset: RelativeDuration) {
        self.white_phase_offset_ns = offset.as_nanos();
    }

    /// Advance autonomous refresh progress by a non-negative relative duration.
    pub fn advance(&mut self, elapsed: RelativeDuration) {
        let mut step = elapsed.as_nanos();
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
            POINTER_THRESHOLD_STATUS => Ok(self.threshold_status),
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
            self.white_remaining_ns = Some(first.saturating_add(self.white_phase_offset_ns));
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
            self.low_streak = 0;
            self.high_streak = 0;
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

    const fn update_threshold_status(&mut self) {
        if !threshold_monitor_is_enabled(self.configuration) {
            self.low_streak = 0;
            self.high_streak = 0;
            return;
        }
        let (Some(low), Some(high)) = (self.low_threshold, self.high_threshold) else {
            return;
        };
        self.low_streak = if self.held_als < low {
            self.low_streak.saturating_add(1)
        } else {
            0
        };
        self.high_streak = if self.held_als > high {
            self.high_streak.saturating_add(1)
        } else {
            0
        };
        let required = persistence_count(self.configuration);
        if self.low_streak >= required {
            self.threshold_status |= STATUS_LOW_BIT;
        }
        if self.high_streak >= required {
            self.threshold_status |= STATUS_HIGH_BIT;
        }
    }
}

impl Default for Veml7700Model {
    fn default() -> Self {
        Self::new()
    }
}
