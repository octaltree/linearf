use crate::{AsyncRt, FlowId, Item, Shared, SourceRegistry};
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
    flows: Shared<VecDeque<(FlowId, Shared<Flow>)>>
}

pub struct Flow {}

impl Session {
    pub async fn start<'a, D, S>(
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
        this.main(rt, source_registry).await;
        Arc::new(this)
    }

    async fn main<'a, D, S>(&self, rt: AsyncRt, source_registry: &Arc<S>)
    where
        D: serde::de::Deserializer<'a>,
        S: SourceRegistry<'a, D> + 'static + Send + Sync
    {
        let (tx1, mut rx1) = mpsc::unbounded_channel();
        let (tx2, _rx2) = mpsc::unbounded_channel::<Item>();

        // TODO: match
        rt.spawn(async move {
            let send = |x| {
                if let Err(e) = tx2.send(x) {
                    log::error!("{:?}", e);
                }
            };
            loop {
                match rx1.recv().await {
                    Some(crate::source::Output::Item(x)) => {
                        send(x);
                    }
                    Some(crate::source::Output::Chunk(xs)) => {
                        for x in xs {
                            send(x);
                        }
                    }
                    None => break
                }
            }
        });

        rt.spawn(async move {});

        tokio::join!({
            source_registry.on_session_start(
                &rt,
                &self.vars.source,
                crate::source::Transmitter::new(tx1),
                (self.vars.clone(), self.source_params.clone())
            )
        });
    }

    #[inline]
    pub(crate) fn vars(&self) -> &Arc<Vars> { &self.vars }

    #[inline]
    pub(crate) fn source_params(&self) -> &Arc<dyn Any + Send + Sync> { &self.source_params }
}

impl Flow {
    pub async fn start<'a, D, S>(
        _rt: AsyncRt,
        _vars: Arc<Vars>,
        _source_params: Arc<dyn Any + Send + Sync>,
        _source_registry: &Arc<S>
    ) -> Shared<Self>
    where
        D: serde::de::Deserializer<'a>,
        S: SourceRegistry<'a, D> + 'static + Send + Sync
    {
        // TODO
        let this = Flow {};
        Arc::new(RwLock::new(this))
    }
}
