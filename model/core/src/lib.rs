pub mod item;
pub mod session;
pub mod source;

pub use crate::{item::Item, session::Session};

use crate::session::Vars;
use serde::{Deserialize, Serialize};
use std::{any::Any, collections::VecDeque, sync::Arc};
use tokio::{runtime::Handle, sync::RwLock};

pub type AsyncRt = Handle;
pub type Shared<T> = Arc<RwLock<T>>;
pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, Default,
)]
#[serde(transparent)]
pub struct SessionId(pub i32);
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, Default,
)]
#[serde(transparent)]
pub struct FlowId(pub i32);

pub struct State {
    last_id: SessionId,
    sessions: VecDeque<(SessionId, Shared<Session>)>
}

impl State {
    pub fn new_shared() -> Shared<Self> {
        let this = Self {
            last_id: SessionId(0),
            sessions: VecDeque::new()
        };
        Arc::new(RwLock::new(this))
    }

    pub fn start_session<'a, D, S>(
        &mut self,
        rt: AsyncRt,
        source: S,
        senario: (Arc<Vars>, D, D)
    ) -> Result<(SessionId, &Shared<Session>), Error>
    where
        D: serde::de::Deserializer<'a>,
        S: SourceRegistry<'a, D>
    {
        todo!()
    }
    //    let id = self.next_session_id();
    //    let source = self.source(&flow.source)?.clone();
    //    let matcher = self.matcher(&flow.matcher)?.clone();
    //    let sess = match self.reuse(&source) {
    //        Some((_, s)) => s,
    //        None => Session::start(rt, flow, source, matcher)
    //    };
    //    self.sessions.push_back((id, sess));
    //    Ok((id, &self.sessions[self.sessions.len() - 1].1))
    //}
}

pub trait SourceRegistry<'de, D>
where
    D: serde::de::Deserializer<'de>
{
    fn new(state: Shared<State>) -> Self
    where
        Self: Sized;

    fn parse(
        &self,
        name: &str,
        deserializer: D
    ) -> Result<Arc<dyn std::any::Any + Send + Sync>, D::Error>;
}
