use crate::{session::Sender, Flow, Item, New, Session, Shared};
use std::sync::Arc;

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
    // TODO: error notification
    async fn generate(
        &mut self,
        tx: Transmitter,
        flow: &Arc<Flow>
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn reusable(&self, prev: &Session, flow: &Arc<Flow>) -> bool;
}

/// Results change dependening on the query
#[async_trait]
pub trait DynamicGenerator: New + Send + Sync {
    async fn generate(
        &mut self,
        tx: Transmitter,
        flow: &Arc<Flow>,
        query: &str
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn reusable(&self, prev: &Session, flow: &Arc<Flow>, query: &str) -> bool;
}

#[derive(Clone)]
pub enum Source {
    Static(Shared<dyn Generator>),
    Dynamic(Shared<dyn DynamicGenerator>)
}
