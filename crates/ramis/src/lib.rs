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
    use ramis_core::{Cancellable, SearchDomain, SelectionPolicy, StaticEvent};
    use ramis_schedule::{BFScheduler, StepScheduler};

    #[allow(type_alias_bounds)]
    type RawBFS<D: SearchDomain, C: Cancellable> = BFScheduler<
        D::State,
        D::Event,
        C,
        <D::Policy as SelectionPolicy>::OracleEvent,
        D::Policy,
        D::Algorithm,
    >;

    /// A Concurrent Breath First Search Scheduler
    pub struct BFS<D: SearchDomain, C: Cancellable> {
        inner: RawBFS<D, C>,
    }

    impl<D: SearchDomain, C> Default for BFS<D, C>
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

    impl<D: SearchDomain, C> BFS<D, C>
    where
        C: Cancellable,
    {
        /// Constructs a new `BFS` scheduler with initial state `state`.
        pub fn new(state: D::State) -> Self {
            Self {
                inner: BFScheduler::new(state),
            }
        }
    }

    impl<D: SearchDomain, C> StepScheduler<D::State, C> for BFS<D, C>
    where
        D::Event: Clone + StaticEvent,
        D::State: Clone,
        C: Cancellable + Clone,
        <D::Policy as SelectionPolicy>::OracleEvent: Clone,
    {
        type ItemMeta = <RawBFS<D, C> as StepScheduler<D::State, C>>::ItemMeta;
        type StateInterpretation = <D::Policy as SelectionPolicy>::OracleEvent;

        fn next(&self, token: C) -> Result<ramis_core::ScheduledStep<D::State, Self::ItemMeta>, C> {
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
}

pub mod core {
    //! Module containing public core types and functionality of the `Ramis` crate.
    pub mod schedule {
        //! Module containing public core scheduler types and functionality of the `Ramis` crate. Prefer using reexports in `ramis::schedule` instead.
        pub use ramis_schedule::BFScheduler;
    }
}
