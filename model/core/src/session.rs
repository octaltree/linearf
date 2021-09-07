use crate::{source::Source, Flow, Item, Shared};
use std::sync::{Arc, Weak};
use tokio::{
    runtime::Handle,
    sync::{mpsc, RwLock}
};

pub type Sender<T> = mpsc::UnboundedSender<T>;

/// State being calculated based on flow
pub struct Session {
    // TODO: items for each query
    flow: Arc<Flow>,
    source: Source,
    query: Option<Arc<String>>,
    items: Vec<Item>
}

impl Session {
    pub(crate) fn flow(&self) -> &Arc<Flow> { &self.flow }
}

impl Session {
    pub async fn start(rt: Handle, flow: Arc<Flow>, source: Source) -> Shared<Self> {
        let this = Self {
            flow,
            source,
            query: None,
            items: Vec::new()
        };
        let shared = Arc::new(RwLock::new(this));
        Session::main(rt, shared.clone()).await;
        shared
    }

    async fn main(rt: Handle, this: Arc<RwLock<Session>>) {
        // source
        // score
        // sort

        // let (tx1, rx1) = mpsc::unbounded_channel();
        // TODO: reusable
        // let stream = Source::start(this.flow()).await;
        // rt.spawn(Source::start(this.flow()))
        // rt.spawn(Session::start(tx, this));
        // let (tx2, rx2) = mpsc::unbounded_channel();
        // loop {
        //    this.upgrade()?;
        //}
    }

    async fn source(
        tx: mpsc::UnboundedSender<Arc<Item>>,
        this: Weak<RwLock<Session>>
    ) -> Option<()> {
        None
    }

    async fn convert(
        tx: mpsc::UnboundedSender<Arc<Item>>,
        this: Weak<RwLock<Session>>
    ) -> Option<()> {
        None
    }

    pub fn count(&self) -> usize { self.items.len() }

    pub fn items(&self, start: usize, stop: usize) -> Option<&[Item]> {
        let l = self.items.len();
        if start <= l && stop <= l {
            Some(&self.items[start..stop])
        } else {
            None
        }
    }

    pub fn change_query<S: Into<String>>(&mut self, s: S) {
        let arc = Arc::new(s.into());
        self.query = Some(arc);
        todo!()
    }
}
