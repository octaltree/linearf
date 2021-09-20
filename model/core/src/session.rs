use crate::{Shared, SourceRegistry};
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
        vars: Arc<Vars>,
        source_params: Arc<dyn Any + Send + Sync>,
        source_registry: Arc<dyn SourceRegistry<'a, D>>
    ) -> Shared<Self> {
        let this = Self {
            vars,
            source_params
        };
        let shared = Arc::new(RwLock::new(this));
        Self::main();
        shared
    }

    fn main() {}
}
