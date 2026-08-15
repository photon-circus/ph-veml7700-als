//! Private register map.

// Driver register-map reaction to `S-09`.

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Register {
    Configuration = 0x00,
    HighThreshold = 0x01,
    LowThreshold = 0x02,
    PowerSaving = 0x03,
    Als = 0x04,
    White = 0x05,
    ThresholdStatus = 0x06,
    DeviceId = 0x07,
}

impl Register {
    pub(crate) const fn pointer(self) -> u8 {
        self as u8
    }
}
