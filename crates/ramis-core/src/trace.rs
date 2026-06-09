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

    /// determines how wether the scheduler should advance into this branch (or its siblings)
    fn branch_directive(s: &Self::OracleEvent) -> BranchDirective {
        match s {
            _ if s == <Self::OracleEvent as OracleEvent>::DEAD => BranchDirective::Prune,
            _ if <Self::OracleEvent as OracleEvent>::ACCEPTED.is_some_and(|acc| acc == s) => {
                BranchDirective::Force
            }
            _ => BranchDirective::Proceed,
        }
    }
}

/// Determines how the scheduler should treat a branch based on its oracle evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BranchDirective {
    /// Discard this branch and prune the subtree.
    Prune,
    /// The parent node should not advance into any of its children, unless forced to do so
    Hold,
    /// The parent node may advance into any of its `BranchDirective::Proceed` children, unless specified otherwise by a sibling
    Proceed,
    /// The parent node should advance into this branch independently of sibling state
    Force,
    /// It is not specified what happens to this node. The scheduler may decide
    Unspecified,
}

impl BranchDirective {
    /// combines two BranchDirective's for one branch
    pub fn and_self(self, other: &Self) -> Self {
        match (self, other) {
            (Self::Prune, _) => Self::Prune,
            (_, Self::Prune) => Self::Prune,

            (Self::Force, _) => Self::Force,
            (_, Self::Force) => Self::Force,

            (Self::Hold, _) => Self::Hold,
            (_, Self::Hold) => Self::Hold,

            (Self::Proceed, Self::Proceed) => Self::Proceed,

            (Self::Unspecified, _) => *other,
            (_, Self::Unspecified) => self,
        }
    }

    /// combines two BranchDirective's for two sibling branches. This indicates the status of the parent of the combined siblings
    pub fn and_across(self, other: &Self) -> Self {
        match (self, other) {
            (Self::Prune, Self::Prune) => Self::Prune,

            (Self::Force, _) => Self::Force,
            (_, Self::Force) => Self::Force,

            (Self::Unspecified, _) => Self::Unspecified,
            (_, Self::Unspecified) => Self::Unspecified,

            (Self::Prune, _) => Self::Prune,
            (_, Self::Prune) => Self::Prune,

            (Self::Hold, _) => Self::Hold,
            (_, Self::Hold) => Self::Hold,

            (Self::Proceed, Self::Proceed) => Self::Proceed,
        }
    }

    /// Determines wether this state is ready for advancement
    pub fn is_ready(&self) -> bool {
        match self {
            Self::Force => true,
            Self::Proceed => true,
            Self::Prune => true,
            Self::Hold => false,
            Self::Unspecified => false,
        }
    }
}

#[macro_export]
/// generates an enum implementing `StaticEvent` and `HasLevelStorage` given the enum variants. The storage of this type is an array.
macro_rules! generate_static_event {
    (
        $(#[$attr:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$v_attr:meta])*
                $variant:ident
            ),+ $(,)?
        }
    ) => {
        $(#[$attr])*
        $vis enum $name {
            $(
                $(#[$v_attr])*
                $variant
            ),+
        }

        impl $crate::HasLevelStorage for $name {
            type LevelStorage<T> = [T; [ $( stringify!($variant) ),+ ].len() ];

            #[inline]
            fn storage_from_fn<T, F>(f: F) -> Self::LevelStorage<T>
            where
                F: FnMut(usize) -> T,
            {
                core::array::from_fn(f)
            }
        }

        impl $crate::StaticEvent for $name {
            const VARIANTS: &Self::LevelStorage<Self> = &[
                $( Self::$variant ),+
            ];
        }
    };
}
