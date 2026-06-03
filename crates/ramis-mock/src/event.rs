use ramis_core::{Algorithm, EventReplay, HasLevelStorage, StaticEvent, generate_static_event};

use crate::path::SimplePath;

pub struct PushAlgorithm;

impl<E: StaticEvent + Clone> Algorithm<SimplePath<E>, E> for PushAlgorithm {
    type Error = ();

    fn step(state: &mut SimplePath<E>, event: E) -> Result<(), Self::Error> {
        state.push(event);
        Ok(())
    }
}

generate_static_event! {
    pub enum Flat {
        V,
    }
}

generate_static_event! {
    pub enum Boolean {
        V1,
        V2,
    }
}

generate_static_event! {
    pub enum Triplet {
        V1,
        V2,
        V3
    }
}
