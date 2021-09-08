#![feature(async_stream)]
#[macro_use]
extern crate serde;
#[macro_use]
extern crate async_trait;

pub mod flow;
pub mod matcher;
pub mod session;
pub mod source;

pub(crate) mod import;

pub use crate::{flow::Flow, matcher::Score, session::Session};
pub use tokio::sync::RwLock;

use crate::source::Source;
use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
    ffi::OsString,
    sync::Arc
};
use tokio::runtime::Handle;

pub type AsyncRt = Handle;
pub type Shared<T> = Arc<RwLock<T>>;

#[derive(Debug, thiserror::Error)]
enum Error {}

#[derive(Default)]
pub struct State {
    sessions: VecDeque<(i32, Shared<Session>)>,
    flows: HashMap<String, Arc<Flow>>,
    base_flow: Flow,
    sources: HashMap<String, Source>
}

impl State {
    pub async fn new_shared() -> Shared<Self> {
        let this = Self::default();
        Arc::new(RwLock::new(this))
    }

    pub async fn register_source<N: Into<String>>(state: &Shared<State>, name: N, source: Source) {
        let x = &mut state.write().await;
        x.sources.insert(name.into(), source);
    }

    pub async fn start_session<'a>(
        &'a mut self,
        rt: Handle,
        flow: Arc<Flow>
    ) -> Result<(i32, &Shared<Session>), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: re-cycle session if a flow of older session is same
        let id = self.next_session_id();
        let source = self
            .sources
            .get(&flow.source)
            .ok_or_else(|| -> Box<dyn std::error::Error + Send + Sync> {
                format!("source {} not found", &flow.source).into()
            })?
            .clone();
        let sess = Session::start(rt, flow, source).await;
        self.sessions.push_back((id, sess));
        Ok((id, &self.sessions[self.sessions.len() - 1].1))
    }

    pub fn close_session(&mut self, session: i32) {
        if let Some(idx) = self
            .sessions
            .iter()
            .enumerate()
            .map(|(idx, (id, _))| (idx, id))
            .find(|(_, &id)| id == session)
            .map(|(idx, _)| idx)
        {
            self.sessions.remove(idx);
        }
    }

    fn next_session_id(&self) -> i32 {
        // WARNING: 0 is indistinguishable from null in vim.
        // Keep at least on session
        if self.sessions.is_empty() {
            1
        } else {
            let last = self.sessions[self.sessions.len() - 1].0;
            last + 1
        }
    }

    pub fn session(&self, id: i32) -> Option<&Shared<Session>> {
        let mut rev = self.sessions.iter().rev();
        rev.find(|s| s.0 == id).map(|(_, s)| s)
    }

    pub fn sessions(&self) -> &VecDeque<(i32, Shared<Session>)> { &self.sessions }

    pub fn flows(&self) -> &HashMap<String, Arc<Flow>> { &self.flows }

    pub fn base_flow(&self) -> &Flow { &self.base_flow }

    pub fn source_names(&self) -> Vec<&str> {
        self.sources.iter().map(|(k, _)| -> &str { k }).collect()
    }
}

pub trait New {
    fn new(_state: &Shared<State>, _rt: &AsyncRt) -> Self
    where
        Self: Sized;
}

// TODO: userdata
#[derive(Debug)]
pub struct Item {
    /// id must not be 0
    pub id: u32,
    pub r#type: &'static str,
    pub value: MaybeUtf8,
    pub view: Option<String>,
    pub view_for_matcing: Option<String>
}

impl Item {
    pub fn new(id: u32, r#type: &'static str, value: MaybeUtf8) -> Self {
        Self {
            id,
            r#type,
            value,
            view: None,
            view_for_matcing: None
        }
    }
}

// TODO: into CStr
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaybeUtf8 {
    Utf8(String),
    Os(OsString),
    Bytes(Vec<u8>)
}

impl Item {
    #[inline]
    pub fn view(&self) -> Cow<'_, str> {
        let opt = self.view.as_deref().map(Cow::Borrowed);
        opt.unwrap_or_else(|| match &self.value {
            MaybeUtf8::Utf8(s) => Cow::Borrowed(s),
            MaybeUtf8::Os(s) => match s.to_string_lossy() {
                Cow::Owned(s) => Cow::Owned(s),
                Cow::Borrowed(s) => Cow::Borrowed(s)
            },
            MaybeUtf8::Bytes(b) => match String::from_utf8_lossy(b) {
                Cow::Owned(s) => Cow::Owned(s),
                Cow::Borrowed(s) => Cow::Borrowed(s)
            }
        })
    }

    #[inline]
    pub fn view_for_matcing(&self) -> Cow<'_, str> {
        let opt = self.view_for_matcing.as_deref().map(Cow::Borrowed);
        opt.unwrap_or_else(|| self.view())
    }
}
