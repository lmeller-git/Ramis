//! Contains types useefule for interpreting generic Oracle events

use core::{cmp::Ordering, marker::PhantomData};

use ramis_core::{OracleEvent, SelectionPolicy};

/// Describes a general OracleEvent
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenericOracleEvent<T = u8> {
    /// A brnach annotated as Dead will may be pruned
    Dead = 0,
    /// A branch annotated as alive(n) may be explored further with a weigth of n
    Alive(T) = 1,
    /// a branch annotated with Accept will be explored (unless a sibling also is annotated as Accept, then it is not specified)
    Accept = 2,
}

impl<T: Ord> Ord for GenericOracleEvent<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Dead, Self::Dead) => Ordering::Equal,
            (Self::Dead, _) => Ordering::Less,
            (_, Self::Dead) => Ordering::Greater,

            (Self::Accept, Self::Accept) => Ordering::Equal,
            (Self::Accept, _) => Ordering::Greater,
            (_, Self::Accept) => Ordering::Less,

            (Self::Alive(a), Self::Alive(b)) => a.cmp(b),
        }
    }
}

impl<T: Ord> PartialOrd for GenericOracleEvent<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: 'static + Eq> OracleEvent for GenericOracleEvent<T> {
    const ACCEPTED: Option<&Self> = Some(&Self::Accept);
    const DEAD: &Self = &Self::Dead;
}

/// A policy interpreting a GenericOracleEvent according to its spec
pub struct HeuristicPolicy<T = u8>(PhantomData<T>);

impl<T: Ord> SelectionPolicy for HeuristicPolicy<T> {
    type OracleEvent = GenericOracleEvent;

    fn compare(a: &Self::OracleEvent, b: &Self::OracleEvent) -> Ordering {
        a.cmp(b)
    }

    fn may_reject(s: &Self::OracleEvent) -> bool {
        *s == GenericOracleEvent::Dead
    }

    fn may_accept(s: &Self::OracleEvent) -> bool {
        *s == GenericOracleEvent::Accept
    }
}
