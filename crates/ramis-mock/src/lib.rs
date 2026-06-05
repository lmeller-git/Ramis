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

use ramis_core::Cancellable;

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
