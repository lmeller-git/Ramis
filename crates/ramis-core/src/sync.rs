#![allow(dead_code, unused_imports)]

#[cfg(any(not(test), all(not(loom), not(shuttle))))]
pub use core_::*;
#[cfg(all(loom, test))]
pub use loom_::*;
#[cfg(all(shuttle, test))]
pub use shuttle_::*;

#[cfg(all(shuttle, test))]
mod shuttle_ {
    #[allow(unused_imports)]
    pub use shuttle::hint;
    pub use shuttle::{sync::atomic, thread};

    pub mod cell {
        #[derive(Debug)]
        pub struct UnsafeCell<T>(core::cell::UnsafeCell<T>);

        #[allow(dead_code)]
        impl<T> UnsafeCell<T> {
            pub fn new(data: T) -> UnsafeCell<T> {
                UnsafeCell(core::cell::UnsafeCell::new(data))
            }

            pub fn with<R>(&self, f: impl FnOnce(*const T) -> R) -> R {
                f(self.0.get())
            }

            pub fn with_mut<R>(&self, f: impl FnOnce(*mut T) -> R) -> R {
                f(self.0.get())
            }
        }

        impl<T: Default> Default for UnsafeCell<T> {
            fn default() -> Self {
                Self::new(T::default())
            }
        }
    }
}

#[cfg(all(loom, test))]
mod loom_ {
    pub use loom::{
        cell,
        hint,
        sync::{Arc, atomic},
        thread,
    };
}

#[cfg(any(not(test), all(not(loom), not(shuttle))))]
mod core_ {
    pub mod cell {
        #[derive(Debug)]
        pub struct UnsafeCell<T>(core::cell::UnsafeCell<T>);

        #[allow(dead_code)]
        impl<T> UnsafeCell<T> {
            pub fn new(data: T) -> UnsafeCell<T> {
                UnsafeCell(core::cell::UnsafeCell::new(data))
            }

            pub fn with<R>(&self, f: impl FnOnce(*const T) -> R) -> R {
                f(self.0.get())
            }

            pub fn with_mut<R>(&self, f: impl FnOnce(*mut T) -> R) -> R {
                f(self.0.get())
            }
        }

        impl<T: Default> Default for UnsafeCell<T> {
            fn default() -> Self {
                Self::new(T::default())
            }
        }
    }
    pub use alloc::sync::*;
    #[cfg(not(feature = "std"))]
    pub use core::hint;
    #[cfg(feature = "std")]
    pub use std::hint;
    #[cfg(feature = "std")]
    pub use std::thread;

    pub use portable_atomic as atomic;
}

#[cfg(all(not(loom), feature = "std"))]
pub use mutex::*;
#[cfg(all(not(loom), not(feature = "std")))]
pub use spin::{Mutex, MutexGuard};

#[cfg(all(not(loom), feature = "std"))]
mod mutex {
    pub use std::sync::MutexGuard;

    #[derive(Debug, Default)]
    pub struct Mutex<T>(std::sync::Mutex<T>);

    impl<T> Mutex<T> {
        #[allow(dead_code)]
        pub const fn new(t: T) -> Self {
            Self(std::sync::Mutex::new(t))
        }

        pub fn lock(&self) -> MutexGuard<'_, T> {
            self.0.lock().unwrap()
        }
    }
}

#[cfg(loom)]
pub(crate) use mutex::*;

#[cfg(loom)]
mod mutex {
    use core::ops::{Deref, DerefMut};

    pub use loom::sync::{Arc, MutexGuard};

    #[derive(Debug, Default)]
    pub struct Mutex<T>(loom::sync::Mutex<T>);

    impl<T> Mutex<T> {
        #[allow(dead_code)]
        pub const fn new(t: T) -> Self {
            Self(loom::sync::Mutex::new(t))
        }

        pub fn lock(&self) -> MutexGuard<'_, T> {
            self.0.lock().unwrap()
        }
    }
}
