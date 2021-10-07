pub use super::common_interface::*;
use std::cmp::Ordering;

pub trait MatcherRegistry {
    fn names(&self) -> &[SmartString] { &[] }

    fn parse<'de, D>(
        &self,
        _name: &str,
        _deserializer: D
    ) -> Option<Result<Arc<dyn Any + Send + Sync>, D::Error>>
    where
        D: serde::de::Deserializer<'de>
    {
        None
    }

    fn score(
        &self,
        _name: &str,
        _senario: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>),
        items: impl Stream<Item = Arc<Item>> + Send + Sync + 'static
    ) -> Pin<Box<dyn Stream<Item = WithScore> + Send + Sync>> {
        use futures::StreamExt;
        Box::pin(items.map(|i| {
            let score = Arc::new(Score::new(i.id, []));
            (i, score)
        }))
    }

    fn reusable(
        &self,
        _name: &str,
        _prev: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>),
        _senario: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>)
    ) -> Reusable {
        Reusable::Same
    }
}

#[derive(Clone)]
pub enum Matcher<L, P> {
    Simple(Arc<dyn SimpleScorer<L, P> + Send + Sync>)
}

pub trait SimpleScorer<L, P>: IsMatcher<Params = P>
where
    L: Linearf + Send + Sync
{
    fn score(&self, senario: (&Arc<Vars>, &Arc<P>), item: &Arc<Item>) -> Score;

    /// It will be called for every flow and may be reused across sessions.
    fn reusable(&self, prev: (&Arc<Vars>, &Arc<P>), senario: (&Arc<Vars>, &Arc<P>)) -> Reusable;
}

pub trait IsMatcher {
    type Params: MatcherParams;
}

pub trait MatcherParams: DeserializeOwned + Serialize {}

impl MatcherParams for BlankParams {}

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
                Ordering::Less => return Some(Ordering::Less),
                Ordering::Greater => return Some(Ordering::Greater),
                _ => {}
            }
        }
        Some(match self.item_id.cmp(&other.item_id) {
            Ordering::Less => Ordering::Greater,
            Ordering::Greater => Ordering::Less,
            Ordering::Equal => Ordering::Equal
        })
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> Ordering {
        for (a, b) in self.v.iter().zip(other.v.iter()) {
            match a.cmp(b) {
                Ordering::Less => return Ordering::Less,
                Ordering::Greater => return Ordering::Greater,
                _ => {}
            }
        }
        other.item_id.cmp(&self.item_id)
    }
}

pub type WithScore = (Arc<Item>, Arc<Score>);
