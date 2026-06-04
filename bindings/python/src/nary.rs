use pyo3::prelude::*;
use ramis::schedule::BFS;
use ramis_core::{ScheduledStep, SearchDomain};
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

macro_rules! define_nary_topologies {
    ($($val:expr),*) => {
        paste::paste! {
            $(
                #[pyclass(from_py_object)]
                #[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
                pub struct [<NAryEvent $val>] {
                    #[pyo3(get)]
                    pub index: usize,
                }

                impl ramis_core::HasLevelStorage for [<NAryEvent $val>] {
                    type LevelStorage<T> = [T; $val];

                    fn storage_from_fn<T, F>(mut f: F) -> Self::LevelStorage<T>
                    where F: FnMut(usize) -> T {
                        let mut idx = 0;
                        std::array::from_fn(|_| { let i = f(idx); idx += 1; i })
                    }
                }

                #[allow(unused_comparisons)]
                impl ramis_core::StaticEvent for [<NAryEvent $val>] {
                    const VARIANTS: &'static Self::LevelStorage<Self> = &{
                        let mut arr = [[<NAryEvent $val>] { index: 0 }; $val];
                        let mut i = 0;
                        while i < $val {
                            arr[i] = [<NAryEvent $val>] { index: i };
                            i += 1;
                        }
                        arr
                    };
                }

                pub struct [<NAryTreeSearch $val>];

                impl SearchDomain for [<NAryTreeSearch $val>] {
                    type Algorithm = PyAlgorithm;
                    type Event = [<NAryEvent $val>];
                    type Policy = GenericResultInterpretor;
                    type State = PyStateWrapper;
                }

                generate_bfs_bindings!(
                    [<NAryBFS $val>],
                    [<NAryBFSStep $val>],
                    [<NAryTreeSearch $val>],
                    PyStateWrapper
                );

                #[pymethods]
                impl [<NAryBFS $val>] {
                    #[getter]
                    pub fn branching_factor(&self) -> usize {
                        $val
                    }
                }

                #[pymethods]
                impl [<NAryBFSStep $val>] {
                    #[getter]
                    pub fn state(&self) -> &Py<PyState> {
                        &self.0.as_ref().map(|step| step.path()).unwrap().0
                    }
                }

            )*
        }
    };
}

// a selection of usual branch factors, try not top bloat bianry size. for more we shouold pivot to a dynamic approach.
// 0 just for the fun of it, does not actually make sense
define_nary_topologies!(0, 1, 2, 3, 4, 5, 6, 7, 8, 16, 32, 64, 128, 255);

#[pyfunction]
#[pyo3(signature = (branching_factor, state))]
pub fn create_nary_scheduler(
    py: Python<'_>,
    branching_factor: usize,
    state: Py<PyState>,
) -> PyResult<Py<PyAny>> {
    macro_rules! match_and_return {
        ($($val:expr),*) => {
            paste::paste! {
                match branching_factor {
                    $(
                        $val => {
                            let raw_bfs = BFS::<[<NAryTreeSearch $val>], PyCancelToken>::new(
                                PyStateWrapper(state)
                            );

                            let scheduler = [<NAryBFS $val>] { raw: raw_bfs };

                            let py_instance = Py::new(py, scheduler)?;
                            Ok(py_instance.into_any())
                        }
                    )*
                    _ => Err(pyo3::exceptions::PyValueError::new_err(
                        format!("Unsupported branching factor: {}.", branching_factor)
                    ))
                }
            }
        }
    }

    // a selection of usual branch factors, try not top bloat bianry size. for more we shouold pivot to a dynamic approach.
    // 0 just for the fun of it, does not actually make sense
    match_and_return!(0, 1, 2, 3, 4, 5, 6, 7, 8, 16, 32, 64, 128, 255)
}
