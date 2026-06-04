#[macro_export]
macro_rules! generate_bfs_bindings {
    ($bfs_name:ident, $step_name:ident, $domain:ty, $state:ty) => {
        // ScheduledStep wrapper
        #[allow(clippy::type_complexity)]
        #[pyclass(from_py_object)]
        #[derive(Clone, Debug)]
        pub struct $step_name(
            pub(crate)  Option<
                ScheduledStep<
                    $state,
                    <$bfs_name as StepScheduler<$state, PyCancelToken>>::ItemMeta,
                >,
            >,
        );

        // Scheduler
        #[pyclass]
        pub struct $bfs_name {
            pub(crate) raw: BFS<$domain, PyCancelToken>,
        }

        // PyO3 bindings for StepScheduler methods
        #[pymethods]
        impl $bfs_name {
            #[pyo3(signature = (cancel_token))]
            pub fn next(&self, cancel_token: Py<CancelToken>) -> PyResult<Option<$step_name>> {
                Ok(<Self as StepScheduler<$state, PyCancelToken>>::next(
                    self,
                    PyCancelToken(cancel_token.into()),
                )
                .ok()
                .map(|step| $step_name(Some(step))))
            }

            #[pyo3(signature = (step, result))]
            pub fn put_result(&self, mut step: PyRefMut<$step_name>, result: GenericResult) {
                <Self as StepScheduler<$state, PyCancelToken>>::put_result(
                    self,
                    step.0.take().unwrap(),
                    result,
                );
            }

            pub fn notify_done(&self) {
                <Self as StepScheduler<$state, PyCancelToken>>::notify_done(self);
            }

            #[pyo3(signature = (item))]
            pub fn is_cancelled(&self, item: PyRef<$step_name>) -> bool {
                item.0.as_ref().is_some_and(|item| {
                    <Self as StepScheduler<$state, PyCancelToken>>::is_cancelled(self, item)
                })
            }
        }

        // StepScheduler impl
        impl StepScheduler<$state, PyCancelToken> for $bfs_name {
            type ItemMeta =
                <BFS<$domain, PyCancelToken> as StepScheduler<$state, PyCancelToken>>::ItemMeta;
            type StateInterpretation = GenericResult;

            fn next(
                &self,
                token: PyCancelToken,
            ) -> Result<ScheduledStep<$state, Self::ItemMeta>, PyCancelToken> {
                self.raw.next(token)
            }

            fn put_result(
                &self,
                state: ScheduledStep<$state, Self::ItemMeta>,
                event: Self::StateInterpretation,
            ) {
                self.raw.put_result(state, event);
            }

            fn notify_done(&self) {
                self.raw.notify_done();
            }

            fn is_cancelled(&self, item: &ScheduledStep<$state, Self::ItemMeta>) -> bool {
                self.raw.is_cancelled(item)
            }
        }
    };
}
