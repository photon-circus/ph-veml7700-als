//! Transport results that distinguish device refusals from model limitations.

use core::fmt;

/// Outcome of a model input that is not a successful device response.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportError {
    /// Documented device refusal at the I²C address.
    NoAcknowledge {
        /// Which part of the transfer the device refused.
        source: NoAcknowledgeSource,
    },
    /// Operation outside this model's declared slice.
    ///
    /// This is not a device NACK, timeout, or fault bit.
    Unsupported(Unsupported),
}

/// Source-backed I²C NACK classification used by this slice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoAcknowledgeSource {
    /// No device acknowledges the 7-bit address.
    Address,
}

/// Reason an input is outside the declared probe/`measure_once` slice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unsupported {
    /// Command pointer is not part of this slice, or is not usable in this
    /// direction.
    RegisterPointer(u8),
    /// Write or write-read payload length is not a supported register transfer.
    TransactionShape,
    /// Conversion while power-saving cadence is enabled.
    PowerSavingEnabledConversion,
    /// Configuration fields other than the shutdown bit changed while active.
    MidConversionReconfiguration,
    /// Integration-time field is a reserved encoding, so no bound exists.
    ReservedIntegrationTime(u16),
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoAcknowledge {
                source: NoAcknowledgeSource::Address,
            } => f.write_str("I2C address NACK"),
            Self::Unsupported(reason) => write!(f, "unsupported model input: {reason}"),
        }
    }
}

impl fmt::Display for Unsupported {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RegisterPointer(pointer) => {
                write!(f, "register pointer 0x{pointer:02X} is outside this slice")
            }
            Self::TransactionShape => f.write_str("transaction shape is outside this slice"),
            Self::PowerSavingEnabledConversion => {
                f.write_str("power-saving-enabled conversion is outside this slice")
            }
            Self::MidConversionReconfiguration => {
                f.write_str("mid-conversion reconfiguration is outside this slice")
            }
            Self::ReservedIntegrationTime(observed) => {
                write!(
                    f,
                    "reserved integration-time encoding 0b{observed:04b} has no conversion bound"
                )
            }
        }
    }
}
