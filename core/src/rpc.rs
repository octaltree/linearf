//! THIS IS NOT JSON-RPC 2.0.
//!
//! header has only a length, and the schema uses the power of serde.

mod transport;

use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc
};
use tokio::sync::RwLock;
use transport::{ReadError, Reader, Writer};

pub async fn run() -> anyhow::Result<()> {
    // let mut worker = Worker::new();
    // worker.run().await
    Ok(())
}

struct Connection {
    reader: Reader,
    writer: Writer,
    next_id: MessageId
}

impl Connection {
    fn new() -> Self {
        Self {
            reader: Reader::new(),
            writer: Writer::new(),
            next_id: MessageId(1)
        }
    }

    async fn request(&mut self, body: MsgSerBody<'_>) -> anyhow::Result<()> {
        // TODO: callback
        let msg = MsgSer {
            id: Some(self.next_id),
            body
        };
        self.writer.send(&msg).await?;
        self.next_id = self.next_id.next();
        Ok(())
    }

    fn next(&mut self) -> Result<Option<MsgDe>, ReadError> { self.reader.read() }
}

#[derive(Debug, Default)]
struct State {
    session_history: VecDeque<SessionId>,
    sessions: HashMap<SessionId, Session>
}

struct Worker {
    connection: Connection,
    state: Arc<RwLock<State>>
}

impl Worker {
    fn new() -> Self {
        Self {
            connection: Connection::new(),
            state: Arc::default()
        }
    }

    async fn run(&mut self) -> anyhow::Result<()> {
        loop {
            let msg = match self.connection.next() {
                Err(ReadError::Closed) => break,
                Err(ReadError::Error(e)) => return Err(e),
                Ok(None) => continue,
                Ok(Some(msg)) => msg
            };
            match msg.body {
                MsgDeBody::Source(req) => {}
                MsgDeBody::Query(req) => {
                    tokio::spawn(async {
                        let x = req;
                    });
                }
                MsgDeBody::Preload(resp) => {
                    tokio::spawn(async {});
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Session {
    id: SessionId,
    /// Whether or not the idx item has been preloaded
    preloaded: Vec<bool>,
    query: String,
    sorted: Sorted
}

#[derive(Debug)]
struct Sorted {}

macro_rules! id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Hash)]
        pub struct $name(i32);

        impl $name {
            fn next(&self) -> Self { $name(self.0 + 1) }
        }
    };
}
id! {MessageId}
// We'll call starting a source once a session. We use it in the resume.
id! {SessionId}

#[derive(Debug, Clone, Serialize)]
pub struct Item<'a> {
    idx: usize,
    value: &'a str,
    value_type: &'static str,
    view: &'a str
}

impl<'a, I> From<&'a I> for Item<'a>
where
    I: 'a + crate::Item
{
    fn from(i: &'a I) -> Self {
        Item {
            idx: i.idx(),
            value: i.value(),
            value_type: i.value_type(),
            view: i.view()
        }
    }
}

/// Message schema for rust to vim
#[derive(Debug, Clone, Copy, Serialize)]
pub struct MsgSer<'a> {
    id: Option<MessageId>,
    body: MsgSerBody<'a>
}

/// Message schema for vim to rust
#[derive(Debug, Deserialize)]
pub struct MsgDe {
    pub id: Option<MessageId>,
    pub body: MsgDeBody
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum MsgSerBody<'a> {
    Source(Result<&'a SourceResponse, &'a SourceError>),
    Query(Result<&'a QueryResponse, &'a QueryResponse>),
    Preload(PreloadRequest<'a>)
}

#[derive(Debug, Clone, Deserialize)]
pub enum MsgDeBody {
    Source(SourceRequest),
    Query(QueryRequest),
    Preload(Result<PreloadResponse, PreloadError>)
}

#[derive(Debug, Clone, Deserialize)]
pub struct SourceRequest {
    pub id: SessionId
}
#[derive(Debug, Clone, Serialize)]
pub struct SourceResponse {}
#[derive(Debug, Clone, Serialize)]
pub struct SourceError {}

#[derive(Debug, Clone, Deserialize)]
pub struct QueryRequest {
    pub id: SessionId,
    pub query: String
}
#[derive(Debug, Clone, Serialize)]
pub struct QueryResponse {}
#[derive(Debug, Clone, Serialize)]
pub struct QueryError {}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct PreloadRequest<'a> {
    pub id: SessionId,
    pub items: &'a [(usize, Item<'a>)]
}
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct PreloadResponse {}
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct PreloadError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_preload() {
        let x: Result<_, PreloadError> = serde_json::from_str(r#"{"Ok": {}}"#).unwrap();
        assert_eq!(x, Result::<_, PreloadError>::Ok(PreloadResponse {}));
        let x: Result<PreloadResponse, PreloadError> =
            serde_json::from_str(r#"{"Err": {}}"#).unwrap();
        assert_eq!(x, Result::<PreloadResponse, _>::Err(PreloadError {}));
    }
}
