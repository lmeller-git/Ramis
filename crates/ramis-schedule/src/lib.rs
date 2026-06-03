//! This modules contains the schedulers used in `Ramis`.

#![deny(missing_docs)]
#![deny(clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]
#![warn(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(any(feature = "std", test)), no_std)]

#[cfg(any(feature = "std", test))]
extern crate std;

extern crate alloc;

mod adaptive;
mod breadth_first;
mod depth_first;

pub use breadth_first::*;
use ramis_core::ScheduledStep;

/// The interface of a scheduler over state T and cancellation token C
pub trait StepScheduler<T, C> {
    /// The type of Oracle events respected by thsi scheduler
    type StateInterpretation;
    /// the type of metadata used to identify ScheduledSteps
    type ItemMeta;

    /// returns the next ScheduledStep. This method only errs if the seqarch space is empty, the scheduler was killed or the algorithm failed on this state.
    fn next(&self, token: C) -> Result<ScheduledStep<T, Self::ItemMeta>, C>;
    /// returns a ScheduledStep along with the oracles interpretation of it back to teh scheduler
    fn put_result(&self, path: ScheduledStep<T, Self::ItemMeta>, event: Self::StateInterpretation);
    /// signals to the scheduler that the algorithm is done
    fn notify_done(&self);
    /// checks wether the worker associated with a ScheduledStep has been cancelled. This is different from calling is_cancelled() on the workers CancellationToken, because the scheduler may cancel the worker in opther ways also
    fn is_cancelled(&self, item: &ScheduledStep<T, Self::ItemMeta>) -> bool;
}
