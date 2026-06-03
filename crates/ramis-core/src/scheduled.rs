/// A scheduled step of the algorithm. This may be evaluated by the oracle
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct ScheduledStep<T, M> {
    recording: T,
    meta: M,
}

impl<T, M> ScheduledStep<T, M> {
    /// constructs a new `ScheduledStep`
    pub fn new(recording: T, meta: M) -> Self {
        Self { recording, meta }
    }

    /// returns the metadata associated with this step
    pub fn meta(&self) -> &M {
        &self.meta
    }

    /// reutrns the state of this step
    pub fn path(&self) -> &T {
        &self.recording
    }
}

/// DFescribes a Cancellation token, which may be used to cancel a worker
pub trait Cancellable {
    /// cancel this worker
    fn cancel(&self);
    /// is this worker cancelled?
    fn is_cancelled(&self) -> bool;
}
