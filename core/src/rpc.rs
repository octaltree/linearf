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
    let mut connection = Connection::new();
    let state: Arc<RwLock<State>> = Arc::default();
    loop {
        let msg = match connection.next() {
            Err(ReadError::Closed) => break,
            Err(ReadError::Error(e)) => return Err(e),
            Ok(None) => continue,
            Ok(Some(msg)) => msg
        };
        match msg.body {
            MsgDeBody::Source(req) => source(&mut connection, &state, req).await?,
            MsgDeBody::Query(req) => {
                tokio::spawn(async {
                    let x = req;
                });
            }
            MsgDeBody::Preload(_resp) => ()
        }
    }
    Ok(())
}

async fn source(
    connection: &mut Connection,
    state: &Arc<RwLock<State>>,
    req: SourceRequest
) -> anyhow::Result<()> {
    // let mut source = crate::import::named_source(req.name);
    let items = vec![];
    preload(connection, &state, req.session, &items).await?;
    Ok(())
}

async fn preload(
    connection: &mut Connection,
    state: &Arc<RwLock<State>>,
    session: SessionId,
    items: &[Item<'_>]
) -> anyhow::Result<()> {
    let st = Arc::downgrade(state);
    let indexes: Vec<usize> = items.iter().map(|item| item.idx).collect();
    connection
        .request(
            MsgSerBody::Preload(PreloadRequest {
                session,
                items: &items
            }),
            move |body| {
                if let MsgDeBody::Preload(Ok(_)) = body {
                    let st = st;
                    let indexes = indexes;
                }
            }
        )
        .await?;
    Ok(())
}

struct Connection {
    reader: Reader,
    writer: Writer,
    id: MessageId,
    callbacks: HashMap<MessageId, Box<dyn FnOnce(&MsgDeBody)>>
}

impl Connection {
    fn new() -> Self {
        Self {
            reader: Reader::new(),
            writer: Writer::new(),
            id: MessageId(1),
            callbacks: HashMap::new()
        }
    }

    async fn request(
        &mut self,
        body: MsgSerBody<'_>,
        f: impl FnOnce(&MsgDeBody) + 'static
    ) -> anyhow::Result<()> {
        self.callbacks.insert(self.id, Box::new(f));
        let msg = MsgSer {
            id: Some(self.id),
            body
        };
        self.writer.send(&msg).await?;
        self.id = self.id.next();
        Ok(())
    }

    fn next(&mut self) -> Result<Option<MsgDe>, ReadError> {
        let result: Result<Option<MsgDe>, _> = self.reader.read();
        if let Ok(Some(msg)) = &result {
            if let Some(callback) = msg.id.as_ref().and_then(|i| self.callbacks.remove(i)) {
                callback(&msg.body);
            }
        }
        result
    }
}

#[derive(Debug, Default)]
struct State {
    session_history: VecDeque<SessionId>,
    sessions: HashMap<SessionId, Session>
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
    pub session: SessionId,
    pub name: String
}
#[derive(Debug, Clone, Serialize)]
pub struct SourceResponse {}
#[derive(Debug, Clone, Serialize)]
pub enum SourceError {
    SourceNotFound(String)
}

#[derive(Debug, Clone, Deserialize)]
pub struct QueryRequest {
    pub session: SessionId,
    pub query: String
}
#[derive(Debug, Clone, Serialize)]
pub struct QueryResponse {}
#[derive(Debug, Clone, Serialize)]
pub struct QueryError {}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct PreloadRequest<'a> {
    pub session: SessionId,
    pub items: &'a [Item<'a>]
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