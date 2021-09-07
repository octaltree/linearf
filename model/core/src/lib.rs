#![feature(async_stream)]
#[macro_use]
extern crate serde;
#[macro_use]
extern crate async_trait;

pub mod flow;
pub mod session;
pub mod source;

pub(crate) mod import;
mod tmp;

// 適切な構造で持つ
// resume
// やりとり
// 1. vim-rust ソース開始リクエスト 状態を変更する
// 2. vim-rust クエリとともに範囲取得 vim側で一定時間ごとにカーソルから近い範囲と件数を取得する
// rust-vim アイテムを先に送る 文字列を先に送っておけばインデックスでやりとりできて速いかもしれない要検証

use crate::source::Source;
pub use crate::{flow::Flow, session::Session};
use serde_json::{Map, Value};
use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
    sync::Arc
};
use tokio::{runtime::Handle, sync::RwLock};

pub type AsyncRt = Handle;
pub type Shared<T> = Arc<RwLock<T>>;

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
    ) -> (i32, &Shared<Session>) {
        // TODO: re-cycle session if a flow of older session is same
        let id = self.next_session_id();
        let sess = Session::start(rt, flow).await;
        self.sessions.push_back((id, sess));
        (id, &self.sessions[self.sessions.len() - 1].1)
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StringBytes {
    String(String),
    Bytes(Vec<u8>)
}

// TODO: userdata
#[derive(Debug)]
pub struct Item {
    pub idx: usize,
    pub value: StringBytes,
    pub r#type: &'static str,
    pub view: Option<String>,
    pub view_for_matcing: Option<String>,
    /// To check mathcing query for dynamic source
    // TODO
    pub query: Option<Arc<String>>
}

impl Item {
    pub fn view(&self) -> Cow<'_, str> {
        let opt = self.view.as_deref().map(Cow::Borrowed);
        opt.unwrap_or_else(|| match &self.value {
            StringBytes::String(s) => Cow::Borrowed(s),
            StringBytes::Bytes(b) => match String::from_utf8_lossy(b) {
                Cow::Owned(s) => Cow::Owned(s),
                Cow::Borrowed(s) => Cow::Borrowed(s)
            }
        })
    }

    pub fn view_for_matcing(&self) -> Cow<'_, str> {
        let opt = self.view_for_matcing.as_deref().map(Cow::Borrowed);
        opt.unwrap_or_else(|| self.view())
    }
}

#[async_trait]
pub trait Matcher {
    type Score;
    fn name() -> &'static str;
    async fn start(&mut self, query: &str, option: Map<String, Value>);
    async fn score(&mut self, item: &Item) -> Self::Score;
}

/// Items will be displayed in descending order of its score.
/// No guarantee of order when it is equal. You should use idx to make it less equal.
pub trait Score: PartialEq + Eq + PartialOrd + Ord + Clone {
    /// If true, the item will not be displayed.
    fn is_excluded(&self) -> bool;
}

#[async_trait]
pub trait Converter {}
