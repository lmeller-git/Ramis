#![no_std]

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;

mod scheduled;
pub mod sync;
mod trace;

use core::fmt::Debug;

pub use scheduled::*;
pub use trace::*;

pub trait Algorithm<State, Event> {
    type Error: Debug;

    fn step(state: &mut State, event: Event) -> Result<(), Self::Error>;
}

pub trait SearchDomain {
    type State;
    type Event: StaticEvent;
    type Policy: SelectionPolicy;
    type Algorithm: Algorithm<Self::State, Self::Event>;
}
