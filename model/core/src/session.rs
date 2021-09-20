use crate::{source::SourceType, AsyncRt, Shared, SourceRegistry};
use serde::{Deserialize, Serialize};
use std::{any::Any, sync::Arc};
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
    vars: Arc<Vars>,
    source_params: Arc<dyn Any + Send + Sync> //_matcher_params: Arc<dyn Any + Send + Sync>
}

impl Session {
    pub fn start<'a, D>(
        rt: AsyncRt,
        vars: Arc<Vars>,
        source_params: Arc<dyn Any + Send + Sync>,
        source_registry: Arc<dyn SourceRegistry<'a, D>>
    ) -> Shared<Self>
    where
        D: serde::de::Deserializer<'a>
    {
        let this = Self {
            vars,
            source_params
        };
        this.main(rt, source_registry);
        let shared = Arc::new(RwLock::new(this));
        shared
    }

    fn main<'a, D>(&self, rt: AsyncRt, source_registry: Arc<dyn SourceRegistry<'a, D>>)
    where
        D: serde::de::Deserializer<'a>
    {
        // let (tx1, rx1) = mpsc::unbounded_channel();
        rt.spawn(async move {});
        ()
    }
}
