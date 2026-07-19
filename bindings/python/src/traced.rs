use std::array;

use pyo3::prelude::*;
use ramis::schedule::BFS;
use ramis_core::{Algorithm, HasLevelStorage, ScheduledStep, SearchDomain, StaticEvent};
use ramis_schedule::StepScheduler;

use crate::{
    CancelToken,
    GenericResult,
    GenericResultInterpretor,
    PyCancelToken,
    generate_bfs_bindings,
};

#[pyclass(from_py_object)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Trace(Vec<TraceEvent>);

#[pymethods]
impl Trace {
    pub fn to_list(&self) -> Vec<TraceEventType> {
        self.0.iter().map(|item| item.0).collect()
    }

    fn __str__(&self) -> String {
        let steps: Vec<String> = self.0.iter().map(|event| format!("{}", event.0)).collect();
        steps.join(" -> ")
    }

    fn __repr__(&self) -> String {
        let bools: Vec<String> = self.0.iter().map(|event| format!("{}", event.0)).collect();

        format!("DDMinPath([{}])", bools.join(", "))
    }
}

type TraceEventType = bool;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Hash)]
pub struct TraceEvent(TraceEventType);

impl From<TraceEventType> for TraceEvent {
    fn from(value: TraceEventType) -> Self {
        Self(value)
    }
}

impl HasLevelStorage for TraceEvent {
    type LevelStorage<T> = [T; 2];

    fn storage_from_fn<T, F>(f: F) -> Self::LevelStorage<T>
    where
        F: FnMut(usize) -> T,
    {
        array::from_fn(f)
    }
}

impl StaticEvent for TraceEvent {
    const VARIANTS: &'static Self::LevelStorage<Self> = &[Self(true), Self(false)];
}

pub struct PushAlgorithm;

impl Algorithm<Trace, TraceEvent> for PushAlgorithm {
    type Error = ();

    fn step(state: &mut Trace, event: TraceEvent) -> Result<(), Self::Error> {
        state.0.push(event);
        Ok(())
    }
}

pub struct TracedAlgoDomain;

impl SearchDomain for TracedAlgoDomain {
    type Algorithm = PushAlgorithm;
    type Event = TraceEvent;
    type Policy = GenericResultInterpretor;
    type State = Trace;
}

generate_bfs_bindings!(TracedBFS, TracedStep, TracedAlgoDomain, Trace);

#[pymethods]
impl TracedStep {
    pub fn path(&self) -> Trace {
        self.0.as_ref().map(|step| step.state().clone()).unwrap()
    }
}

#[pymethods]
impl TracedBFS {
    #[new]
    pub fn new() -> Self {
        Self {
            raw: BFS::default(),
        }
    }
}
