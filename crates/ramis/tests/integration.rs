use core::marker::PhantomData;

use ramis::schedule::BFS;
use ramis_core::{SearchDomain, StaticEvent};
use ramis_mock::{
    MockCancel,
    event::{Flat, Triplet},
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
    type Cancel = MockCancel;
    type Path = SimplePath<E>;
    type Policy = MockPolicy;
}

#[test]
fn test_infinite_stream_bfs() {
    assert_infinite_without_feedback::<BFS<SimplDomain<Flat>>, Flat>();
}

#[test]
fn test_termination_bfs() {
    assert_bounded_termination_with_feedback::<BFS<SimplDomain<Flat>>, Flat>(1);
    assert_bounded_termination_with_feedback::<BFS<SimplDomain<Triplet>>, Triplet>(3);
}

#[test]
fn test_cancel_bfs() {
    assert_token_cancellation_propagation::<BFS<SimplDomain<Flat>>, Flat>();
}

#[test]
fn test_kill_bfs() {
    assert_notify_done_terminates_immediately::<BFS<SimplDomain<Flat>>, Flat>();
}

#[test]
fn test_get_put_sequential_bfs() {
    mpmc_concurrent::<BFS<SimplDomain<Triplet>>, Triplet>(1);
}

#[test]
fn test_get_put_concurrent_bfs() {
    mpmc_concurrent::<BFS<SimplDomain<Triplet>>, Triplet>(16);
}

#[test]
fn test_get_put_prune_sequential_bfs() {
    mpmc_concurrent_pruned::<BFS<SimplDomain<Triplet>>, Triplet>(1);
}

#[test]
fn test_get_put_prune_concurrent_bfs() {
    mpmc_concurrent_pruned::<BFS<SimplDomain<Triplet>>, Triplet>(16);
}

#[cfg(shuttle)]
mod shuttle_ {
    use super::*;

    #[test]
    fn get_put_shuttle_bfs() {
        shuttle::check_random(
            || {
                mpmc_concurrent::<BFS<SimplDomain<Triplet>>, Triplet>(8);
            },
            100,
        )
    }

    #[test]
    fn get_put_pruned_shuttle_bfs() {
        shuttle::check_random(
            || {
                mpmc_concurrent_pruned::<BFS<SimplDomain<Triplet>>, Triplet>(8);
            },
            100,
        )
    }
}
