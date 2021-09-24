use crate::{AsyncRt, FlowId, Item, MatcherRegistry, Senario, Shared, SourceRegistry};
use serde::{Deserialize, Serialize};
use std::{any::Any, collections::VecDeque, sync::Arc};
use tokio::sync::{mpsc, RwLock};

pub type Sender<T> = mpsc::UnboundedSender<T>;
pub type Receiver<T> = mpsc::UnboundedReceiver<T>;

pub fn new_channel<T>() -> (Sender<T>, Receiver<T>) { mpsc::unbounded_channel() }

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Vars {
    pub source: String,
    pub matcher: String,
    pub query: String
}

pub struct Session {
    vars: Arc<Vars>,
    source_params: Arc<dyn Any + Send + Sync>, // Arc<dyn Any + Send + Sync>
    matcher_params: Arc<dyn Any + Send + Sync>, // Arc<dyn Any + Send + Sync>
    flows: Shared<VecDeque<(FlowId, Shared<Flow>)>>
}

pub struct Flow {
    vars: Arc<Vars>,
    source_params: Arc<dyn Any + Send + Sync>, // Arc<dyn Any + Send + Sync>
    matcher_params: Arc<dyn Any + Send + Sync>  // Arc<dyn Any + Send + Sync>
}

impl Session {
    pub async fn start<'a, D, S, M>(
        rt: AsyncRt,
        senario: Senario<Arc<Vars>, Arc<dyn Any + Send + Sync>>,
        source_registry: &Arc<S>,
        matcher_registry: &Arc<M>
    ) -> Arc<Self>
    where
        D: serde::de::Deserializer<'a>,
        S: SourceRegistry<'a, D> + 'static + Send + Sync,
        M: MatcherRegistry<'a, D> + 'static + Send + Sync
    {
        let Senario {
            linearf: vars,
            source: source_params,
            matcher: matcher_params
        } = senario.clone();
        let this = Self {
            vars,
            source_params,
            matcher_params,
            flows: Default::default()
        };
        let flow = Flow::start(rt, senario, source_registry, matcher_registry, true)
            .await
            .unwrap();
        {
            let mut flows = this.flows.write().await;
            flows.push_back((FlowId(1), flow));
        }
        Arc::new(this)
    }

    pub async fn tick<'a, D, S, M>(
        rt: AsyncRt,
        senario: Senario<Arc<Vars>, Arc<dyn Any + Send + Sync>>,
        source_registry: &Arc<S>,
        matcher_registry: &Arc<M>
    ) -> Option<FlowId>
    where
        D: serde::de::Deserializer<'a>,
        S: SourceRegistry<'a, D> + 'static + Send + Sync,
        M: MatcherRegistry<'a, D> + 'static + Send + Sync
    {
        let flow = Flow::start(rt, senario, source_registry, matcher_registry, false).await?;
        None
    }

    #[inline]
    pub(crate) fn vars(&self) -> &Arc<Vars> { &self.vars }

    #[inline]
    pub(crate) fn source_params(&self) -> &Arc<dyn Any + Send + Sync> { &self.source_params }

    #[inline]
    pub(crate) fn matcher_params(&self) -> &Arc<dyn Any + Send + Sync> { &self.matcher_params }
}

impl Flow {
    pub async fn start<'a, D, S, M>(
        rt: AsyncRt,
        senario: Senario<Arc<Vars>, Arc<dyn Any + Send + Sync>>,
        source_registry: &Arc<S>,
        matcher_registry: &Arc<M>,
        first: bool
    ) -> Option<Shared<Self>>
    where
        D: serde::de::Deserializer<'a>,
        S: SourceRegistry<'a, D> + 'static + Send + Sync,
        M: MatcherRegistry<'a, D> + 'static + Send + Sync
    {
        let Senario {
            linearf: vars,
            source: source_params,
            matcher: matcher_params
        } = senario;
        let this = Flow {
            vars,
            source_params,
            matcher_params
        };
        this.main(rt, source_registry, matcher_registry).await;
        Some(Arc::new(RwLock::new(this)))
    }

    async fn main<'a, D, S, M>(
        &self,
        rt: AsyncRt,
        source_registry: &Arc<S>,
        matcher_registry: &Arc<M>
    ) where
        D: serde::de::Deserializer<'a>,
        S: SourceRegistry<'a, D> + 'static + Send + Sync,
        M: MatcherRegistry<'a, D> + 'static + Send + Sync
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
}
