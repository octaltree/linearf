use crate::{
    session::{Receiver, Sender, Vars},
    Error, Item, Shared
};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::{any::Any, sync::Arc};

pub trait SourceParams: DeserializeOwned + Serialize {}

pub struct Transmitter {
    tx: Sender<Vec<Item>>
}

impl Transmitter {
    pub fn new(tx: Sender<Vec<Item>>) -> Self { Self { tx } }

    #[inline]
    pub fn item(&self, i: Item) -> Result<(), Error> { Ok(self.tx.send(vec![i])?) }

    #[inline]
    pub fn chunk<A: Into<Vec<Item>>>(&self, xs: A) -> Result<(), Error> {
        Ok(self.tx.send(xs.into())?)
    }
}

#[async_trait]
trait FlowGenerator {
    type Params: SourceParams;

    async fn run(&mut self, args: Receiver<(Transmitter, (&Arc<Vars>, &Arc<Self::Params>))>);

    async fn reusable(
        &self,
        prev: (&Arc<Vars>, &Arc<Self::Params>),
        senario: (&Arc<Vars>, &Arc<Self::Params>)
    ) -> bool;
}

#[async_trait]
trait SimpleGenerator {
    type Params: SourceParams;

    async fn generate(&self, tx: Transmitter, senario: (&Arc<Vars>, &Arc<Self::Params>));

    async fn reusable(
        &self,
        prev: (&Arc<Vars>, &Arc<Self::Params>),
        senario: (&Arc<Vars>, &Arc<Self::Params>)
    ) -> bool;
}
