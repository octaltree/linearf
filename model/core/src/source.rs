use crate::{
    session::{Receiver, Sender, Vars},
    AsyncRt, Item, New, Shared, State
};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::{any::Any, sync::Arc};
use tokio::sync::RwLock;

pub trait SourceParams: DeserializeOwned + Serialize {}

impl SourceParams for () {}

#[derive(Debug)]
pub(crate) enum Output {
    Item(Item),
    Chunk(Vec<Item>)
}

#[derive(Debug)]
pub struct Transmitter {
    tx: Sender<Output>
}

impl Transmitter {
    pub(crate) fn new(tx: Sender<Output>) -> Self { Self { tx } }

    #[inline]
    pub fn item(&self, i: Item) {
        if let Err(e) = self.tx.send(Output::Item(i)) {
            log::error!("{:?}", e);
        }
    }

    #[inline]
    pub fn chunk<A: Into<Vec<Item>>>(&self, xs: A) {
        if let Err(e) = self.tx.send(Output::Chunk(xs.into())) {
            log::error!("{:?}", e);
        }
    }
}

pub trait IsSource {
    type Params: SourceParams;
}

#[async_trait]
pub trait SimpleGenerator<P>: New + IsSource<Params = P> {
    fn into_source(self) -> Source<P>
    where
        Self: Sized + 'static + Send + Sync
    {
        Source::Simple(Arc::new(RwLock::new(self)))
    }

    async fn generate(&self, tx: Transmitter, senario: (&Arc<Vars>, &Arc<P>));

    async fn reusable(&self, prev: (&Arc<Vars>, &Arc<P>), senario: (&Arc<Vars>, &Arc<P>)) -> bool;
}

#[async_trait]
pub trait FlowGenerator<P>: New + IsSource<Params = P> {
    fn into_source(self) -> Source<P>
    where
        Self: Sized + 'static + Send + Sync
    {
        Source::Flow(Arc::new(RwLock::new(self)))
    }

    async fn run(&mut self, args: Receiver<(Transmitter, (Arc<Vars>, Arc<P>))>);

    async fn reusable(&self, prev: (&Arc<Vars>, &Arc<P>), senario: (&Arc<Vars>, &Arc<P>)) -> bool;
}

#[derive(Clone)]
pub enum Source<P> {
    Simple(Shared<dyn SimpleGenerator<P> + Send + Sync>),
    Flow(Shared<dyn FlowGenerator<P> + Send + Sync>)
}

#[async_trait]
pub trait SourceRegistry<'de, D>
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
    ) -> bool
    where
        Self: Sized
    {
        false
    }

    async fn on_session_start(
        &self,
        _rt: &AsyncRt,
        _name: &str,
        _tx: Transmitter,
        _senario: (Arc<Vars>, Arc<dyn Any + Send + Sync>)
    ) where
        Self: Sized
    {
    }

    async fn on_flow_start(
        &self,
        _rt: &AsyncRt,
        _name: &str,
        _tx: Transmitter,
        _senario: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>)
    ) where
        Self: Sized
    {
    }
}
