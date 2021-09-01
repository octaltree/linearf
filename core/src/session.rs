use crate::Item;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Weak};
use tokio::{
    runtime::Handle,
    sync::{mpsc, RwLock}
};

/// State being calculated based on flow
#[derive(Debug)]
pub struct Session {
    flow: Arc<Flow>,
    query: Option<Arc<String>>,
    items: Vec<Item>
}

/// Setting sources and matches
/// Cache may be used when equal
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct Flow {}

impl Session {
    pub async fn start(rt: Handle, flow: Arc<Flow>) -> Arc<RwLock<Self>> {
        let this = Self {
            flow,
            query: None,
            items: Vec::new()
        };
        let shared = Arc::new(RwLock::new(this));
        Session::main(rt, Arc::downgrade(&shared));
        shared
    }

    fn main(rt: Handle, this: Weak<RwLock<Session>>) {
        // let (tx1, rx1) = mpsc::unbounded_channel();
        // rt.spawn(Session::source(tx, this));
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
