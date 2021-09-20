use crate::{AsyncRt, Shared, SourceRegistry};
use serde::{Deserialize, Serialize};
use std::{any::Any, collections::VecDeque, sync::Arc};
use tokio::sync::{mpsc, RwLock};

pub(crate) type Sender<T> = mpsc::UnboundedSender<T>;
pub(crate) type Receiver<T> = mpsc::UnboundedReceiver<T>;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Vars {
    pub(crate) source: String,
    pub(crate) matcher: String,
    pub(crate) query: String
}

pub struct Session {
    pub(crate) vars: Arc<Vars>,
    pub(crate) source_params: Arc<dyn Any + Send + Sync>, // Arc<dyn Any + Send + Sync>
    flows: Shared<VecDeque<Shared<Flow>>>
}

pub struct Flow {}

impl Session {
    pub fn start<'a, D>(
        rt: AsyncRt,
        vars: Arc<Vars>,
        source_params: Arc<dyn Any + Send + Sync>,
        source_registry: Arc<dyn SourceRegistry<'a, D>>
    ) -> Arc<Self>
    where
        D: serde::de::Deserializer<'a>
    {
        let this = Self {
            vars,
            source_params,
            flows: Default::default()
        };
        this.main(rt, source_registry);
        Arc::new(this)
    }

    fn main<'a, D>(&self, rt: AsyncRt, source_registry: Arc<dyn SourceRegistry<'a, D>>)
    where
        D: serde::de::Deserializer<'a>
    {
        // let (tx1, rx1) = mpsc::unbounded_channel();
        rt.spawn(async move {});
        ()
    }

    #[inline]
    pub(crate) fn vars(&self) -> &Arc<Vars> { &self.vars }

    #[inline]
    pub(crate) fn source_params(&self) -> &Arc<dyn Any + Send + Sync> { &self.source_params }
}
