//! Raw task queues used in schedulers

use std::collections::VecDeque;

use ramis_core::{SyncQueue, sync::Mutex};

#[derive(Debug)]
/// A VecDeque behind a Mutex
pub struct LockedVecDeque<T>(Mutex<VecDeque<T>>);

impl<T> LockedVecDeque<T> {
    /// COnstructs a new LockedVecDeque
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self(Mutex::new(VecDeque::new()))
    }

    /// Constructs a new VecDeque with cpacity cap
    #[allow(dead_code)]
    pub fn with_capacity(cap: usize) -> Self {
        Self(Mutex::new(VecDeque::with_capacity(cap)))
    }
}

impl<T> Default for LockedVecDeque<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> AsRef<Mutex<VecDeque<T>>> for LockedVecDeque<T> {
    fn as_ref(&self) -> &Mutex<VecDeque<T>> {
        &self.0
    }
}

impl<T> SyncQueue<T> for LockedVecDeque<T> {
    fn push(&self, item: T) -> Result<(), T> {
        self.as_ref().lock().push_back(item);
        Ok(())
    }

    fn pop(&self) -> Option<T> {
        self.as_ref().lock().pop_front()
    }

    fn force_push(&self, item: T) {
        _ = self.push(item);
    }

    fn clear(&self) {
        self.as_ref().lock().clear();
    }

    fn len(&self) -> usize {
        self.as_ref().lock().len()
    }

    fn capacity(&self) -> usize {
        self.as_ref().lock().capacity()
    }

    fn is_empty(&self) -> bool {
        self.as_ref().lock().is_empty()
    }

    fn is_full(&self) -> bool {
        false
    }
}

#[cfg(feature = "bounded")]
pub mod bounded {
    //! Bounded queues for tasks
    use nblf_queue::{MPMCQueue, PooledQueue, core::slots::Tagged64};

    use super::*;
    /// A bounded lock free queue
    pub struct NBLFQ<T>(PooledQueue<T, Tagged64>);

    impl<T> NBLFQ<T> {
        /// Constructs a new NBLFQ, bounded by size
        #[allow(dead_code)]
        pub fn new(size: usize) -> Self {
            Self(PooledQueue::with_slot::<Tagged64>(size))
        }
    }

    impl<T> AsRef<PooledQueue<T, Tagged64>> for NBLFQ<T> {
        fn as_ref(&self) -> &PooledQueue<T, Tagged64> {
            &self.0
        }
    }

    impl<T> SyncQueue<T> for NBLFQ<T> {
        fn push(&self, item: T) -> Result<(), T> {
            self.as_ref().push(item)
        }

        fn pop(&self) -> Option<T> {
            self.as_ref().pop()
        }

        fn force_push(&self, item: T) {
            _ = self.as_ref().force_push(item);
        }

        fn clear(&self) {
            while self.pop().is_some() {}
        }

        fn len(&self) -> usize {
            self.as_ref().len()
        }

        fn capacity(&self) -> usize {
            self.as_ref().capacity()
        }

        fn is_empty(&self) -> bool {
            self.as_ref().is_empty()
        }

        fn is_full(&self) -> bool {
            self.as_ref().is_full()
        }
    }
}
