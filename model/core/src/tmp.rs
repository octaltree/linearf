use super::*;
use std::cmp::Ordering;

/// Bigger f64 is higher priority. If the order is not determined, bigger idx is lower priority.
/// If f64 is NaN, it will be excluded.
#[derive(Clone, Copy)]
struct F64Ord {
    x: f64,
    idx: usize
}

impl Score for F64Ord {
    fn is_excluded(&self) -> bool { self.x.is_nan() }
}

impl PartialEq for F64Ord {
    fn eq(&self, other: &Self) -> bool { self.x.eq(&other.x) && self.idx.eq(&other.idx) }
}

impl Eq for F64Ord {}

impl PartialOrd for F64Ord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for F64Ord {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.x <= other.x, self.x >= other.x) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => match self.idx.cmp(&other.idx) {
                Ordering::Less => Ordering::Greater,
                Ordering::Equal => Ordering::Equal,
                Ordering::Greater => Ordering::Less
            }
        }
    }
}

/// Bigger u16 is higher priority. If the order is not determined, bigger idx is lower priority.
/// If u16 is 0, it will be excluded.
#[derive(Clone, Copy)]
struct U16Ord {
    x: u16,
    idx: usize
}

impl Score for U16Ord {
    fn is_excluded(&self) -> bool { self.x == 0 }
}

impl PartialEq for U16Ord {
    fn eq(&self, other: &Self) -> bool { self.x.eq(&other.x) && self.idx.eq(&other.idx) }
}

impl Eq for U16Ord {}

impl PartialOrd for U16Ord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for U16Ord {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.x <= other.x, self.x >= other.x) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => match self.idx.cmp(&other.idx) {
                Ordering::Less => Ordering::Greater,
                Ordering::Equal => Ordering::Equal,
                Ordering::Greater => Ordering::Less
            }
        }
    }
}
