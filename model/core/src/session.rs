pub mod sorted;

pub use crate::matcher::Score;
use crate::{matcher, AsyncRt, Error, FlowId, Item, MatcherRegistry, Senario, SourceRegistry};
use serde::{Deserialize, Serialize};
use sorted::Sorted;
use std::{any::Any, collections::VecDeque, sync::Arc};
use tokio::sync::mpsc;

pub type Sender<T> = mpsc::UnboundedSender<T>;
pub type Receiver<T> = mpsc::UnboundedReceiver<T>;

pub fn new_channel<T>() -> (Sender<T>, Receiver<T>) { mpsc::unbounded_channel() }

#[derive(Debug)]
pub enum Output<T> {
    Item(T),
    Chunk(Vec<T>)
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Vars {
    pub source: String,
    pub matcher: String,
    pub query: String
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum BlankParams {
    Unit(()),
    Obj {}
}

pub struct Session {
    // Session must have one or more flows
    flows: VecDeque<(FlowId, Flow)>
}

pub struct Flow {
    vars: Arc<Vars>,
    source_params: Arc<dyn Any + Send + Sync>, // Arc<dyn Any + Send + Sync>
    matcher_params: Arc<dyn Any + Send + Sync>, // Arc<dyn Any + Send + Sync>
    sorted: Sorted
}

impl Session {
    pub async fn start<'a, D, S, M>(
        rt: AsyncRt,
        source_registry: Arc<S>,
        matcher_registry: Arc<M>,
        senario: Senario<Arc<Vars>, Arc<dyn Any + Send + Sync>>
    ) -> Self
    where
        D: serde::de::Deserializer<'a>,
        S: SourceRegistry<'a, D> + 'static + Send + Sync,
        M: MatcherRegistry<'a, D> + 'static + Send + Sync
    {
        let mut this = Self {
            flows: Default::default()
        };
        let flow = Flow::start(rt, senario, source_registry, matcher_registry, true)
            .await
            .unwrap();
        this.flows.push_back((FlowId(1), flow));
        this
    }

    pub async fn tick<'a, D, S, M>(
        &mut self,
        rt: AsyncRt,
        source_registry: Arc<S>,
        matcher_registry: Arc<M>,
        senario: Senario<Arc<Vars>, Arc<dyn Any + Send + Sync>>
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
    pub(crate) fn flows(&self) -> &VecDeque<(FlowId, Flow)> { &self.flows }

    #[inline]
    pub fn last_flow(&self) -> (FlowId, &Flow) {
        let last = &self.flows[self.flows.len() - 1];
        (last.0, &last.1)
    }

    pub(crate) fn resume_flow(&mut self, id: FlowId) -> Result<(), Error> {
        let flow = self
            .remove_flow(id)
            .ok_or_else(|| format!("Flow {:?} is not found", id))?;
        self.flows.push_back((id, flow));
        Ok(())
    }

    fn remove_flow(&mut self, flow: FlowId) -> Option<Flow> {
        if let Some(idx) = self
            .flows
            .iter()
            .enumerate()
            .map(|(idx, (id, _))| (idx, id))
            .find(|(_, &id)| id == flow)
            .map(|(idx, _)| idx)
        {
            self.flows.remove(idx).map(|(_, s)| s)
        } else {
            None
        }
    }
}

impl Flow {
    pub async fn start<'a, D, S, M>(
        rt: AsyncRt,
        senario: Senario<Arc<Vars>, Arc<dyn Any + Send + Sync>>,
        source_registry: Arc<S>,
        matcher_registry: Arc<M>,
        first: bool
    ) -> Option<Self>
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
        let mut this = Flow {
            vars,
            source_params,
            matcher_params,
            sorted: Sorted::new(rt.clone())
        };
        this.main(rt, source_registry, matcher_registry, first)
            .await?;
        Some(this)
    }

    async fn main<'a, D, S, M>(
        &mut self,
        rt: AsyncRt,
        source_registry: Arc<S>,
        matcher_registry: Arc<M>,
        first: bool
    ) -> Option<()>
    where
        D: serde::de::Deserializer<'a>,
        S: SourceRegistry<'a, D> + 'static + Send + Sync,
        M: MatcherRegistry<'a, D> + 'static + Send + Sync
    {
        let (tx1, rx1) = new_channel();
        let (tx2, rx2) = new_channel();
        self.sorted.start(rx2);
        self.run_matcher(&rt, matcher_registry, rx1, tx2);
        let run = if first {
            source_registry
                .on_session_start(
                    &rt,
                    &self.vars.source,
                    crate::source::Transmitter::new(tx1),
                    (self.vars.clone(), self.source_params.clone())
                )
                .await;
            true
        } else {
            source_registry
                .on_flow_start(
                    &rt,
                    &self.vars.source,
                    crate::source::Transmitter::new(tx1),
                    (&self.vars, &self.source_params)
                )
                .await
        };
        run.then(|| ())
    }

    fn run_matcher<'a, D, M>(
        &self,
        rt: &AsyncRt,
        matcher_registry: Arc<M>,
        mut rx1: Receiver<crate::source::Output>,
        tx2: Sender<matcher::Output>
    ) where
        D: serde::de::Deserializer<'a>,
        M: MatcherRegistry<'a, D> + 'static + Send + Sync
    {
        let vars = self.vars.clone();
        let params = self.matcher_params.clone();
        rt.spawn(async move {
            let start = std::time::Instant::now();
            matcher_registry
                .score(&vars.matcher, rx1, tx2, (&vars, &params))
                .await;
            log::debug!("root matcher {:?}", std::time::Instant::now() - start);
        });
    }

    #[inline]
    pub(crate) fn vars(&self) -> &Arc<Vars> { &self.vars }

    #[inline]
    pub(crate) fn source_params(&self) -> &Arc<dyn Any + Send + Sync> { &self.source_params }

    #[inline]
    pub(crate) fn matcher_params(&self) -> &Arc<dyn Any + Send + Sync> { &self.matcher_params }
}
