//! crate internal sync plumbing for testing and no_std support

#![allow(
    dead_code,
    unused_imports,
    unknown_lints,
    clippy::disallowed_modules,
    missing_docs
)]

#[cfg(all(not(loom), not(shuttle)))]
pub use core_::*;
#[cfg(loom)]
pub use loom_::*;
#[cfg(shuttle)]
pub use shuttle_::*;

#[cfg(shuttle)]
mod shuttle_ {
    #[allow(unused_imports)]
    pub use shuttle::hint;
    pub use shuttle::{
        sync::{Arc, Weak, atomic},
        thread,
    };

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

#[cfg(loom)]
mod loom_ {
    pub use loom::{
        cell,
        hint,
        sync::{Arc, Weak, atomic},
        thread,
    };
}

#[cfg(all(not(loom), not(shuttle)))]
mod core_ {
    pub mod cell {
        //! UnsafeCell
        #[derive(Debug)]
        /// wraps core::cell::UnsafeCell
        pub struct UnsafeCell<T>(core::cell::UnsafeCell<T>);

        #[allow(dead_code)]
        impl<T> UnsafeCell<T> {
            /// creates a new UnsafeCell
            pub fn new(data: T) -> UnsafeCell<T> {
                UnsafeCell(core::cell::UnsafeCell::new(data))
            }

            /// allows immutable acces to the stored value
            pub fn with<R>(&self, f: impl FnOnce(*const T) -> R) -> R {
                f(self.0.get())
            }

            /// allows mutable access to the stored value
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
#[cfg(all(not(loom), not(shuttle), not(feature = "std")))]
pub use spin::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[cfg(all(not(loom), not(shuttle), feature = "std"))]
mod mutex {
    pub use std::sync::MutexGuard;

    #[derive(Debug, Default)]
    /// wraps std::sync::Mutex
    pub struct Mutex<T>(std::sync::Mutex<T>);

    impl<T> Mutex<T> {
        #[allow(dead_code)]
        /// Constructs a new Mutex
        pub const fn new(t: T) -> Self {
            Self(std::sync::Mutex::new(t))
        }

        /// locks the Mutex. This calls unwrap() on the internal Mutex, panicking on poison.
        pub fn lock(&self) -> MutexGuard<'_, T> {
            self.0.lock().unwrap()
        }
    }

    pub use std::sync::{RwLockReadGuard, RwLockWriteGuard};

    #[derive(Debug, Default)]
    /// wraps std::sync::RwLock
    pub struct RwLock<T>(std::sync::RwLock<T>);

    impl<T> RwLock<T> {
        #[allow(dead_code)]
        /// Constructs a new RwLock
        pub const fn new(t: T) -> Self {
            Self(std::sync::RwLock::new(t))
        }

        /// read locks the RwLock. This calls unwrap() on the internal RwLock, panicking on poison.
        pub fn read(&self) -> RwLockReadGuard<'_, T> {
            self.0.read().unwrap()
        }

        /// write locks the RwLock. This calls unwrap() on the internal RwLock, panicking on poison.
        pub fn write(&self) -> RwLockWriteGuard<'_, T> {
            self.0.write().unwrap()
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
    /// wraps a loom:::sync::Mutext
    pub struct Mutex<T>(loom::sync::Mutex<T>);

    impl<T> Mutex<T> {
        #[allow(dead_code)]
        /// constructs a new Mutex
        pub const fn new(t: T) -> Self {
            Self(loom::sync::Mutex::new(t))
        }

        /// locks the mutex. unwraps poison
        pub fn lock(&self) -> MutexGuard<'_, T> {
            self.0.lock().unwrap()
        }
    }

    pub use loom::sync::{RwLockReadGuard, RwLockWriteGuard};

    #[derive(Debug, Default)]
    /// wraps loom::sync::RwLock
    pub struct RwLock<T>(loom::sync::RwLock<T>);

    impl<T> RwLock<T> {
        #[allow(dead_code)]
        /// Constructs a new RwLock
        pub const fn new(t: T) -> Self {
            Self(loom::sync::RwLock::new(t))
        }

        /// read locks the RwLock. This calls unwrap() on the internal RwLock, panicking on poison.
        pub fn read(&self) -> RwLockReadGuard<'_, T> {
            self.0.read().unwrap()
        }

        /// write locks the RwLock. This calls unwrap() on the internal RwLock, panicking on poison.
        pub fn write(&self) -> RwLockWriteGuard<'_, T> {
            self.0.write().unwrap()
        }
    }
}

#[cfg(shuttle)]
pub(crate) use mutex::*;

#[cfg(shuttle)]
mod mutex {
    use core::ops::{Deref, DerefMut};

    pub use shuttle::sync::{Arc, MutexGuard};

    #[derive(Debug, Default)]
    /// wraps a shuttle::sync::mutex
    pub struct Mutex<T>(shuttle::sync::Mutex<T>);

    impl<T> Mutex<T> {
        #[allow(dead_code)]
        /// constructs a new mutex
        pub const fn new(t: T) -> Self {
            Self(shuttle::sync::Mutex::new(t))
        }

        /// locks the mutex. unwrapsp poison
        pub fn lock(&self) -> MutexGuard<'_, T> {
            self.0.lock().unwrap()
        }
    }

    pub use shuttle::sync::{RwLockReadGuard, RwLockWriteGuard};

    #[derive(Debug, Default)]
    /// wraps shuttle::sync::RwLock
    pub struct RwLock<T>(shuttle::sync::RwLock<T>);

    impl<T> RwLock<T> {
        #[allow(dead_code)]
        /// Constructs a new RwLock
        pub const fn new(t: T) -> Self {
            Self(shuttle::sync::RwLock::new(t))
        }

        /// read locks the RwLock. This calls unwrap() on the internal RwLock, panicking on poison.
        pub fn read(&self) -> RwLockReadGuard<'_, T> {
            self.0.read().unwrap()
        }

        /// write locks the RwLock. This calls unwrap() on the internal RwLock, panicking on poison.
        pub fn write(&self) -> RwLockWriteGuard<'_, T> {
            self.0.write().unwrap()
        }
    }
}
