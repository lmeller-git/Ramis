use std::{
    array,
    sync::{Arc, atomic::AtomicBool},
};

use im::Vector;
use ramis_core::{
    Algorithm,
    Cancellable,
    EventReplay,
    HasLevelStorage,
    OracleEvent,
    SearchDomain,
    SelectionPolicy,
    StaticEvent,
};

pub struct MockDomain;

impl SearchDomain for MockDomain {
    type Algorithm = PushAlgorithm;
    type Event = MockEvent;
    type Policy = BooleanAcceptor;
    type State = MockPath;
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct MockPath {
    pub p: Vector<MockEvent>,
}

impl EventReplay for MockPath {
    type EventType = MockEvent;

    fn push(&mut self, event: Self::EventType) {
        self.p.push_back(event);
    }

    fn is_prefix_of(&self, other: &Self) -> bool {
        if self.p.len() > other.p.len() {
            return false;
        }

        self.p.iter().zip(other.p.iter()).all(|(a, b)| a == b)
    }

    fn extend_with_slice(&mut self, slice: &[Self::EventType]) {
        self.p.extend(slice.iter().cloned());
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MockEvent(pub bool);

impl HasLevelStorage for MockEvent {
    type LevelStorage<T> = [T; 2];

    fn storage_from_fn<T, F>(f: F) -> Self::LevelStorage<T>
    where
        F: FnMut(usize) -> T,
    {
        array::from_fn(f)
    }
}

impl StaticEvent for MockEvent {
    const VARIANTS: &'static Self::LevelStorage<Self> = &[Self(true), Self(false)];
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct MockInterpretation(pub bool);

impl OracleEvent for MockInterpretation {
    const ACCEPTED: Option<&Self> = Some(&Self(true));
    const DEAD: &Self = &Self(false);
}

pub struct BooleanAcceptor;

impl SelectionPolicy for BooleanAcceptor {
    type OracleEvent = MockInterpretation;

    fn compare(a: &MockInterpretation, b: &MockInterpretation) -> std::cmp::Ordering {
        a.cmp(b)
    }
}

#[derive(Clone, Debug, Default)]
pub struct MockCancelToken {
    is_cancelled: Arc<AtomicBool>,
}

impl MockCancelToken {
    pub fn new() -> Self {
        Self {
            is_cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.is_cancelled.load(std::sync::atomic::Ordering::Acquire)
    }
}

impl Cancellable for MockCancelToken {
    fn cancel(&self) {
        self.is_cancelled
            .store(true, std::sync::atomic::Ordering::Release);
    }

    fn is_cancelled(&self) -> bool {
        self.is_cancelled.load(std::sync::atomic::Ordering::Acquire)
    }
}

pub struct PushAlgorithm;

impl Algorithm<MockPath, MockEvent> for PushAlgorithm {
    type Error = ();

    fn step(state: &mut MockPath, event: MockEvent) -> Result<(), Self::Error> {
        state.push(event);
        Ok(())
    }
}
