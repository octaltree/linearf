pub use crate::session::{BlankParams, ReusableContext};
use crate::{session::Vars, stream::Stream, AsyncRt, FlowId, Item, New, SessionId, Shared, State};
use serde::{de::DeserializeOwned, Serialize};
use std::{any::Any, pin::Pin, sync::Arc};

pub trait SourceParams: DeserializeOwned + Serialize {}

impl SourceParams for BlankParams {}

pub trait IsSource {
    type Params: SourceParams;
}

/// reusable and stream will be called for each flow
pub trait SimpleGenerator<P>: New + IsSource<Params = P> {
    fn into_source(self) -> Source<P>
    where
        Self: Sized + 'static + Send + Sync
    {
        Source::Simple(Arc::new(self))
    }

    fn stream(&self, senario: (&Arc<Vars>, &Arc<P>)) -> Pin<Box<dyn Stream<Item = Item>>>;

    /// This methods must not lock Shared<State>. you can get State from `ctx.state` instead of Shared<State>
    fn reusable(
        &self,
        ctx: ReusableContext<'_>,
        prev: (&Arc<Vars>, &Arc<P>),
        senario: (&Arc<Vars>, &Arc<P>)
    ) -> bool;
}

#[derive(Clone)]
pub enum Source<P> {
    Simple(Arc<dyn SimpleGenerator<P> + Send + Sync>)
}

pub trait SourceRegistry<'de, D>
where
    D: serde::de::Deserializer<'de>
{
    fn new(state: Shared<State>, rt: AsyncRt) -> Self
    where
        Self: Sized;

    fn parse(
        &self,
        _name: &str,
        _deserializer: D
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, D::Error> {
        Ok(None)
    }

    fn reusable(
        &self,
        _name: &str,
        _ctx: ReusableContext<'_>,
        _prev: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>),
        _senario: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>)
    ) -> bool
    where
        Self: Sized
    {
        false
    }

    fn stream(
        &self,
        _name: &str,
        _senario: (Arc<Vars>, Arc<dyn Any + Send + Sync>)
    ) -> Pin<Box<dyn Stream<Item = Item>>>
    where
        Self: Sized
    {
        Box::pin(crate::stream::empty())
    }
}
