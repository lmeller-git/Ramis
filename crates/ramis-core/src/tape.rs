use core::cmp::Ordering;

pub trait EventReplay: Sized {
    type EventType;
    fn push(&mut self, event: Self::EventType);
    fn is_prefix_of(&self, other: &Self) -> bool;
    fn extend_with_slice(&mut self, slice: &[Self::EventType]);
}

pub trait DynamicEventReplay: EventReplay {
    fn children(&self) -> impl Iterator<Item = Self>;
}

pub trait StaticEvent: Sized + 'static {
    const VARIANTS: &'static [Self];
    const BRANCHING_FACTOR: usize;
}

#[allow(non_camel_case_types)]
pub trait _is_valid {
    #[allow(non_upper_case_globals)]
    const _is_valid: ();
}

impl<T> _is_valid for T
where
    T: StaticEvent,
{
    const _is_valid: () = const {
        assert!(
            Self::VARIANTS.len() == Self::BRANCHING_FACTOR,
            "branching factor of a staticevent should be == number of variants"
        )
    };
}

impl<T> DynamicEventReplay for T
where
    T: EventReplay + Clone,
    T::EventType: StaticEvent + Clone,
{
    fn children(&self) -> impl Iterator<Item = Self> {
        T::EventType::VARIANTS.iter().cloned().map(|segment| {
            let mut t_clone = self.clone();
            t_clone.push(segment);
            t_clone
        })
    }
}

pub trait SelectionPolicy {
    type OracleEvent;
    fn compare(a: &Self::OracleEvent, b: &Self::OracleEvent) -> Ordering;
    fn may_reject(s: &Self::OracleEvent) -> bool;
    fn may_accept(s: &Self::OracleEvent) -> bool;
}
