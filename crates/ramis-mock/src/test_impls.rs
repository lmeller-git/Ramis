use ramis_core::StaticEvent;
use ramis_schedule::StepScheduler;

use super::*;
use crate::{oracle::MockOracleEvent, path::SimplePath};

// =========================================================================
// 1. BASIC SEMANTICS TESTS
// =========================================================================

/// Invariant: A scheduler must generate an infinite stream of search paths
/// if no pruning feedback is supplied to it via `put_result`.
pub fn assert_infinite_without_feedback<S, E>()
where
    E: StaticEvent + Clone,
    S: Default + StepScheduler<SimplePath<E>, MockCancel, StateInterpretation = MockOracleEvent>,
{
    let scheduler = S::default();
    const DEPTH_THRESHOLD: usize = 500;

    // Pull an arbitrary high number of paths. Without feedback pruning,
    // the frontier must continuously expand and never dry up.
    for i in 0..DEPTH_THRESHOLD {
        let token = MockCancel::default();
        let res = scheduler.next(token);
        assert!(
            res.is_ok(),
            "Frontier dried up at step {} without receiving any dead feedback!",
            i
        );
    }
}

/// Invariant: If we actively provide pruning feedback (e.g., declaring paths dead),
/// and the maximum tree depth or variable options are naturally bounded,
/// the scheduler MUST eventually exhaust the state space and stop returning items.
pub fn assert_bounded_termination_with_feedback<S, E>(max_steps: usize)
where
    E: StaticEvent + Clone,
    S: Default + StepScheduler<SimplePath<E>, MockCancel, StateInterpretation = MockOracleEvent>,
{
    let scheduler = S::default();
    let token = MockCancel::default();
    let mut loop_count = 0;

    // Loop until next returns an error signifying the frontier is completely exhausted
    while let Ok(step) = scheduler.next(token.clone()) {
        loop_count += 1;

        // Explicitly prune EVERY single path as Dead to drain the tree
        scheduler.put_result(step, MockOracleEvent::Dead);

        // in this particular case exactly E::VARIANTS.len() paths will be created. TODO
        assert!(
            loop_count <= max_steps,
            "Scheduler failed to terminate! State space should be exhausted by now."
        );
    }
}

/// Invariant: Clean interruption behavior. Triggering cancellation on a token
/// causes internal step evaluations to flag that path as cancelled.
pub fn assert_token_cancellation_propagation<S, E>()
where
    E: StaticEvent + Clone,
    S: Default + StepScheduler<SimplePath<E>, MockCancel, StateInterpretation = MockOracleEvent>,
{
    let scheduler = S::default();
    let token = MockCancel::default();

    let step = scheduler.next(token.clone()).expect("Should fetch root");

    // Trigger cancellation on the token clone
    token.cancel();

    // The scheduler's internal representation of this item must recognize it is cancelled
    assert!(
        scheduler.is_cancelled(&step),
        "Scheduler failed to register that the token was cancelled!"
    );
}

/// Invariant: `notify_done()` signals an immediate, unrecoverable shutdown.
pub fn assert_notify_done_terminates_immediately<S, E>()
where
    E: StaticEvent + Clone,
    S: Default + StepScheduler<SimplePath<E>, MockCancel, StateInterpretation = MockOracleEvent>,
{
    let scheduler = S::default();
    let token = MockCancel::default();

    let _step = scheduler.next(token.clone()).expect("Should fetch root");

    // Abrupt termination call
    scheduler.notify_done();

    // All subsequent attempts to extract tasks must fail instantly
    assert!(
        scheduler.next(token).is_err(),
        "Scheduler allowed tasks to be drawn after notify_done() was issued!"
    );
}
