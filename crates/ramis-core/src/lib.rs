//! Core fucntionality for the `Ramis` crate

#![deny(missing_docs)]
#![deny(clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]
#![warn(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(any(feature = "std", test)), no_std)]

#[cfg(any(feature = "std", test))]
extern crate std;

extern crate alloc;

mod scheduled;
pub mod sync;
mod trace;

use core::fmt::Debug;

pub use scheduled::*;
pub use trace::*;

/// The core trait describing an algorithm.
/// This crate describes how an algorithm transforms some data `State` given an event `Event`.
pub trait Algorithm<State, Event> {
    /// The step method may return an error, signalling to teh scheduler to abort this operation.
    type Error: Debug;

    /// performs one step of teh algorithm
    fn step(state: &mut State, event: Event) -> Result<(), Self::Error>;
}

/// The core trait describing the problem to solve
pub trait SearchDomain {
    /// the data which is transformed by the algorithm.
    type State;
    /// A single branchpoint in the abstract choice-tree. Defines a step of the algorithm.
    type Event: StaticEvent;
    /// Defines how an oracle result for some state S is interpreted by the schedueler
    type Policy: SelectionPolicy;
    /// The algorithm used. Describes how the state is transformed on events.
    type Algorithm: Algorithm<Self::State, Self::Event>;
}
