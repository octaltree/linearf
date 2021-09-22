use crate::{AsyncRt, Shared, SourceRegistry};
use serde::{Deserialize, Serialize};
use std::{any::Any, collections::VecDeque, sync::Arc};
use tokio::sync::{mpsc, RwLock};

pub type Sender<T> = mpsc::UnboundedSender<T>;
pub type Receiver<T> = mpsc::UnboundedReceiver<T>;

pub fn new_channel<T>() -> (Sender<T>, Receiver<T>) { mpsc::unbounded_channel() }

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Vars {
    pub(crate) source: String,
    pub(crate) matcher: String,
    pub(crate) query: String
}

pub struct Session {
    vars: Arc<Vars>,
    source_params: Arc<dyn Any + Send + Sync>, // Arc<dyn Any + Send + Sync>
    flows: Shared<VecDeque<Shared<Flow>>>
}

pub struct Flow {}

impl Session {
    pub fn start<'a, D, S>(
        rt: AsyncRt,
        vars: Arc<Vars>,
        source_params: Arc<dyn Any + Send + Sync>,
        source_registry: &Arc<S>
    ) -> Arc<Self>
    where
        D: serde::de::Deserializer<'a>,
        S: SourceRegistry<'a, D> + 'static + Send + Sync
    {
        let this = Self {
            vars,
            source_params,
            flows: Default::default()
        };
        this.main(rt, source_registry);
        Arc::new(this)
    }

    fn main<'a, D, S>(&self, rt: AsyncRt, source_registry: &Arc<S>)
    where
        D: serde::de::Deserializer<'a>,
        S: SourceRegistry<'a, D> + 'static + Send + Sync
    {
        let (tx1, mut rx1) = mpsc::unbounded_channel();
        let source_name = &self.vars.source;
        rt.block_on(async {
            source_registry
                .on_session_start(
                    &rt,
                    source_name,
                    crate::source::Transmitter::new(tx1),
                    (self.vars.clone(), self.source_params.clone())
                )
                .await;
        });
        rt.spawn(async move {
            loop {
                match rx1.recv().await {
                    Some(crate::source::Output::Item(x)) => {
                        log::info!("{:?}", x);
                    }
                    Some(crate::source::Output::Chunk(xs)) => {
                        for x in xs {
                            log::info!("{:?}", x);
                        }
                    }
                    None => break
                }
            }
        });
        ()
    }

    #[inline]
    pub(crate) fn vars(&self) -> &Arc<Vars> { &self.vars }

    #[inline]
    pub(crate) fn source_params(&self) -> &Arc<dyn Any + Send + Sync> { &self.source_params }
}
