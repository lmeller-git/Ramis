use pyo3::prelude::*;
use ramis::schedule::BFS;
use ramis_core::{ScheduledStep, SearchDomain, generate_static_event};
use ramis_schedule::StepScheduler;

use crate::{
    CancelToken,
    GenericResult,
    GenericResultInterpretor,
    PyAlgorithm,
    PyCancelToken,
    PyState,
    PyStateWrapper,
    generate_bfs_bindings,
};

generate_static_event!(
    #[pyclass(from_py_object)]
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
    pub enum BinaryEvent {
        No,
        Yes,
    }
);

pub struct BinaryTreeSearch;

impl SearchDomain for BinaryTreeSearch {
    type Algorithm = PyAlgorithm;
    type Event = BinaryEvent;
    type Policy = GenericResultInterpretor;
    type State = PyStateWrapper;
}

generate_bfs_bindings!(BinaryBFS, BinaryBFSStep, BinaryTreeSearch, PyStateWrapper);

#[pymethods]
impl BinaryBFSStep {
    pub fn state(&self) -> &Py<PyState> {
        &self.0.as_ref().map(|step| step.path()).unwrap().0
    }
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
}
