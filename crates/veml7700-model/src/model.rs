//! Quiescent VEML7700 state machine for the probe/`measure_once` slice.

use crate::duration::RelativeDuration;
use crate::error::{NoAcknowledgeSource, TransportError, Unsupported};
use crate::registers::{
    DEVICE_ID, I2C_ADDRESS, POINTER_ALS, POINTER_CONFIGURATION, POINTER_ID, POINTER_POWER_SAVING,
    POINTER_WHITE, RESET_CONFIGURATION, RESET_POWER_SAVING, configuration_fields_are_supported,
    conversion_bound_ns, integration_field, is_shutdown, power_saving_is_supported,
    without_shutdown,
};

/// Independent VEML7700 predictor for probe and one successful `measure_once`.
///
/// The model remains quiescent between explicit inputs. Bus operations do not
/// consume invented duration, and [`inspect`](Self::inspect) does not mutate state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Veml7700Model {
    configuration: u16,
    power_saving: u16,
    held_als: u16,
    held_white: u16,
    completed_als: Option<u16>,
    completed_white: Option<u16>,
    remaining_ns: Option<u64>,
}

/// Frozen observation of model state. Calling this method does not mutate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Inspection {
    /// Configuration register word.
    pub configuration: u16,
    /// Power-saving register word.
    pub power_saving: u16,
    /// Last completed ALS output word, if this model has completed a conversion.
    pub als: Option<u16>,
    /// Last completed white output word, if this model has completed a conversion.
    pub white: Option<u16>,
    /// Currently held injected ALS sample.
    pub held_als: u16,
    /// Currently held injected white sample.
    pub held_white: u16,
    /// Remaining conversion progress, if a conversion is in progress.
    pub remaining: Option<RelativeDuration>,
}

impl Veml7700Model {
    /// Construct the documented reset/default state needed by this slice.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            configuration: RESET_CONFIGURATION,
            power_saving: RESET_POWER_SAVING,
            held_als: 0,
            held_white: 0,
            completed_als: None,
            completed_white: None,
            remaining_ns: None,
        }
    }

    /// Replace the persistent raw ALS/white pair available to a later conversion.
    ///
    /// This does not latch the output registers.
    pub const fn set_raw_sample(&mut self, als_counts: u16, white_counts: u16) {
        self.held_als = als_counts;
        self.held_white = white_counts;
    }

    /// Advance conversion progress by a non-negative relative duration.
    ///
    /// Extra time after completion is discarded. No absolute time is stored.
    pub const fn advance(&mut self, elapsed: RelativeDuration) {
        let Some(remaining) = self.remaining_ns else {
            return;
        };
        let step = elapsed.as_nanos();
        if step >= remaining {
            self.latch_held_pair();
            self.remaining_ns = None;
        } else {
            self.remaining_ns = Some(remaining - step);
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
        let bytes = word.to_le_bytes();
        read.copy_from_slice(&bytes);
        Ok(())
    }

    /// Return a non-mutating snapshot of retained state.
    #[must_use]
    pub const fn inspect(&self) -> Inspection {
        Inspection {
            configuration: self.configuration,
            power_saving: self.power_saving,
            als: self.completed_als,
            white: self.completed_white,
            held_als: self.held_als,
            held_white: self.held_white,
            remaining: match self.remaining_ns {
                Some(nanos) => Some(RelativeDuration::from_nanos(nanos)),
                None => None,
            },
        }
    }

    const fn require_address(address: u8) -> Result<(), TransportError> {
        if address > 0x7f {
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
            POINTER_POWER_SAVING => Ok(self.power_saving),
            POINTER_ALS => match self.completed_als {
                Some(value) => Ok(value),
                None => Err(TransportError::Unsupported(
                    Unsupported::NoCompletedConversion(POINTER_ALS),
                )),
            },
            POINTER_WHITE => match self.completed_white {
                Some(value) => Ok(value),
                None => Err(TransportError::Unsupported(
                    Unsupported::NoCompletedConversion(POINTER_WHITE),
                )),
            },
            POINTER_ID => Ok(DEVICE_ID),
            other => Err(TransportError::Unsupported(Unsupported::RegisterPointer(
                other,
            ))),
        }
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
        if is_shutdown(self.configuration) {
            self.write_configuration_from_shutdown(word)
        } else {
            self.write_configuration_while_active(word)
        }
    }

    const fn write_configuration_from_shutdown(&mut self, word: u16) -> Result<(), TransportError> {
        if !is_shutdown(word) {
            if let Err(error) = self.ensure_conversion_start(word) {
                return Err(error);
            }
            self.remaining_ns = conversion_bound_ns(word);
        }
        self.configuration = word;
        Ok(())
    }

    const fn write_configuration_while_active(&mut self, word: u16) -> Result<(), TransportError> {
        if !is_shutdown(word) || without_shutdown(word) != without_shutdown(self.configuration) {
            return Err(TransportError::Unsupported(
                Unsupported::MidConversionReconfiguration,
            ));
        }
        self.remaining_ns = None;
        self.configuration = word;
        Ok(())
    }

    const fn write_power_saving(&mut self, word: u16) -> Result<(), TransportError> {
        if !power_saving_is_supported(word) {
            return Err(TransportError::Unsupported(Unsupported::PowerSavingWord(
                word,
            )));
        }
        self.power_saving = word;
        Ok(())
    }

    const fn ensure_conversion_start(&self, configuration: u16) -> Result<(), TransportError> {
        if conversion_bound_ns(configuration).is_some() {
            Ok(())
        } else {
            Err(TransportError::Unsupported(
                Unsupported::ReservedIntegrationTime(integration_field(configuration)),
            ))
        }
    }

    const fn latch_held_pair(&mut self) {
        self.completed_als = Some(self.held_als);
        self.completed_white = Some(self.held_white);
    }
}

impl Default for Veml7700Model {
    fn default() -> Self {
        Self::new()
    }
}
