#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod traits {
    pub use ramis_core::{
        Cancellable,
        DynamicEventReplay,
        EventReplay,
        SelectionPolicy,
        StaticEvent,
    };
    pub use ramis_schedule::StepScheduler;
}

pub use ramis_schedule::BFScheduler;
