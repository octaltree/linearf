#![feature(async_stream)]
#[macro_use]
extern crate serde;

pub mod imp;
pub(crate) mod import;
// pub mod rpc;

// rpc
// 状態管理
// ソース rustとvim script両方
// Match score
// 適切な構造で持つ
// resume

// やりとり
// 1. vim-rust ソース開始リクエスト 状態を変更する
// 2. vim-rust クエリとともに範囲取得 vim側で一定時間ごとにカーソルから近い範囲と件数を取得する
// rust-vim アイテムを先に送る 文字列を先に送っておけばインデックスでやりとりできて速いかもしれない要検証

use async_trait::async_trait;
use serde_json::{Map, Value};
use std::{
    collections::{HashMap, VecDeque},
    stream::Stream,
    sync::Arc
};
use tokio::sync::RwLock;

/// NOTE: Source and Match have the potential to have cache, so make them live longer.

#[async_trait]
pub trait Source: Default {
    fn name() -> &'static str;
    async fn start(&mut self, option: Map<String, Value>);
    async fn item_stream<S>(&mut self) -> S
    where
        S: Stream<Item = Item>;
}

pub struct Item {
    idx: usize,
    value: String,
    r#type: &'static str,
    view: Option<String>,
    view_for_matcing: Option<String>
}

impl Item {
    fn view(&self) -> &str { self.view.as_deref().unwrap_or(&self.value) }

    fn view_for_matcing(&self) -> &str {
        self.view_for_matcing
            .as_deref()
            .unwrap_or_else(|| self.view())
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

#[derive(Debug)]
pub struct Flow {}

#[derive(Debug)]
pub struct Session {
    id: i32,
    flow: Flow
}

pub async fn start() {}

#[derive(Debug, Default)]
pub struct State {
    shutdown: bool,
    id: i32,
    sessions: VecDeque<RwLock<Session>>,
    flow: HashMap<String, Flow>
}

impl State {
    pub fn new() -> Arc<RwLock<Self>> {
        let this = Self::default();
        Arc::new(RwLock::new(this))
    }
}

struct Linearf;

impl Linearf {
    fn new() -> Linearf { todo!() }

    async fn start(flow: Flow) -> i32 { todo!() }

    async fn start_by_name<S: AsRef<str>>(flow: S) -> i32 { todo!() }

    async fn shutdown() { todo!() }
}

impl Session {
    async fn count(&self) -> usize { todo!() }

    async fn items(&self, start: usize, stop: usize) -> &[Item] { todo!() }
}

mod tmp {
    use super::*;
    use std::cmp::Ordering;

    /// Bigger f64 is higher priority. If the order is not determined, bigger idx is lower priority.
    /// If f64 is NaN, it will be excluded.
    #[derive(Clone, Copy)]
    struct F64Ord {
        x: f64,
        idx: usize
    }

    impl Score for F64Ord {
        fn is_excluded(&self) -> bool { self.x.is_nan() }
    }

    impl PartialEq for F64Ord {
        fn eq(&self, other: &Self) -> bool { self.x.eq(&other.x) && self.idx.eq(&other.idx) }
    }

    impl Eq for F64Ord {}

    impl PartialOrd for F64Ord {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
    }

    impl Ord for F64Ord {
        fn cmp(&self, other: &Self) -> Ordering {
            match (self.x <= other.x, self.x >= other.x) {
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                _ => match self.idx.cmp(&other.idx) {
                    Ordering::Less => Ordering::Greater,
                    Ordering::Equal => Ordering::Equal,
                    Ordering::Greater => Ordering::Less
                }
            }
        }
    }

    /// Bigger u16 is higher priority. If the order is not determined, bigger idx is lower priority.
    /// If u16 is 0, it will be excluded.
    #[derive(Clone, Copy)]
    struct U16Ord {
        x: u16,
        idx: usize
    }

    impl Score for U16Ord {
        fn is_excluded(&self) -> bool { self.x == 0 }
    }

    impl PartialEq for U16Ord {
        fn eq(&self, other: &Self) -> bool { self.x.eq(&other.x) && self.idx.eq(&other.idx) }
    }

    impl Eq for U16Ord {}

    impl PartialOrd for U16Ord {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
    }

    impl Ord for U16Ord {
        fn cmp(&self, other: &Self) -> Ordering {
            match (self.x <= other.x, self.x >= other.x) {
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                _ => match self.idx.cmp(&other.idx) {
                    Ordering::Less => Ordering::Greater,
                    Ordering::Equal => Ordering::Equal,
                    Ordering::Greater => Ordering::Less
                }
            }
        }
    }
}
