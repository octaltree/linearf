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

#[derive(Default)]
pub struct State {
    last_id: SessionId,
    sessions: VecDeque<(SessionId, Shared<Session>)>
}

impl State {
    pub async fn new_shared() -> Shared<Self> {
        let this = Self::default();
        Arc::new(RwLock::new(this))
    }

    pub fn stert_session(
        &mut self,
        rt: AsyncRt,
        senario: (Arc<Vars>, Arc<dyn Any>, Arc<dyn Any>)
    ) -> Result<(SessionId, &Shared<Session>), Error> {
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

pub trait SourceRegistry {
    fn parse<'de, D>(
        &self,
        name: &str,
        deserializer: D
    ) -> Result<Arc<dyn std::any::Any>, D::Error>
    where
        D: serde::de::Deserializer<'de>;
}

pub trait MatcherRegistry {
    fn parse<'de, D>(
        &self,
        name: &str,
        deserializer: D
    ) -> Result<Arc<dyn std::any::Any>, D::Error>
    where
        D: serde::de::Deserializer<'de>;
}
