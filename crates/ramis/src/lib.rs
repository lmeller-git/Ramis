#![doc = include_str!("../../../README.md")]
#![deny(missing_docs)]
#![deny(clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]
#![warn(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(any(feature = "std", test)), no_std)]

#[cfg(feature = "std")]
extern crate std;

pub mod traits {
    //! Module containing core traits used to describe an Algorithm to a scheduler.
    pub use ramis_core::{
        Algorithm,
        Cancellable,
        HasLevelStorage,
        OracleEvent,
        SearchDomain,
        SelectionPolicy,
        StaticEvent,
    };
    pub use ramis_schedule::StepScheduler;
}

pub mod components {
    //! Useful prebuilt components for expressing algorihtm run with `Ramis`
    pub use ramis_mock::{
        AtomicCancellationToken,
        event::*,
        oracle::{GenericOracleEvent, HeuristicPolicy},
        path::{TraceRecorder, TracedSearcher, VecTrace},
    };
}

pub mod schedule {
    //! Module containing schedulers. All schedulers in this module are more convenient newtypes or reexports from `ramis::core::schedule`.
    pub use ramis_core::BranchDirective;
    use ramis_core::{Algorithm, BackOff, Cancellable, SearchDomain, SelectionPolicy, StaticEvent};
    use ramis_mock::NoBackOff;
    pub use ramis_schedule::StepError;
    use ramis_schedule::{BFScheduler, StepScheduler, schedule::BFS as RawBFS};

    /// A Concurrent Breadth First Search Scheduler with unbounded capacity
    pub struct BFS<D: SearchDomain, C: Cancellable, B: BackOff = NoBackOff> {
        inner: RawBFS<D, C, B>,
    }

    impl<D: SearchDomain, C> Default for BFS<D, C, NoBackOff>
    where
        D::State: Default,
        C: Cancellable,
    {
        fn default() -> Self {
            Self {
                inner: BFScheduler::default(),
            }
        }
    }

    impl<D: SearchDomain, C: Cancellable> BFS<D, C, NoBackOff> {
        /// Constructs a new `BFS` scheduler with initial state `state`.
        pub fn new(state: D::State) -> Self {
            Self {
                inner: BFScheduler::new(state),
            }
        }

        /// Consturcts a new `BFS` scheduler with initial state `state` and backoff B
        pub fn with_backoff<B: BackOff>(state: D::State) -> BFS<D, C, B> {
            BFS {
                inner: BFScheduler::new(state),
            }
        }
    }

    impl<D: SearchDomain, C, B>
        StepScheduler<D::State, C, <D::Algorithm as Algorithm<D::State, D::Event>>::Error>
        for BFS<D, C, B>
    where
        D::Event: Clone + StaticEvent,
        D::State: Clone,
        C: Cancellable + Clone,
        <D::Policy as SelectionPolicy>::OracleEvent: Clone,
        B: BackOff,
    {
        type ItemMeta = <RawBFS<D, C, B> as StepScheduler<
            D::State,
            C,
            <D::Algorithm as Algorithm<D::State, D::Event>>::Error,
        >>::ItemMeta;
        type StateInterpretation = <D::Policy as SelectionPolicy>::OracleEvent;

        fn next(
            &self,
            token: C,
        ) -> Result<
            ramis_core::ScheduledStep<D::State, Self::ItemMeta>,
            StepError<C, <D::Algorithm as Algorithm<D::State, D::Event>>::Error>,
        > {
            self.inner.next(token)
        }

        fn put_result(
            &self,
            path: ramis_core::ScheduledStep<D::State, Self::ItemMeta>,
            event: Self::StateInterpretation,
        ) {
            self.inner.put_result(path, event);
        }

        fn notify_done(&self) {
            self.inner.notify_done();
        }

        fn is_cancelled(&self, item: &ramis_core::ScheduledStep<D::State, Self::ItemMeta>) -> bool {
            self.inner.is_cancelled(item)
        }
    }

    #[cfg(feature = "bounded")]
    pub use bounded::*;

    #[cfg(feature = "bounded")]
    mod bounded {
        use ramis_core::Algorithm;
        use ramis_mock::Exponential;
        use ramis_schedule::{backend::bounded::NBLFQ, schedule::BoundedBFS as RawBoundedBFS};

        use super::*;

        /// A Concurrent Breadth First Search Scheduler with bounded capacity
        pub struct BoundedBFS<D: SearchDomain, C: Cancellable, B: BackOff = Exponential> {
            inner: RawBoundedBFS<D, C, B>,
        }

        impl<D: SearchDomain, C: Cancellable> BoundedBFS<D, C, Exponential> {
            /// Constructs a new `BoundedBFS` scheduler with initial state `state`, bounded by `capacity`
            pub fn new(state: D::State, capacity: usize) -> Self {
                Self {
                    inner: BFScheduler::new_with_queue(state, NBLFQ::new(capacity)),
                }
            }

            /// Consturcts a new `BoundedBFS` scheduler with initial state `state` and backoff B, bounded by `capacity`
            pub fn with_backoff<B: BackOff>(
                state: D::State,
                capacity: usize,
            ) -> BoundedBFS<D, C, B> {
                BoundedBFS {
                    inner: BFScheduler::new_with_queue(state, NBLFQ::new(capacity)),
                }
            }
        }

        impl<D: SearchDomain, C, B>
            StepScheduler<D::State, C, <D::Algorithm as Algorithm<D::State, D::Event>>::Error>
            for BoundedBFS<D, C, B>
        where
            D::Event: Clone + StaticEvent,
            D::State: Clone,
            C: Cancellable + Clone,
            <D::Policy as SelectionPolicy>::OracleEvent: Clone,
            B: BackOff,
        {
            type ItemMeta = <RawBFS<D, C, B> as StepScheduler<
                D::State,
                C,
                <D::Algorithm as Algorithm<D::State, D::Event>>::Error,
            >>::ItemMeta;
            type StateInterpretation = <D::Policy as SelectionPolicy>::OracleEvent;

            fn next(
                &self,
                token: C,
            ) -> Result<
                ramis_core::ScheduledStep<D::State, Self::ItemMeta>,
                StepError<C, <D::Algorithm as Algorithm<D::State, D::Event>>::Error>,
            > {
                self.inner.next(token)
            }

            fn put_result(
                &self,
                path: ramis_core::ScheduledStep<D::State, Self::ItemMeta>,
                event: Self::StateInterpretation,
            ) {
                self.inner.put_result(path, event);
            }

            fn notify_done(&self) {
                self.inner.notify_done();
            }

            fn is_cancelled(
                &self,
                item: &ramis_core::ScheduledStep<D::State, Self::ItemMeta>,
            ) -> bool {
                self.inner.is_cancelled(item)
            }
        }
    }
}
