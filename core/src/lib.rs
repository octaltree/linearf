#![feature(async_stream)]
#[macro_use]
extern crate serde;

pub mod imp;
pub(crate) mod import;
pub mod rpc;

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
use std::stream::Stream;

/// NOTE: Source and Match have the potential to have cache, so make them live longer.

#[async_trait]
pub trait Source: Default {
    type Item: Item;
    fn name() -> &'static str;
    async fn start(&mut self, option: Map<String, Value>);
    async fn item_stream<S>(&mut self) -> S
    where
        S: Stream<Item = Self::Item>;
}

pub trait Item {
    fn idx(&self) -> usize;
    fn value(&self) -> &str;
    fn value_type(&self) -> &'static str;
    /// A single line of text to be displayed in the candidate list
    fn view(&self) -> &str;
    /// This is used for matching. It should be substring of view.
    fn view_for_matcing(&self) -> &str { self.view() }
}

#[async_trait]
pub trait Match: Default {
    type Score;
    fn name() -> &'static str;
    async fn start(&mut self, query: &str, option: Map<String, Value>);
    async fn score<I>(&mut self, item: &I) -> Self::Score
    where
        I: Item;
}

/// Items will be displayed in descending order of its score.
/// No guarantee of order when it is equal. You should use idx to make it less equal.
pub trait Score: PartialEq + Eq + PartialOrd + Ord + Clone {
    /// If true, the item will not be displayed.
    fn is_excluded(&self) -> bool;
}

pub async fn start() {
    tokio::spawn(async {
        log::debug!("foo");
    });
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
