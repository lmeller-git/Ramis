#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod traits {
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

pub mod schedule {
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
    pub mod schedule {
        pub use ramis_schedule::BFScheduler;
    }
}
