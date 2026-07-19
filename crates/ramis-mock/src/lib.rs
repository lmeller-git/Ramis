//! This module contains useful defuault components for algorithms run with `Ramis`.

#![deny(missing_docs)]
#![deny(clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]
#![warn(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(any(feature = "std", test)), no_std)]

#[cfg(any(feature = "std", test))]
extern crate std;

extern crate alloc;

pub mod event;
pub mod oracle;
pub mod path;

#[cfg(any(test, feature = "std"))]
pub mod test_impls;

#[cfg(feature = "std")]
use std::time::Duration;

use ramis_core::{BackOff, Cancellable};

/// A CancellationToken backed by an AtomicBool. Must be polled.
#[derive(Debug, Clone, Default)]
pub struct AtomicCancellationToken {
    is_cancelled: ramis_core::sync::Arc<ramis_core::sync::atomic::AtomicBool>,
}

impl Cancellable for AtomicCancellationToken {
    fn cancel(&self) {
        self.is_cancelled
            .store(true, ramis_core::sync::atomic::Ordering::Release);
    }

    fn is_cancelled(&self) -> bool {
        self.is_cancelled
            .load(ramis_core::sync::atomic::Ordering::Acquire)
    }
}

/// A BackOff stragety that does nothing
pub struct NoBackOff;

impl BackOff for NoBackOff {
    const INIT: Self = Self;

    fn backoff(&mut self) {}

    fn reset(&mut self) {}
}

/// A Backoff that waits for an exponentially increasing time
pub struct Exponential(u64);

impl BackOff for Exponential {
    const INIT: Self = Self(1);

    fn backoff(&mut self) {
        #[cfg(feature = "std")]
        ramis_core::sync::thread::sleep(Duration::from_micros(self.0));
        #[cfg(not(feature = "std"))]
        for _ in 0..self.0 {
            ramis_core::sync::hint::spin_loop();
        }

        self.0 = (self.0 * 2).min(1024);
    }

    fn reset(&mut self) {
        self.0 = 1
    }
}

#[cfg(feature = "std")]
/// A Backoff that yields the thread
pub struct Yield;

#[cfg(feature = "std")]
impl BackOff for Yield {
    const INIT: Self = Self;

    fn backoff(&mut self) {
        ramis_core::sync::thread::yield_now();
    }

    fn reset(&mut self) {}
}
