#![feature(async_stream)]
#[macro_use]
extern crate serde;
#[macro_use]
extern crate async_trait;

pub mod imp;
pub(crate) mod import;
pub mod session;
pub mod source;
mod tmp;

// 状態管理
// ソース rustとvim script両方
// Match score
// 適切な構造で持つ
// resume

// やりとり
// 1. vim-rust ソース開始リクエスト 状態を変更する
// 2. vim-rust クエリとともに範囲取得 vim側で一定時間ごとにカーソルから近い範囲と件数を取得する
// rust-vim アイテムを先に送る 文字列を先に送っておけばインデックスでやりとりできて速いかもしれない要検証

pub use crate::{
    session::{Flow, Session},
    source::Source
};
use serde_json::{Map, Value};
use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
    sync::Arc
};
use tokio::{runtime::Handle, sync::RwLock};

#[derive(Debug, Default)]
pub struct State {
    sessions: VecDeque<(i32, Arc<RwLock<Session>>)>,
    flows: HashMap<String, Arc<Flow>>,
    sources: HashMap<String, Arc<dyn Source>>
}

impl State {
    pub async fn new_shared() -> Arc<RwLock<Self>> {
        let this = Self::default();
        let a = Arc::new(RwLock::new(this));
        // let x = a.write().await;
        a
    }

    pub async fn start_session<'a>(
        &'a mut self,
        rt: Handle,
        flow: &str
    ) -> Option<(i32, &Arc<RwLock<Session>>)> {
        // TODO: re-cycle session if a flow of older session is same
        let id = self.next_session_id();
        let sess = Session::start(rt, Arc::clone(self.flows.get(flow)?)).await;
        self.sessions.push_back((id, sess));
        Some((id, &self.sessions[self.sessions.len() - 1].1))
    }

    fn next_session_id(&self) -> i32 {
        if self.sessions.is_empty() {
            1
        } else {
            let last = self.sessions[self.sessions.len() - 1].0;
            last + 1
        }
    }

    pub async fn session(&self, id: i32) -> Option<&Arc<RwLock<Session>>> {
        let mut rev = self.sessions.iter().rev();
        rev.find(|s| s.0 == id).map(|(_, s)| s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum StringBytes {
    String(String),
    Bytes(Vec<u8>)
}

// TODO: userdata
#[derive(Debug)]
pub struct Item {
    idx: usize,
    value: StringBytes,
    r#type: &'static str,
    view: Option<String>,
    view_for_matcing: Option<String>,
    /// To check mathcing query for dynamic source
    query: Option<Arc<String>>
}

impl Item {
    fn view(&self) -> Cow<'_, str> {
        let opt = self.view.as_deref().map(Cow::Borrowed);
        opt.unwrap_or_else(|| match &self.value {
            StringBytes::String(s) => Cow::Borrowed(s),
            StringBytes::Bytes(b) => match String::from_utf8_lossy(b) {
                Cow::Owned(s) => Cow::Owned(s),
                Cow::Borrowed(s) => Cow::Borrowed(s)
            }
        })
    }

    fn view_for_matcing(&self) -> Cow<'_, str> {
        let opt = self.view_for_matcing.as_deref().map(Cow::Borrowed);
        opt.unwrap_or_else(|| self.view())
    }
}

#[async_trait]
pub trait Match: Default {
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
