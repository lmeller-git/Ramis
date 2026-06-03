use ramis_core::{HasLevelStorage, StaticEvent, generate_static_event};

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
