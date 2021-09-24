pub mod item;
pub mod matcher;
pub mod session;
pub mod source;

pub use crate::{
    item::{Item, MaybeUtf8},
    session::{Session, Vars}
};
pub use async_trait::async_trait;
pub use tokio::sync::RwLock;

use crate::{matcher::MatcherRegistry, source::SourceRegistry};
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

    pub async fn start_session<'a, D, S, M>(
        &mut self,
        rt: AsyncRt,
        source: Arc<S>,
        matcher: Arc<M>,
        senario: Senario<Vars, D>
    ) -> Result<(SessionId, &Arc<Session>), Error>
    where
        D: serde::de::Deserializer<'a>,
        S: SourceRegistry<'a, D> + 'static + Send + Sync,
        M: MatcherRegistry<'a, D> + 'static + Send + Sync,
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
        let matcher_params = matcher
            .parse(&s_linearf.matcher, s_matcher)?
            .ok_or_else(|| format!("matcher \"{}\" is not found", &s_linearf.matcher))?;
        let reusable = self
            .reuse(
                &source,
                &matcher,
                &s_linearf.source,
                &s_linearf.matcher,
                Senario {
                    linearf: &s_linearf,
                    source: &source_params,
                    matcher: &matcher_params
                }
            )
            .await;
        let (id, sess) = match reusable {
            Some((id, s)) => {
                self.remove_session(id);
                (id, s)
            }
            None => {
                let senario = Senario {
                    linearf: s_linearf,
                    source: source_params,
                    matcher: matcher_params
                };
                let sess = Session::start(rt, senario, &source, &matcher).await;
                let id = self.next_id();
                (id, sess)
            }
        };
        self.sessions.push_back((id, sess));
        Ok((id, &self.sessions[self.sessions.len() - 1].1))
    }

    async fn reuse<'a, D, S, M>(
        &mut self,
        source: &Arc<S>,
        matcher: &Arc<M>,
        source_name: &str,
        matcher_name: &str,
        senario: Senario<&Arc<Vars>, &Arc<dyn Any + Send + Sync>>
    ) -> Option<(SessionId, Arc<Session>)>
    where
        D: serde::de::Deserializer<'a>,
        S: SourceRegistry<'a, D> + 'static + Send + Sync,
        M: MatcherRegistry<'a, D> + 'static + Send + Sync
    {
        for (id, sess) in self.sessions.iter().rev() {
            if sess.vars().source != source_name || sess.vars().matcher != matcher_name {
                continue;
            }
            if source
                .reusable(
                    source_name,
                    (sess.vars(), sess.source_params()),
                    (senario.linearf, senario.source)
                )
                .await
                && matcher
                    .reusable(
                        matcher_name,
                        (sess.vars(), sess.matcher_params()),
                        (senario.linearf, senario.matcher)
                    )
                    .await
            {
                return Some((*id, sess.clone()));
            }
        }
        None
    }

    pub fn remove_session(&mut self, session: SessionId) {
        if let Some(idx) = self
            .sessions
            .iter()
            .enumerate()
            .map(|(idx, (id, _))| (idx, id))
            .find(|(_, &id)| id == session)
            .map(|(idx, _)| idx)
        {
            self.sessions.remove(idx);
        }
    }

    fn next_id(&mut self) -> SessionId {
        self.last_id = SessionId(self.last_id.0 + 1);
        self.last_id
    }

    pub async fn tick<'a, D, S, M>(
        &mut self,
        rt: AsyncRt,
        source: Arc<S>,
        matcher: Arc<M>,
        id: SessionId,
        senario: Senario<Vars, D>
    ) -> Result<FlowId, Error>
    where
        D: serde::de::Deserializer<'a>,
        S: SourceRegistry<'a, D> + 'static + Send + Sync,
        M: MatcherRegistry<'a, D> + 'static + Send + Sync,
        <D as serde::de::Deserializer<'a>>::Error: Send + Sync + 'static
    {
        let sess = self
            .session(id)
            .ok_or_else(|| format!("session {} is not found", id.0))?;
        Ok(FlowId(42))
    }

    pub fn session(&self, id: SessionId) -> Option<&Arc<Session>> {
        let mut rev = self.sessions.iter().rev();
        rev.find(|s| s.0 == id).map(|(_, s)| s)
    }
}

pub trait New {
    fn new(_state: &Shared<State>) -> Self
    where
        Self: Sized;
}

#[derive(Clone)]
pub struct Senario<V, P> {
    pub linearf: V,
    pub source: P,
    pub matcher: P
}
