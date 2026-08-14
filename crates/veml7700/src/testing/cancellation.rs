//! Pending-capable transport and delay for cancellation-boundary tests.
//!
//! A driver operation is cancelled by dropping its future. To reach a chosen
//! boundary first, something has to return [`Poll::Pending`] there: every
//! transport in these tests is otherwise ready immediately, so a future polled
//! once would run to completion and there would be no boundary to drop at.
//!
//! These wrappers make the *n*th transport operation, or the measurement delay,
//! park forever. A test polls the operation once, observes `Pending`, and drops
//! it. What that establishes is **sequencing**: exactly which transactions were
//! issued before the drop, and therefore which writes a caller may assume were
//! attempted. It establishes nothing about the device's physical state, which no
//! scripted transport can.

use core::future::{Future, pending};
use core::pin::pin;
use core::task::{Context, Poll, Waker};

use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::i2c::{ErrorType, I2c, Operation, SevenBitAddress};

/// Drive a future to its first suspension point and drop it there.
///
/// Returns `Poll::Pending` when the operation parked, which is the cancellation
/// case under test, or `Poll::Ready` when it finished without reaching one.
pub(crate) fn poll_once_then_drop<F: Future>(future: F) -> Poll<F::Output> {
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    pin!(future).as_mut().poll(&mut context)
}

/// Transport that parks forever on one chosen operation.
///
/// The parked operation never reaches the inner transport, so an expectation
/// scripted for it stays unconsumed. Counting what the inner transport did
/// consume is how a test names the boundary the future was dropped at.
pub(crate) struct PendingAt<T> {
    inner: T,
    pending_at: usize,
    attempted: usize,
}

impl<T> PendingAt<T> {
    /// Park on the operation with this zero-based index.
    ///
    /// An index at or beyond the operation count never parks, which is how the
    /// same harness runs the uncancelled case.
    pub(crate) const fn new(inner: T, pending_at: usize) -> Self {
        Self {
            inner,
            pending_at,
            attempted: 0,
        }
    }

    pub(crate) fn into_inner(self) -> T {
        self.inner
    }

    /// True when the next operation is the one that parks.
    fn parks_now(&mut self) -> bool {
        let parks = self.attempted == self.pending_at;
        if !parks {
            self.attempted += 1;
        }
        parks
    }
}

impl<T: ErrorType> ErrorType for PendingAt<T> {
    type Error = T::Error;
}

impl<T> I2c<SevenBitAddress> for PendingAt<T>
where
    T: I2c<SevenBitAddress>,
{
    async fn read(&mut self, address: SevenBitAddress, read: &mut [u8]) -> Result<(), Self::Error> {
        if self.parks_now() {
            pending::<()>().await;
        }
        self.inner.read(address, read).await
    }

    async fn write(&mut self, address: SevenBitAddress, write: &[u8]) -> Result<(), Self::Error> {
        if self.parks_now() {
            pending::<()>().await;
        }
        self.inner.write(address, write).await
    }

    async fn write_read(
        &mut self,
        address: SevenBitAddress,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        if self.parks_now() {
            pending::<()>().await;
        }
        self.inner.write_read(address, write, read).await
    }

    async fn transaction(
        &mut self,
        address: SevenBitAddress,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        if self.parks_now() {
            pending::<()>().await;
        }
        self.inner.transaction(address, operations).await
    }
}

/// Delay that either completes immediately or parks forever.
///
/// The measurement wait is the longest suspension in the driver and the one a
/// caller is most likely to cancel across, so it needs its own boundary.
pub(crate) struct CancellableDelay {
    parks: bool,
    elapsed_ns: u64,
}

impl CancellableDelay {
    pub(crate) const fn ready() -> Self {
        Self {
            parks: false,
            elapsed_ns: 0,
        }
    }

    pub(crate) const fn parking() -> Self {
        Self {
            parks: true,
            elapsed_ns: 0,
        }
    }

    pub(crate) const fn elapsed_ns(&self) -> u64 {
        self.elapsed_ns
    }
}

impl DelayNs for CancellableDelay {
    async fn delay_ns(&mut self, ns: u32) {
        if self.parks {
            pending::<()>().await;
        }
        self.elapsed_ns += u64::from(ns);
    }
}
