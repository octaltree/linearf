pub use crate::session::BlankParams;
use crate::{
    session::{Receiver, Sender, Vars},
    source, Item, New, Shared, State
};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::{any::Any, cmp::Ordering, sync::Arc};
use tokio::sync::RwLock;

pub trait MatcherParams: DeserializeOwned + Serialize {}

impl MatcherParams for BlankParams {}

pub type Output = crate::session::Output<(Arc<Item>, Score)>;

pub trait IsMatcher {
    type Params: MatcherParams;
}

#[async_trait]
pub trait SimpleScorer<P>: New + IsMatcher<Params = P> {
    fn into_matcher(self) -> Matcher<P>
    where
        Self: Sized + 'static + Send + Sync
    {
        Matcher::Simple(Arc::new(self))
    }

    // TODO: remove async
    async fn score(&self, senario: (&Arc<Vars>, &Arc<P>), item: &Arc<Item>) -> Score;

    async fn reusable(&self, prev: (&Arc<Vars>, &Arc<P>), senario: (&Arc<Vars>, &Arc<P>)) -> bool;
}

#[derive(Clone)]
pub enum Matcher<P> {
    Simple(Arc<dyn SimpleScorer<P> + Send + Sync>)
}

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
        match self.item_id.cmp(&other.item_id) {
            Ordering::Less => Ordering::Greater,
            Ordering::Greater => Ordering::Less,
            Ordering::Equal => Ordering::Equal
        }
    }
}

#[async_trait]
pub trait MatcherRegistry<'de, D>
where
    D: serde::de::Deserializer<'de>
{
    fn new(state: Shared<State>) -> Self
    where
        Self: Sized;

    fn parse(
        &self,
        _name: &str,
        _deserializer: D
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, D::Error> {
        Ok(None)
    }

    async fn reusable(
        &self,
        _name: &str,
        _prev: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>),
        _senario: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>)
    ) -> bool {
        false
    }

    async fn score<'a>(
        &self,
        _name: &str,
        _rx: Receiver<crate::source::Output>,
        _tx: Sender<Output>,
        _senario: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>)
    ) {
    }
}
