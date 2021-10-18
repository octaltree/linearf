use std::cmp::Ordering;

/// Items will be displayed in v DESC, item_id ASC.
/// No guarantee of order when it is equal.
#[derive(Debug, PartialEq, Eq)]
pub struct Score {
    pub item_id: u32,
    /// If empty, the item will not be displayed
    pub v: Vec<i16>
}

impl Score {
    pub fn new<V: Into<Vec<i16>>>(item_id: u32, v: V) -> Self {
        Self {
            item_id,
            v: v.into()
        }
    }

    /// If true, the item will not be displayed.
    #[inline]
    pub fn should_be_excluded(&self) -> bool { self.v.is_empty() }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        for (a, b) in self.v.iter().zip(other.v.iter()) {
            match a.cmp(b) {
                Ordering::Less => return Some(Ordering::Greater),
                Ordering::Greater => return Some(Ordering::Less),
                _ => {}
            }
        }
        Some(self.item_id.cmp(&other.item_id))
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> Ordering {
        for (a, b) in self.v.iter().zip(other.v.iter()) {
            match a.cmp(b) {
                Ordering::Less => return Ordering::Greater,
                Ordering::Greater => return Ordering::Less,
                _ => {}
            }
        }
        self.item_id.cmp(&other.item_id)
    }
}
