//! Strict ordered I²C expectation transport.

use alloc::collections::VecDeque;
use alloc::vec::Vec;

use embedded_hal_async::i2c::{
    Error as I2cError, ErrorKind, ErrorType, I2c, Operation, SevenBitAddress,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ScriptError {
    kind: ErrorKind,
}

impl ScriptError {
    pub(crate) const fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }
}

impl I2cError for ScriptError {
    fn kind(&self) -> ErrorKind {
        self.kind
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Expectation {
    Write {
        address: SevenBitAddress,
        data: Vec<u8>,
        result: Result<(), ScriptError>,
    },
    WriteRead {
        address: SevenBitAddress,
        write: Vec<u8>,
        returns: Vec<u8>,
        result: Result<(), ScriptError>,
    },
}

pub(crate) struct ScriptedI2c {
    expectations: VecDeque<Expectation>,
}

impl ScriptedI2c {
    pub(crate) fn new(expectations: impl IntoIterator<Item = Expectation>) -> Self {
        Self {
            expectations: expectations.into_iter().collect(),
        }
    }

    pub(crate) fn done(self) {
        let expectations = self.expectations;
        assert!(
            expectations.is_empty(),
            "unconsumed I2C expectations: {expectations:?}"
        );
    }

    fn next(&mut self) -> Expectation {
        self.expectations
            .pop_front()
            .expect("unexpected extra I2C transaction")
    }
}

impl ErrorType for ScriptedI2c {
    type Error = ScriptError;
}

impl I2c<SevenBitAddress> for ScriptedI2c {
    async fn read(
        &mut self,
        _address: SevenBitAddress,
        _read: &mut [u8],
    ) -> Result<(), Self::Error> {
        panic!("standalone read was not expected")
    }

    async fn write(&mut self, address: SevenBitAddress, write: &[u8]) -> Result<(), Self::Error> {
        match self.next() {
            Expectation::Write {
                address: expected,
                data,
                result,
            } => {
                assert_eq!(address, expected, "I2C write address");
                assert_eq!(write, data, "I2C write payload");
                result
            }
            other => panic!("expected {other:?}, observed write"),
        }
    }

    async fn write_read(
        &mut self,
        address: SevenBitAddress,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        match self.next() {
            Expectation::WriteRead {
                address: expected,
                write: expected_write,
                returns,
                result,
            } => {
                assert_eq!(address, expected, "I2C write_read address");
                assert_eq!(write, expected_write, "I2C pointer payload");
                assert_eq!(read.len(), returns.len(), "I2C read length");
                if result.is_ok() {
                    read.copy_from_slice(&returns);
                }
                result
            }
            other => panic!("expected {other:?}, observed write_read"),
        }
    }

    async fn transaction(
        &mut self,
        _address: SevenBitAddress,
        _operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        panic!("generic transaction was not expected")
    }
}
