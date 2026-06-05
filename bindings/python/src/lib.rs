use pyo3::{
    prelude::*,
    types::{PyDict, PyTuple},
};
use ramis_core::{Algorithm, Cancellable, OracleEvent, SelectionPolicy, sync::Arc};

use crate::{
    binary::{BinaryBFS, BinaryBFSStep, BinaryEvent},
    nary::create_nary_scheduler,
    traced::{Trace, TracedBFS, TracedStep},
};

mod binary;
mod macros;
mod nary;
mod traced;

#[pyclass(subclass, from_py_object)]
#[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct CancelToken;

#[pymethods]
impl CancelToken {
    #[new]
    #[pyo3(signature = (*_args, **_kwargs))]
    fn new(_args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>) -> Self {
        CancelToken
    }
}

#[derive(Clone)]
pub struct PyCancelToken(Arc<Py<CancelToken>>);

impl Cancellable for PyCancelToken {
    fn cancel(&self) {
        Python::attach(|py| _ = self.0.call_method0(py, "cancel").unwrap());
    }

    fn is_cancelled(&self) -> bool {
        Python::attach(|py| {
            self.0
                .call_method0(py, "is_cancelled")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }
}

#[pyclass(subclass, from_py_object)]
#[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct PyState;

#[pymethods]
impl PyState {
    #[new]
    #[pyo3(signature = (*_args, **_kwargs))]
    fn new(_args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>) -> Self {
        PyState
    }
}

#[derive(Debug)]
pub struct PyStateWrapper(Py<PyState>);

impl Clone for PyStateWrapper {
    fn clone(&self) -> Self {
        Python::attach(|py| Self(self.0.clone_ref(py)))
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct PyAlgorithm;

impl<Event> Algorithm<PyStateWrapper, Event> for PyAlgorithm
where
    Event: for<'py> IntoPyObject<'py>,
{
    type Error = ();

    fn step(state: &mut PyStateWrapper, event: Event) -> Result<(), Self::Error> {
        Python::attach(|py| {
            state.0 = state
                .0
                .call_method1(py, "step", (event,))
                .expect("step_fn must return new state")
                .extract(py)
                .unwrap();
        });
        Ok(())
    }
}

#[pyclass(from_py_object)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GenericResult(u64);

impl OracleEvent for GenericResult {
    const ACCEPTED: Option<&Self> = None;
    const DEAD: &Self = &Self(0);
}

#[pymethods]
impl GenericResult {
    #[new]
    #[pyo3(signature = (r))]
    pub fn new(r: u64) -> Self {
        Self(r)
    }

    pub fn is_dead(&self) -> bool {
        self == Self::DEAD
    }

    pub fn is_accepted(&self) -> bool {
        Self::ACCEPTED.is_some_and(|acc| acc == self)
    }

    pub fn raw_score(&self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
#[pyclass(from_py_object)]
pub struct GenericResultInterpretor;

#[pymethods]
impl GenericResultInterpretor {
    #[new]
    pub fn new() -> Self {
        Self
    }
}

impl SelectionPolicy for GenericResultInterpretor {
    type OracleEvent = GenericResult;

    fn compare(a: &Self::OracleEvent, b: &Self::OracleEvent) -> std::cmp::Ordering {
        a.cmp(b)
    }

    fn may_reject(s: &Self::OracleEvent) -> bool {
        s == Self::OracleEvent::DEAD
    }

    fn may_accept(s: &Self::OracleEvent) -> bool {
        Self::OracleEvent::ACCEPTED.is_some_and(|accepted| s == accepted)
    }
}

#[pymodule]
fn lib_ramis(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyState>()?;
    m.add_class::<CancelToken>()?;
    m.add_class::<GenericResult>()?;
    m.add_class::<GenericResultInterpretor>()?;

    let traced = PyModule::new(py, "traced")?;
    traced.add_class::<Trace>()?;
    traced.add_class::<TracedBFS>()?;
    traced.add_class::<TracedStep>()?;

    m.add_submodule(&traced)?;

    let binary = PyModule::new(py, "binary")?;

    binary.add_class::<BinaryBFS>()?;
    binary.add_class::<BinaryEvent>()?;
    binary.add_class::<BinaryBFSStep>()?;

    m.add_submodule(&binary)?;

    let nary = &PyModule::new(py, "nary")?;

    nary.add_function(wrap_pyfunction!(create_nary_scheduler, nary)?)?;

    m.add_submodule(nary)?;

    Ok(())
}
