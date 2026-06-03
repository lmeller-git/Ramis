#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod traits {
    pub use ramis_core::{
        Cancellable,
        DynamicEventReplay,
        EventReplay,
        SearchDomain,
        SelectionPolicy,
        StaticEvent,
    };
    pub use ramis_schedule::StepScheduler;
}

pub mod schedule {
    use ramis_core::{_is_valid, EventReplay, SearchDomain, SelectionPolicy, StaticEvent};
    use ramis_schedule::{BFScheduler, StepScheduler};

    pub struct BFS<D: SearchDomain, const N: usize> {
        #[allow(clippy::type_complexity)]
        inner: BFScheduler<
            D::Path,
            <D::Path as EventReplay>::EventType,
            D::Cancel,
            <D::Policy as SelectionPolicy>::OracleEvent,
            D::Policy,
            N, // { <<D::Path as EventReplay>::EventType as StaticEvent>::BRANCHING_FACTOR },
        >,
    }

    impl<D: SearchDomain, const N: usize> Default for BFS<D, N>
    where
        <D::Path as EventReplay>::EventType: Eq + Clone + StaticEvent,
        D::Path: Clone + Default,
    {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<D: SearchDomain, const N: usize> BFS<D, N>
    where
        <D::Path as EventReplay>::EventType: Eq + Clone + StaticEvent,
        D::Path: Clone + Default,
    {
        pub fn new() -> Self {
            #[allow(clippy::let_unit_value)]
            const {
                _ = <<D::Path as EventReplay>::EventType as _is_valid>::_is_valid;
                assert!(
                    N == <<D::Path as EventReplay>::EventType as StaticEvent>::BRANCHING_FACTOR,
                    "N should be == to StaticEvent::N"
                );
            }
            Self {
                inner: BFScheduler::new(),
            }
        }
    }

    impl<D: SearchDomain, const N: usize> StepScheduler<D::Path, D::Cancel> for BFS<D, N>
    where
        <D::Path as EventReplay>::EventType: Eq + Clone + StaticEvent,
        D::Path: Clone,
    {
        type ItemMeta = <BFScheduler<
            D::Path,
            <D::Path as EventReplay>::EventType,
            D::Cancel,
            <D::Policy as SelectionPolicy>::OracleEvent,
            D::Policy,
            N, // { <<D::Path as EventReplay>::EventType as StaticEvent>::BRANCHING_FACTOR },
        > as StepScheduler<D::Path, D::Cancel>>::ItemMeta;
        type StateInterpretation = <D::Policy as SelectionPolicy>::OracleEvent;

        fn next(
            &self,
            token: D::Cancel,
        ) -> Result<ramis_core::ScheduledStep<D::Path, Self::ItemMeta>, D::Cancel> {
            self.inner.next(token)
        }

        fn put_result(
            &self,
            path: ramis_core::ScheduledStep<D::Path, Self::ItemMeta>,
            event: Self::StateInterpretation,
        ) {
            self.inner.put_result(path, event);
        }

        fn notify_done(&self) {
            self.inner.notify_done();
        }

        fn is_cancelled(&self, item: &ramis_core::ScheduledStep<D::Path, Self::ItemMeta>) -> bool {
            self.inner.is_cancelled(item)
        }
    }
}

pub mod core {
    pub mod schedule {
        pub use ramis_schedule::BFScheduler;
    }
}
