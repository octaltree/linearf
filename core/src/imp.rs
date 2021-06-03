use crate::{Item, Source};
use std::{borrow::Cow, path::PathBuf};

#[derive(Default)]
struct FileRec {}

#[derive(Default)]
struct Grep {}

// TODO: file name and content encoding. json can only encode utf8 or [u8] or base64.

// impl Source for FileRec {
//    type Item;

//    fn name() -> &'static str {
//        todo!()
//    }

//    async fn start(&mut self, option: serde_json::Map<String, serde_json::Value>) {
//        todo!()
//    }

//    async fn item_stream<S>(&mut self) -> S
//    where
//        S: std::stream::Stream<Item = Self::Item> {
//        todo!()
//    }
//}

// impl Source for Grep {
//    type Item;

//    fn name() -> &'static str {
//        todo!()
//    }

//    async fn start(&mut self, option: serde_json::Map<String, serde_json::Value>) {
//        todo!()
//    }

//    async fn item_stream<S>(&mut self) -> S
//    where
//        S: std::stream::Stream<Item = Self::Item> {
//        todo!()
//    }
//}
