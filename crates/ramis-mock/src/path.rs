//! Contains types useful for tracking traces through the searched tree

use alloc::vec::Vec;
use core::marker::PhantomData;

use ramis_core::{Algorithm, EventReplay, SearchDomain, StaticEvent};

use crate::oracle::HeuristicPolicy;

/// Builds a VecTrace through the searched tree
pub struct TraceRecorder;

impl<E: StaticEvent + Clone> Algorithm<VecTrace<E>, E> for TraceRecorder {
    type Error = ();

    fn step(state: &mut VecTrace<E>, event: E) -> Result<(), Self::Error> {
        state.push(event);
        Ok(())
    }
}

/// Recoeds a trace of Events throuh the search tree
#[derive(Debug, Clone)]
pub struct VecTrace<E> {
    /// the trace of events
    pub trace: Vec<E>,
}

impl<E> VecTrace<E> {
    /// Constructs a new VecTrace
    pub fn new() -> Self {
        Self::default()
    }
}

impl<E> Default for VecTrace<E> {
    fn default() -> Self {
        Self { trace: Vec::new() }
    }
}

impl<E: StaticEvent + Clone> EventReplay for VecTrace<E> {
    type EventType = E;

    fn push(&mut self, event: Self::EventType) {
        self.trace.push(event);
    }

    fn is_prefix_of(&self, other: &Self) -> bool {
        other.trace.starts_with(&self.trace)
    }

    fn extend_with_slice(&mut self, slice: &[Self::EventType]) {
        self.trace.extend(slice.iter().cloned());
    }
}

/// Describes an algorithm that simply records a Trace of Events `E` through the search space, guided by OracleEvents `T`.
pub struct TracedSearcher<E, T>(PhantomData<(E, T)>);

impl<E: StaticEvent + Clone, T: Ord> SearchDomain for TracedSearcher<E, T> {
    type Algorithm = TraceRecorder;
    type Event = E;
    type Policy = HeuristicPolicy<T>;
    type State = VecTrace<E>;
}
