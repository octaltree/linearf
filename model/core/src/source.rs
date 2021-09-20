use crate::{
    session::{Receiver, Sender, Vars},
    Error, Item, New, Shared, State
};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

pub trait SourceParams: DeserializeOwned + Serialize {}

impl SourceParams for () {}

pub struct Transmitter {
    tx: Sender<Vec<Item>>
}

impl Transmitter {
    pub fn new(tx: Sender<Vec<Item>>) -> Self { Self { tx } }

    #[inline]
    pub fn item(&self, i: Item) {
        if let Err(e) = self.tx.send(vec![i]) {
            log::error!("{:?}", e);
        }
    }

    #[inline]
    pub fn chunk<A: Into<Vec<Item>>>(&self, xs: A) {
        if let Err(e) = self.tx.send(xs.into()) {
            log::error!("{:?}", e);
        }
    }
}

pub trait HasSourceParams {
    type Params: SourceParams;
}

#[async_trait]
pub trait SimpleGenerator<P>: New + HasSourceParams<Params = P> {
    fn into_source(self: Self) -> Source<P>
    where
        Self: Sized + 'static + Send + Sync
    {
        Source::Simple(Arc::new(RwLock::new(self)))
    }

    async fn generate(&self, tx: Transmitter, senario: (&Arc<Vars>, &Arc<P>));

    async fn reusable(&self, prev: (&Arc<Vars>, &Arc<P>), senario: (&Arc<Vars>, &Arc<P>)) -> bool;
}

#[async_trait]
pub trait FlowGenerator<P>: New + HasSourceParams<Params = P> {
    fn into_source(self: Self) -> Source<P>
    where
        Self: Sized + 'static + Send + Sync
    {
        Source::Flow(Arc::new(RwLock::new(self)))
    }

    async fn run(&mut self, args: Receiver<(Transmitter, (&Arc<Vars>, &Arc<P>))>);

    async fn reusable(&self, prev: (&Arc<Vars>, &Arc<P>), senario: (&Arc<Vars>, &Arc<P>)) -> bool;
}

pub enum Source<P> {
    Simple(Shared<dyn SimpleGenerator<P> + Send + Sync>),
    Flow(Shared<dyn FlowGenerator<P> + Send + Sync>)
}

pub enum SourceType {
    Simple,
    Flow
}

impl<P> From<&Source<P>> for SourceType {
    fn from(s: &Source<P>) -> Self {
        match s {
            Source::Simple(_) => SourceType::Simple,
            Source::Flow(_) => SourceType::Flow
        }
    }
}
