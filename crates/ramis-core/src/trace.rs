use core::cmp::Ordering;

/// trace of efents throughg a tree
pub trait EventReplay: Sized {
    /// the node type
    type EventType;
    /// push a node to this trace
    fn push(&mut self, event: Self::EventType);
    /// check wether self is a prefix of other, i.e. check wether other is a subtree of self
    fn is_prefix_of(&self, other: &Self) -> bool;
    /// extends self with all nodes in slice
    fn extend_with_slice(&mut self, slice: &[Self::EventType]);
}

/// non static version of Event
pub trait DynamicEventReplay: EventReplay {
    /// an iterator over the variants of the event in this type
    fn children(&self) -> impl Iterator<Item = Self>;
}

/// Defines the type of storage used for nodes in a tree, as well as its branching fatcor
pub trait HasLevelStorage {
    /// the storage type. This may be used as storage for arbitrary types in schedulers
    type LevelStorage<T>: AsRef<[T]> + AsMut<[T]>;
    /// constructs a LevelStorage from the providfed fn
    fn storage_from_fn<T, F>(f: F) -> Self::LevelStorage<T>
    where
        F: FnMut(usize) -> T;
}

/// An event in the algorithm. In general this corresponds to one node in a tree, where each variant corresponds to one branch
pub trait StaticEvent: HasLevelStorage + Sized + 'static + Eq {
    /// The variants of this event that guide the algorithm
    const VARIANTS: &Self::LevelStorage<Self>;
}

impl<T> DynamicEventReplay for T
where
    T: EventReplay + Clone,
    T::EventType: StaticEvent + Clone,
{
    fn children(&self) -> impl Iterator<Item = Self> {
        T::EventType::VARIANTS
            .as_ref()
            .iter()
            .cloned()
            .map(|segment| {
                let mut t_clone = self.clone();
                t_clone.push(segment);
                t_clone
            })
    }
}

/// The feedback from some oracle
pub trait OracleEvent: 'static + PartialEq {
    /// A branch gets pruned, if the oracle annotated it as DEAD
    const DEAD: &Self;
    /// A branch gets forcefully accepted, if the oracle annotated it as ACCEPTED
    const ACCEPTED: Option<&Self>;
}

/// The policey which describes how an Oracle event may be interpreted
pub trait SelectionPolicy {
    /// the type of oracle events this policy applies to
    type OracleEvent: OracleEvent;
    /// the scheduler prefers the greatest event according to this comparison
    fn compare(a: &Self::OracleEvent, b: &Self::OracleEvent) -> Ordering;

    /// the scheduler may prune this branch
    fn may_reject(s: &Self::OracleEvent) -> bool {
        s == Self::OracleEvent::DEAD
    }

    /// the scheduler should/may accept this branch
    fn may_accept(s: &Self::OracleEvent) -> bool {
        Self::OracleEvent::ACCEPTED.is_some_and(|ev| ev == s)
    }
}

#[macro_export]
/// generates an enum implementing `StaticEvent` and `HasLevelStorage` given the enum variants. The storage of this type is an array.
macro_rules! generate_static_event {
    (
        $vis:vis enum $name:ident {
            $($variant:ident),+ $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        $vis enum $name {
            $($variant),+
        }

        impl HasLevelStorage for $name {
            type LevelStorage<T> = [T; [ $( stringify!($variant) ),+ ].len() ];

            #[inline]
            fn storage_from_fn<T, F>(f: F) -> Self::LevelStorage<T>
            where
                F: FnMut(usize) -> T,
            {
                core::array::from_fn(f)
            }
        }

        impl StaticEvent for $name {
            const VARIANTS: &Self::LevelStorage<Self> = &[
                $( Self::$variant ),+
            ];
        }
    };
}
