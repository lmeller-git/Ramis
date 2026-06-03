use core::marker::PhantomData;

use ramis::schedule::BFS;
use ramis_core::{SearchDomain, StaticEvent};
use ramis_mock::{
    MockCancel,
    event::{Flat, PushAlgorithm, Triplet},
    oracle::MockPolicy,
    path::SimplePath,
    test_impls::{
        assert_bounded_termination_with_feedback,
        assert_infinite_without_feedback,
        assert_notify_done_terminates_immediately,
        assert_token_cancellation_propagation,
        mpmc_concurrent,
        mpmc_concurrent_pruned,
    },
};

struct SimplDomain<E> {
    _p: PhantomData<E>,
}

impl<E: Clone + StaticEvent> SearchDomain for SimplDomain<E> {
    type Algorithm = PushAlgorithm;
    type Event = E;
    type Policy = MockPolicy;
    type State = SimplePath<E>;
}

#[test]
fn test_infinite_stream_bfs() {
    assert_infinite_without_feedback::<BFS<SimplDomain<Flat>, MockCancel>, Flat>();
}

#[test]
fn test_termination_bfs() {
    assert_bounded_termination_with_feedback::<BFS<SimplDomain<Flat>, MockCancel>, Flat>(1);
    assert_bounded_termination_with_feedback::<BFS<SimplDomain<Triplet>, MockCancel>, Triplet>(3);
}

#[test]
fn test_cancel_bfs() {
    assert_token_cancellation_propagation::<BFS<SimplDomain<Flat>, MockCancel>, Flat>();
}

#[test]
fn test_kill_bfs() {
    assert_notify_done_terminates_immediately::<BFS<SimplDomain<Flat>, MockCancel>, Flat>();
}

#[test]
fn test_get_put_sequential_bfs() {
    mpmc_concurrent::<BFS<SimplDomain<Triplet>, MockCancel>, Triplet>(1);
}

#[test]
fn test_get_put_concurrent_bfs() {
    mpmc_concurrent::<BFS<SimplDomain<Triplet>, MockCancel>, Triplet>(16);
}

#[test]
fn test_get_put_prune_sequential_bfs() {
    mpmc_concurrent_pruned::<BFS<SimplDomain<Triplet>, MockCancel>, Triplet>(1);
}

#[test]
fn test_get_put_prune_concurrent_bfs() {
    mpmc_concurrent_pruned::<BFS<SimplDomain<Triplet>, MockCancel>, Triplet>(16);
}

#[cfg(shuttle)]
mod shuttle_ {
    use super::*;

    #[test]
    fn get_put_shuttle_bfs() {
        shuttle::check_random(
            || {
                mpmc_concurrent::<BFS<SimplDomain<Triplet>, MockCancel>, Triplet>(8);
            },
            100,
        )
    }

    #[test]
    fn get_put_pruned_shuttle_bfs() {
        shuttle::check_random(
            || {
                mpmc_concurrent_pruned::<BFS<SimplDomain<Triplet>, MockCancel>, Triplet>(8);
            },
            100,
        )
    }
}
