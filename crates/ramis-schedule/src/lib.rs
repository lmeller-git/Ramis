//! This modules contains the schedulers used in `Ramis`.

#![deny(missing_docs)]
#![deny(clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]
#![warn(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(any(feature = "std", test)), no_std)]

#[cfg(any(feature = "std", test))]
extern crate std;

extern crate alloc;

mod adaptive;
pub mod backend;
mod breadth_first;
mod depth_first;

pub use breadth_first::*;
use ramis_core::ScheduledStep;

#[derive(Debug)]
/// An error of the scheduler. This error is not necessarily terminal.
pub enum StepError<C, E> {
    /// The algorithm has terminated
    Terminated(C),
    /// We are busy and cannot scheduler more work currently
    Busy(C),
    /// The algorithm ran into an error
    Algorithmic((C, E)),
    /// Somethign unexpected happened
    TODO(C),
}

/// The interface of a scheduler over state T and cancellation token C
pub trait StepScheduler<T, C, E> {
    /// The type of Oracle events respected by thsi scheduler
    type StateInterpretation;
    /// the type of metadata used to identify ScheduledSteps
    type ItemMeta;

    /// returns the next ScheduledStep. This method only errs if the seqarch space is empty, the scheduler was killed or the algorithm failed on this state.
    fn next(&self, token: C) -> Result<ScheduledStep<T, Self::ItemMeta>, StepError<C, E>>;
    /// returns a ScheduledStep along with the oracles interpretation of it back to teh scheduler
    fn put_result(&self, path: ScheduledStep<T, Self::ItemMeta>, event: Self::StateInterpretation);
    /// signals to the scheduler that the algorithm is done
    fn notify_done(&self);
    /// checks wether the worker associated with a ScheduledStep has been cancelled. This is different from calling is_cancelled() on the workers CancellationToken, because the scheduler may cancel the worker in opther ways also
    fn is_cancelled(&self, item: &ScheduledStep<T, Self::ItemMeta>) -> bool;
}

pub mod schedule {
    //! Contains types exported from ramis-schedule
    #![allow(type_alias_bounds)]
    use ramis_core::{Cancellable, SearchDomain, SelectionPolicy, SyncQueue};

    #[cfg(feature = "bounded")]
    use crate::backend::bounded::NBLFQ;
    use crate::{BFScheduler, backend::LockedVecDeque, breadth_first};

    type RawBFS<
        D: SearchDomain,
        C: Cancellable,
        Q: SyncQueue<breadth_first::ScheduledTask<D::Event, C, D::State>>,
        B,
    > = BFScheduler<
        D::State,
        D::Event,
        C,
        <D::Policy as SelectionPolicy>::OracleEvent,
        D::Policy,
        D::Algorithm,
        Q,
        B,
    >;

    /// A BFS scheduler with unbounded capacity
    pub type BFS<D: SearchDomain, C: Cancellable + Send + Sync, B> = RawBFS<
        D,
        C,
        LockedVecDeque<
            breadth_first::ScheduledTask<D::Event, C, <D::Policy as SelectionPolicy>::OracleEvent>,
        >,
        B,
    >;

    /// A BFS scheduler with bounded capacity
    #[cfg(feature = "bounded")]
    pub type BoundedBFS<D: SearchDomain, C: Cancellable, B> = RawBFS<
        D,
        C,
        NBLFQ<
            breadth_first::ScheduledTask<D::Event, C, <D::Policy as SelectionPolicy>::OracleEvent>,
        >,
        B,
    >;
}
