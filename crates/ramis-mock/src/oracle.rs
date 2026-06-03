use ramis_core::SelectionPolicy;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MockOracleEvent {
    Dead = 0,
    Accept = 2,
    Alive(u8) = 1,
}

pub struct MockPolicy;

impl SelectionPolicy for MockPolicy {
    type OracleEvent = MockOracleEvent;

    fn compare(a: &Self::OracleEvent, b: &Self::OracleEvent) -> core::cmp::Ordering {
        a.cmp(b)
    }

    fn may_reject(s: &Self::OracleEvent) -> bool {
        *s == MockOracleEvent::Dead
    }

    fn may_accept(s: &Self::OracleEvent) -> bool {
        *s == MockOracleEvent::Accept
    }
}
