use alloc::vec::Vec;

use ramis_core::{EventReplay, StaticEvent};

#[derive(Debug, Clone)]
pub struct SimplePath<E> {
    pub p: Vec<E>,
}

impl<E> Default for SimplePath<E> {
    fn default() -> Self {
        Self { p: Vec::new() }
    }
}

impl<E: StaticEvent + Clone> EventReplay for SimplePath<E> {
    type EventType = E;

    fn push(&mut self, event: Self::EventType) {
        self.p.push(event);
    }

    fn is_prefix_of(&self, other: &Self) -> bool {
        other.p.starts_with(&self.p)
    }

    fn extend_with_slice(&mut self, slice: &[Self::EventType]) {
        self.p.extend(slice.iter().cloned());
    }
}
