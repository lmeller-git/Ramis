use std::array;

use pyo3::prelude::*;
use ramis::schedule::BFS;
use ramis_core::{Algorithm, HasLevelStorage, ScheduledStep, SearchDomain, StaticEvent};
use ramis_schedule::StepScheduler;

use crate::{CancelToken, GenericResult, GenericResultInterpretor, PyCancelToken};

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

#[allow(clippy::type_complexity)]
#[pyclass(from_py_object)]
#[derive(Clone, Debug)]
pub struct TracedStep(
    Option<ScheduledStep<Trace, <TracedBFS as StepScheduler<Trace, PyCancelToken>>::ItemMeta>>,
);

#[pymethods]
impl TracedStep {
    pub fn path(&self) -> Trace {
        self.0.as_ref().map(|step| step.path().clone()).unwrap()
    }
}

pub struct TracedAlgoDomain;

impl SearchDomain for TracedAlgoDomain {
    type Algorithm = PushAlgorithm;
    type Event = TraceEvent;
    type Policy = GenericResultInterpretor;
    type State = Trace;
}

#[pyclass]
pub struct TracedBFS {
    raw: BFS<TracedAlgoDomain, PyCancelToken>,
}

#[pymethods]
impl TracedBFS {
    #[new]
    fn new() -> Self {
        Self {
            raw: BFS::default(),
        }
    }

    pub fn next(&self, cancel_token: Py<CancelToken>) -> PyResult<Option<TracedStep>> {
        Ok(<Self as StepScheduler<Trace, PyCancelToken>>::next(
            self,
            PyCancelToken(cancel_token.into()),
        )
        .ok()
        .map(|step| TracedStep(Some(step))))
    }

    pub fn put_result(&self, mut step: PyRefMut<TracedStep>, result: GenericResult) {
        <Self as StepScheduler<Trace, PyCancelToken>>::put_result(
            self,
            step.0.take().unwrap(),
            result,
        );
    }

    pub fn notify_done(&self) {
        <Self as StepScheduler<Trace, PyCancelToken>>::notify_done(self);
    }

    pub fn is_cancelled(&self, item: PyRef<TracedStep>) -> bool {
        item.0.as_ref().is_some_and(|item| {
            <Self as StepScheduler<Trace, PyCancelToken>>::is_cancelled(self, item)
        })
    }
}

impl StepScheduler<Trace, PyCancelToken> for TracedBFS {
    type ItemMeta =
        <BFS<TracedAlgoDomain, PyCancelToken> as StepScheduler<Trace, PyCancelToken>>::ItemMeta;
    type StateInterpretation = GenericResult;

    fn next(
        &self,
        token: PyCancelToken,
    ) -> Result<ScheduledStep<Trace, Self::ItemMeta>, PyCancelToken> {
        self.raw.next(token)
    }

    fn put_result(
        &self,
        state: ScheduledStep<Trace, Self::ItemMeta>,
        event: Self::StateInterpretation,
    ) {
        self.raw.put_result(state, event);
    }

    fn notify_done(&self) {
        self.raw.notify_done();
    }

    fn is_cancelled(&self, item: &ScheduledStep<Trace, Self::ItemMeta>) -> bool {
        self.raw.is_cancelled(item)
    }
}
