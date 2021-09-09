use crate::{
    matcher::{Matcher, Score},
    source::{Source, Transmitter},
    Flow, Item, Query, Shared
};
use std::sync::Arc;
use tokio::{
    runtime::Handle,
    sync::{mpsc, RwLock},
    task::JoinHandle
};

pub(crate) type Sender<T> = mpsc::UnboundedSender<T>;

/// State being calculated based on flow
pub struct Session {
    rt: Handle,
    flow: Arc<Flow>,
    source: Source,
    matcher: Shared<dyn Matcher>,
    // TODO: items for each query
    // TODO: query's items for dynamic
    items: Vec<Item>
}

impl Session {
    #[inline]
    pub(crate) fn flow(&self) -> &Arc<Flow> { &self.flow }
}

impl Session {
    pub fn start(
        rt: Handle,
        flow: Arc<Flow>,
        source: Source,
        matcher: Shared<dyn Matcher>,
        query: Query
    ) -> Shared<Self> {
        // TODO: query at start
        let this = Self {
            rt: rt.clone(),
            flow,
            source,
            matcher,
            items: Vec::new()
        };
        let shared = Arc::new(RwLock::new(this));
        Session::main(rt, shared.clone(), query);
        shared
    }

    // TODO: stop threads
    fn main(rt: Handle, this: Arc<RwLock<Session>>, query: Query) {
        let (tx1, rx1) = mpsc::unbounded_channel();
        let source_handle = source(&rt, this.clone(), &Query, tx1);
        let (tx2, rx2) = mpsc::unbounded_channel();
        let matcher_handle = matcher(&rt, this.clone(), &Query, rx1, tx2);
        // Source::start(this.flow()))
        // rt.spawn(Session::start(tx, this));
        // let (tx2, rx2) = mpsc::unbounded_channel();
        // loop {
        //    this.upgrade()?;
        //}
    }

    pub fn query(&mut self, query: Query) { todo!() }

    #[inline]
    pub fn count(&self) -> usize { self.items.len() }

    pub fn items(&self, start: usize, stop: usize) -> Option<&[Item]> {
        let l = self.items.len();
        if start <= l && stop <= l {
            Some(&self.items[start..stop])
        } else {
            None
        }
    }
}

fn source(
    rt: &Handle,
    this: Arc<RwLock<Session>>,
    tx: Sender<Vec<Item>>
) -> JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>> {
    rt.spawn(async move {
        let sess = &mut this.write().await;
        match &sess.source {
            Source::Static(s) => {
                let s = &mut s.write().await;
                s.generate(Transmitter::new(tx), &sess.flow).await
            }
            Source::Dynamic(s) => {
                // TODO
                Ok(())
            }
        }
    })
}

fn matcher(
    rt: &Handle,
    this: Arc<RwLock<Session>>,
    mut rx: mpsc::UnboundedReceiver<Vec<Item>>,
    // TODO: bench chunk
    tx: Sender<(Item, Score)>
) -> JoinHandle<()> {
    rt.spawn(async move {
        let sess = &mut this.write().await;
        let start = std::time::Instant::now();
        while let Some(chunk) = rx.recv().await {
            for item in chunk {}
        }
        let elapsed = std::time::Instant::now() - start;
        log::debug!("{:?}", elapsed);
    })
}
