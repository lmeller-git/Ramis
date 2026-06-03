#![no_std]

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;

mod scheduled;
pub mod sync;
mod tape;

pub use scheduled::*;
pub use tape::*;

pub trait SearchDomain // where
//     <Self::Path as EventReplay>::EventType: StaticEvent,
{
    type Path: EventReplay;
    type Cancel: Cancellable;
    type Policy: SelectionPolicy;
}
