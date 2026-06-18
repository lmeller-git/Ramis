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

/// A synchronized queue used for queueing tasks in a scheduler
pub trait SyncQueue<T> {
    /// tries to push a value to the queue.
    fn push(&self, item: T) -> Result<(), T>;
    /// tries to pop a value from the queue.
    fn pop(&self) -> Option<T>;
    /// pushes a value to teh queue, overwriting one if necessary
    fn force_push(&self, item: T);
    /// Clears the queue
    fn clear(&self);
    /// the length of the queue
    fn len(&self) -> usize;
    /// the capacity of the queue
    fn capacity(&self) -> usize;

    /// is the queue empty?
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// is the queue full?
    fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }
}

/// Describes a backoff strategy, which may be used instead of spinning
pub trait BackOff {
    /// The initial state of the Backoff object
    const INIT: Self;
    /// backoff one step using this strategy
    fn backoff(&mut self);
    /// reset whatever internal state may need to be reset after soem steps
    fn reset(&mut self);
}
