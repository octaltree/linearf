use crate::{session::Sender, Flow, Item, New, Session, Shared, Snapshot};
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll}
};

pub struct Transmitter {
    tx: Sender<Vec<Item>>
}

impl Transmitter {
    pub fn new(tx: Sender<Vec<Item>>) -> Self { Self { tx } }

    #[inline]
    pub fn item(&self, i: Item) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.tx.send(vec![i])?)
    }

    #[inline]
    pub fn chunk<A: Into<Vec<Item>>>(
        &self,
        xs: A
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.tx.send(xs.into())?)
    }
}

/// Source that are not affected by query
/// NOTE: Source have the potential to have itw own cache, so make them live longer.
#[async_trait]
pub trait Generator: New + Send + Sync {
    // TODO: mut posession
    // TODO: error notification
    async fn generate(
        &mut self,
        tx: Transmitter,
        flow: &Arc<Flow>,
        snapshot: Snapshot
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn reusable(&self, prev: &Session, flow: &Arc<Flow>, snapshot: Snapshot) -> bool;
}

/// Results change dependening on the query
#[async_trait]
pub trait DynamicGenerator: New + Send + Sync {
    async fn generate(
        &mut self,
        tx: Transmitter,
        flow: &Arc<Flow>,
        snapshot: Snapshot,
        query: &str
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn reusable(
        &self,
        prev: &Session,
        flow: &Arc<Flow>,
        snapshot: Snapshot,
        query: &str
    ) -> bool;
}

#[derive(Clone)]
pub enum Source {
    Static(Shared<dyn Generator>),
    Dynamic(Shared<dyn DynamicGenerator>)
}

/// has priority queue
struct SourceRunner {
    source: HashMap<String, Source>
}

impl SourceRunner {
    fn insert() {}
}

impl Future for SourceRunner {
    type Output = i32;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> { todo!() }
}
