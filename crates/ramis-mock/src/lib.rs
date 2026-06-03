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

#[derive(Debug, Clone, Default)]
pub struct MockCancel {
    is_cancelled: ramis_core::sync::Arc<ramis_core::sync::atomic::AtomicBool>,
}

impl Cancellable for MockCancel {
    fn cancel(&self) {
        self.is_cancelled
            .store(true, ramis_core::sync::atomic::Ordering::Release);
    }

    fn is_cancelled(&self) -> bool {
        self.is_cancelled
            .load(ramis_core::sync::atomic::Ordering::Acquire)
    }
}
