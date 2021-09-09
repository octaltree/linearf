use crate::{Flow, Item, New};
use std::sync::Arc;

#[async_trait]
pub trait Matcher: New {
    async fn score(&mut self, flow: &Arc<Flow>, query: &str, item: &Item) -> Score;
}

/// Items will be displayed in descending order
/// No guarantee of order when it is equal. You should use idx to make it less equal.
pub struct Score {
    pub id: u32,
    pub v: Vec<i16>
}

impl Score {
    pub fn new(id: u32, v: Vec<i16>) -> Self { Self { id, v } }

    /// If true, the item will not be displayed.
    pub fn should_be_excluded(&self) -> bool { self.v.is_empty() }
}

impl PartialEq for Score {
    fn eq(&self, other: &Self) -> bool { self.v == other.v && !self.v.is_empty() }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { None }
}
