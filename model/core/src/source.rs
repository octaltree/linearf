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
pub trait SimpleGenerator<L, R, P>: New<L, R> + IsSource<Params = P>
where
    L: crate::Linearf<R> + Send + Sync,
    R: crate::Registry
{
    fn into_source(self) -> Source<L, R, P>
    where
        Self: Sized + 'static + Send + Sync
    {
        Source::Simple(Arc::new(self))
    }

    fn stream(&self, senario: (&Arc<Vars>, &Arc<P>)) -> Pin<Box<dyn Stream<Item = Item>>>;

    /// This methods must not lock Shared<State>. you can get State from `ctx.state` instead of Shared<State>
    fn reusable(
        &self,
        ctx: ReusableContext,
        prev: (&Arc<Vars>, &Arc<P>),
        senario: (&Arc<Vars>, &Arc<P>)
    ) -> bool;
}

#[derive(Clone)]
pub enum Source<L, R, P> {
    Simple(Arc<dyn SimpleGenerator<L, R, P> + Send + Sync>)
}

pub trait SourceRegistry {
    fn names(&self) -> &[String] { &[] }

    fn parse<'de, D>(
        &self,
        _name: &str,
        _deserializer: D
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, D::Error>
    where
        D: serde::de::Deserializer<'de>
    {
        Ok(None)
    }

    fn reusable(
        &self,
        _name: &str,
        _ctx: ReusableContext,
        _prev: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>),
        _senario: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>)
    ) -> bool {
        false
    }

    fn stream(
        &self,
        _name: &str,
        _senario: (Arc<Vars>, Arc<dyn Any + Send + Sync>)
    ) -> Pin<Box<dyn Stream<Item = Item>>> {
        Box::pin(crate::stream::empty())
    }
}
