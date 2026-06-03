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
