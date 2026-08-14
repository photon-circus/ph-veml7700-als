//! Transport results that distinguish device refusals from model limitations.

use core::fmt;

/// Outcome of a model input that is not a successful device response.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
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
#[non_exhaustive]
pub enum NoAcknowledgeSource {
    /// No device acknowledges the 7-bit address.
    Address,
}

/// Reason an input is outside the declared behavioral slice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Unsupported {
    /// Command pointer is not part of this slice, or is not usable in this
    /// direction.
    RegisterPointer(u8),
    /// Write or write-read payload length is not a supported register transfer.
    TransactionShape,
    /// Address value cannot be represented by the declared 7-bit I²C boundary.
    AddressOutOfRange(u8),
    /// An output register was read before this model had completed a conversion.
    NoCompletedConversion(u8),
    /// A threshold register was read before this model had observed it written.
    NoProgrammedThreshold(u8),
    /// Threshold status was read before a monitored ALS refresh established it.
    NoQualifiedStatus(u8),
    /// Configuration word contains behavior outside the declared slice.
    ConfigurationWord(u16),
    /// Power-saving word contains behavior outside the declared slice.
    PowerSavingWord(u16),
    /// An active configuration write was repeated, or configuration fields were
    /// changed while active.
    MidConversionReconfiguration,
    /// Integration-time field is a reserved encoding, so no bound exists.
    ReservedIntegrationTime(u16),
    /// Power-saving cadence is not documented for the selected integration.
    UndocumentedPowerSavingCadence {
        /// Complete configuration word selecting the integration time.
        configuration: u16,
        /// Complete power-saving word selecting and enabling the mode.
        power_saving: u16,
    },
    /// Threshold programming while monitoring is enabled is outside this slice.
    ThresholdWriteWhileMonitoring(u8),
    /// Enabling the monitor before both thresholds are programmed is unsupported.
    IncompleteThresholdDomain,
    /// Changing power-saving cadence while conversions are active is unsupported.
    ActivePowerSavingReconfiguration,
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
            Self::AddressOutOfRange(address) => write!(
                f,
                "address 0x{address:02X} is outside the 7-bit model input domain"
            ),
            Self::NoCompletedConversion(pointer) => write!(
                f,
                "output register 0x{pointer:02X} has no completed conversion in this model"
            ),
            Self::NoProgrammedThreshold(pointer) => write!(
                f,
                "threshold register 0x{pointer:02X} has no programmed value in this model"
            ),
            Self::NoQualifiedStatus(pointer) => write!(
                f,
                "threshold-status register 0x{pointer:02X} has no qualified status in this model"
            ),
            Self::ConfigurationWord(observed) => write!(
                f,
                "configuration word 0x{observed:04X} is outside this slice"
            ),
            Self::PowerSavingWord(observed) => write!(
                f,
                "power-saving word 0x{observed:04X} is outside this slice"
            ),
            Self::MidConversionReconfiguration => {
                f.write_str("configuration write while active is outside this slice")
            }
            Self::ReservedIntegrationTime(observed) => {
                write!(
                    f,
                    "reserved integration-time encoding 0b{observed:04b} has no conversion bound"
                )
            }
            Self::UndocumentedPowerSavingCadence {
                configuration,
                power_saving,
            } => write!(
                f,
                "power-saving word 0x{power_saving:04X} has no documented cadence for configuration 0x{configuration:04X}"
            ),
            Self::ThresholdWriteWhileMonitoring(pointer) => write!(
                f,
                "threshold register 0x{pointer:02X} cannot be programmed in this slice while monitoring is enabled"
            ),
            Self::IncompleteThresholdDomain => {
                f.write_str("threshold monitoring requires both thresholds to be programmed")
            }
            Self::ActivePowerSavingReconfiguration => {
                f.write_str("power-saving reconfiguration while active is outside this slice")
            }
        }
    }
}

// `Display` already existed on both types; these add the standard trait so a
// model failure can join an error chain like any other. Neither carries an
// underlying cause: an unsupported interaction is a limit this model declares,
// not a failure it forwarded from somewhere else, and inventing a source would
// misrepresent where the boundary is.

impl core::error::Error for TransportError {}

impl core::error::Error for Unsupported {}
