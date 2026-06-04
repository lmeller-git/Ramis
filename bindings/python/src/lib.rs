use std::{array, sync::Weak};

use pyo3::prelude::*;
use ramis_core::{
    Algorithm,
    Cancellable,
    EventReplay,
    HasLevelStorage,
    OracleEvent,
    ScheduledStep,
    SelectionPolicy,
    StaticEvent,
    sync::Arc,
};
use ramis_schedule::{BFScheduler as RawBFScheduler, StepScheduler, TreeNode};

#[derive(Clone)]
pub struct PyCancelToken(Arc<Py<PyAny>>);

impl Cancellable for PyCancelToken {
    fn cancel(&self) {
        Python::attach(|py| _ = self.0.bind(py).call_method0("set"));
    }

    fn is_cancelled(&self) -> bool {
        // Python::attach(|py| self.0.bind(py).call_method0("get"))
        false
    }
}

pub struct PushAlgorithm;

impl Algorithm<DDMinPath, DDMinEvent> for PushAlgorithm {
    type Error = ();

    fn step(state: &mut DDMinPath, event: DDMinEvent) -> Result<(), Self::Error> {
        state.push(event);
        Ok(())
    }
}

#[pyclass]
pub struct BFScheduler {
    inner: RawBFScheduler<
        DDMinPath,
        DDMinEvent,
        PyCancelToken,
        ReductionStepResult,
        DDMinEventInterpretor,
        PushAlgorithm,
    >,
}

#[pymethods]
impl BFScheduler {
    #[new]
    fn new() -> Self {
        Self {
            inner: RawBFScheduler::default(),
        }
    }

    fn next(&self, cancel_token: Py<PyAny>) -> PyResult<Option<PyScheduledStep>> {
        Ok(self
            .inner
            .next(PyCancelToken(cancel_token.into()))
            .ok()
            .map(|step| PyScheduledStep(Some(step))))
    }

    fn put_result(&self, mut path: PyRefMut<PyScheduledStep>, event: ReductionStepResult) {
        self.inner.put_result(path.0.take().unwrap(), event);
    }
}

#[allow(clippy::type_complexity)]
#[pyclass(from_py_object)]
#[derive(Clone, Debug)]
pub struct PyScheduledStep(
    Option<
        ScheduledStep<DDMinPath, Weak<TreeNode<DDMinEvent, PyCancelToken, ReductionStepResult>>>,
    >,
);

#[pymethods]
impl PyScheduledStep {
    pub fn path(&self) -> DDMinPath {
        self.0.as_ref().map(|step| step.path().clone()).unwrap()
    }
}

#[pyclass(from_py_object)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct DDMinPath(Vec<DDMinEvent>);

#[pymethods]
impl DDMinPath {
    pub fn to_list(&self) -> Vec<DDMinEventType> {
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

impl EventReplay for DDMinPath {
    type EventType = DDMinEvent;

    fn push(&mut self, event: Self::EventType) {
        self.0.push(event);
    }

    fn is_prefix_of(&self, other: &Self) -> bool {
        other.0.starts_with(&self.0)
    }

    fn extend_with_slice(&mut self, slice: &[Self::EventType]) {
        self.0.extend_from_slice(slice);
    }
}

type DDMinEventType = bool;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Hash)]
pub struct DDMinEvent(DDMinEventType);

impl OracleEvent for DDMinEvent {
    const ACCEPTED: Option<&Self> = None;
    const DEAD: &Self = &Self(false);
}

impl From<DDMinEventType> for DDMinEvent {
    fn from(value: DDMinEventType) -> Self {
        Self(value)
    }
}

impl HasLevelStorage for DDMinEvent {
    type LevelStorage<T> = [T; 2];

    fn storage_from_fn<T, F>(f: F) -> Self::LevelStorage<T>
    where
        F: FnMut(usize) -> T,
    {
        array::from_fn(f)
    }
}

impl StaticEvent for DDMinEvent {
    const VARIANTS: &'static Self::LevelStorage<Self> = &[Self(true), Self(false)];
}

#[pyclass(from_py_object)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReductionStepResult(u8);

impl OracleEvent for ReductionStepResult {
    const ACCEPTED: Option<&Self> = None;
    const DEAD: &Self = &Self(0);
}

#[pymethods]
impl ReductionStepResult {
    #[staticmethod]
    pub fn new(r: u8) -> Self {
        Self(r)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
#[pyclass(from_py_object)]
pub struct DDMinEventInterpretor;

#[pymethods]
impl DDMinEventInterpretor {
    #[new]
    pub fn new() -> Self {
        Self
    }
}

impl SelectionPolicy for DDMinEventInterpretor {
    type OracleEvent = ReductionStepResult;

    fn compare(a: &Self::OracleEvent, b: &Self::OracleEvent) -> std::cmp::Ordering {
        a.cmp(b)
    }

    fn may_reject(s: &Self::OracleEvent) -> bool {
        s.0 == 0
    }

    fn may_accept(_s: &Self::OracleEvent) -> bool {
        false
    }
}

#[pymodule]
fn lib_ramis(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<DDMinPath>()?;
    m.add_class::<BFScheduler>()?;
    Ok(())
}
