pub mod item;
pub mod matcher;
pub mod session;
pub mod source;

pub use crate::{
    item::{Item, MaybeUtf8},
    session::{Session, Vars}
};
pub use tokio::sync::RwLock;

use crate::source::SourceRegistry;
use serde::{Deserialize, Serialize};
use std::{any::Any, collections::VecDeque, sync::Arc};
use tokio::runtime::Handle;

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
    sessions: VecDeque<(SessionId, Arc<Session>)>
}

impl State {
    pub fn new_shared() -> Shared<Self> {
        let this = Self {
            last_id: SessionId(0),
            sessions: VecDeque::new()
        };
        Arc::new(RwLock::new(this))
    }

    pub async fn start_session<'a, D, S>(
        &mut self,
        rt: AsyncRt,
        source: Arc<S>,
        senario: Senario<D, D>
    ) -> Result<(SessionId, &Arc<Session>), Error>
    where
        D: serde::de::Deserializer<'a>,
        S: SourceRegistry<'a, D> + 'static + Send + Sync,
        <D as serde::de::Deserializer<'a>>::Error: Send + Sync + 'static
    {
        let Senario {
            linearf: s_linearf,
            source: s_source,
            matcher: s_matcher
        } = senario;
        let s_linearf = Arc::new(s_linearf);
        let source_params = source
            .parse(&s_linearf.source, s_source)?
            .ok_or_else(|| format!("source \"{}\" is not found", &s_linearf.source))?;
        let sess = match self
            .reuse(&source, &s_linearf.source, (&s_linearf, &source_params))
            .await
        {
            Some((_, s)) => s,
            None => Session::start(rt, s_linearf, source_params, &source).await
        };
        let id = self.next_id();
        self.sessions.push_back((id, sess));
        Ok((id, &self.sessions[self.sessions.len() - 1].1))
    }

    async fn reuse<'a, D, S>(
        &mut self,
        source: &Arc<S>,
        name: &str,
        senario: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>)
    ) -> Option<(SessionId, Arc<Session>)>
    where
        D: serde::de::Deserializer<'a>,
        S: SourceRegistry<'a, D> + 'static + Send + Sync
    {
        for (id, sess) in self.sessions.iter().rev() {
            if sess.vars().source != name {
                continue;
            }
            if source
                .reusable(name, (&sess.vars(), &sess.source_params()), senario)
                .await
            {
                return Some((*id, sess.clone()));
            }
        }
        None
    }

    fn next_id(&mut self) -> SessionId {
        self.last_id = SessionId(self.last_id.0 + 1);
        self.last_id
    }
}

pub trait New {
    fn new(_state: &Shared<State>) -> Self
    where
        Self: Sized;
}

pub struct Senario<S, M> {
    pub linearf: Vars,
    pub source: S,
    pub matcher: M
}
