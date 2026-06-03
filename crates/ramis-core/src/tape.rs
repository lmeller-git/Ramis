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

pub trait HasLevelStorage {
    type LevelStorage<T>: AsRef<[T]> + AsMut<[T]>;
    fn storage_from_fn<T, F>(f: F) -> Self::LevelStorage<T>
    where
        F: FnMut(usize) -> T;
}

pub trait StaticEvent: HasLevelStorage + Sized + 'static {
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

pub trait SelectionPolicy {
    type OracleEvent;
    fn compare(a: &Self::OracleEvent, b: &Self::OracleEvent) -> Ordering;
    fn may_reject(s: &Self::OracleEvent) -> bool;
    fn may_accept(s: &Self::OracleEvent) -> bool;
}
