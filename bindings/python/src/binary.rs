use pyo3::prelude::*;
use ramis::schedule::BFS;
use ramis_core::{
    HasLevelStorage,
    ScheduledStep,
    SearchDomain,
    StaticEvent,
    generate_static_event,
};
use ramis_schedule::StepScheduler;

use crate::{
    CancelToken,
    GenericResult,
    GenericResultInterpretor,
    PyAlgorithm,
    PyCancelToken,
    PyState,
    PyStateWrapper,
};

generate_static_event!(
    #[pyclass(from_py_object)]
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
    pub enum Binary {
        No,
        Yes,
    }
);

#[allow(clippy::type_complexity)]
#[pyclass(from_py_object)]
#[derive(Clone, Debug)]
pub struct BinaryBFSStep(
    Option<
        ScheduledStep<
            PyStateWrapper,
            <BinaryBFS as StepScheduler<PyStateWrapper, PyCancelToken>>::ItemMeta,
        >,
    >,
);

#[pymethods]
impl BinaryBFSStep {
    pub fn state(&self) -> &Py<PyState> {
        &self.0.as_ref().map(|step| step.path()).unwrap().0
    }
}

pub struct BinaryTreeSearch;

impl SearchDomain for BinaryTreeSearch {
    type Algorithm = PyAlgorithm;
    type Event = Binary;
    type Policy = GenericResultInterpretor;
    type State = PyStateWrapper;
}

#[pyclass]
pub struct BinaryBFS {
    raw: BFS<BinaryTreeSearch, PyCancelToken>,
}

#[pymethods]
impl BinaryBFS {
    #[new]
    #[pyo3(signature = (state))]
    pub fn new(state: Py<PyState>) -> Self {
        Self {
            raw: BFS::new(PyStateWrapper(state)),
        }
    }

    pub fn next(&self, cancel_token: Py<CancelToken>) -> PyResult<Option<BinaryBFSStep>> {
        Ok(
            <Self as StepScheduler<PyStateWrapper, PyCancelToken>>::next(
                self,
                PyCancelToken(cancel_token.into()),
            )
            .ok()
            .map(|step| BinaryBFSStep(Some(step))),
        )
    }

    pub fn put_result(&self, mut step: PyRefMut<BinaryBFSStep>, result: GenericResult) {
        <Self as StepScheduler<PyStateWrapper, PyCancelToken>>::put_result(
            self,
            step.0.take().unwrap(),
            result,
        );
    }

    pub fn notify_done(&self) {
        <Self as StepScheduler<PyStateWrapper, PyCancelToken>>::notify_done(self);
    }

    pub fn is_cancelled(&self, item: PyRef<BinaryBFSStep>) -> bool {
        item.0.as_ref().is_some_and(|item| {
            <Self as StepScheduler<PyStateWrapper, PyCancelToken>>::is_cancelled(self, item)
        })
    }
}

impl StepScheduler<PyStateWrapper, PyCancelToken> for BinaryBFS {
    type ItemMeta = <BFS<BinaryTreeSearch, PyCancelToken> as StepScheduler<
        PyStateWrapper,
        PyCancelToken,
    >>::ItemMeta;
    type StateInterpretation = GenericResult;

    fn next(
        &self,
        token: PyCancelToken,
    ) -> Result<ScheduledStep<PyStateWrapper, Self::ItemMeta>, PyCancelToken> {
        self.raw.next(token)
    }

    fn put_result(
        &self,
        state: ScheduledStep<PyStateWrapper, Self::ItemMeta>,
        event: Self::StateInterpretation,
    ) {
        self.raw.put_result(state, event);
    }

    fn notify_done(&self) {
        self.raw.notify_done();
    }

    fn is_cancelled(&self, item: &ScheduledStep<PyStateWrapper, Self::ItemMeta>) -> bool {
        self.raw.is_cancelled(item)
    }
}
