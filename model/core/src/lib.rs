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
    sessions: VecDeque<(SessionId, Session)>
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
    ) -> Result<(SessionId, FlowId), Error>
    where
        D: serde::de::Deserializer<'a>,
        S: SourceRegistry<'a, D> + 'static + Send + Sync,
        M: MatcherRegistry<'a, D> + 'static + Send + Sync,
        <D as serde::de::Deserializer<'a>>::Error: Send + Sync + 'static
    {
        let Senario {
            linearf: vars,
            source: source_params,
            matcher: matcher_params
        } = parse_params(&source, &matcher, senario)?;
        let reusable = self
            .reuse(
                &source,
                &matcher,
                Senario {
                    linearf: &vars,
                    source: &source_params,
                    matcher: &matcher_params
                }
            )
            .await;
        let (id, sess) = match reusable {
            Some((sid, fid)) => {
                // unwrap: reusable returns the id that exists
                let mut s = self.remove_session(sid).unwrap();
                s.resume_flow(fid).unwrap();
                (sid, s)
            }
            None => {
                let senario = Senario {
                    linearf: vars,
                    source: source_params,
                    matcher: matcher_params
                };
                let sess = Session::start(rt, &source, &matcher, senario).await;
                let id = self.next_id();
                (id, sess)
            }
        };
        self.sessions.push_back((id, sess));
        let (id, sess) = &self.sessions[self.sessions.len() - 1];
        let (fid, _) = sess.last_flow();
        Ok((*id, fid))
    }

    async fn reuse<'a, D, S, M>(
        &mut self,
        source: &Arc<S>,
        matcher: &Arc<M>,
        senario: Senario<&Arc<Vars>, &Arc<dyn Any + Send + Sync>>
    ) -> Option<(SessionId, FlowId)>
    where
        D: serde::de::Deserializer<'a>,
        S: SourceRegistry<'a, D> + 'static + Send + Sync,
        M: MatcherRegistry<'a, D> + 'static + Send + Sync
    {
        for (sid, sess) in self.sessions.iter().rev() {
            for (fid, flow) in sess.flows().iter().rev() {
                let vars = flow.vars();
                if vars.source != senario.linearf.source || vars.matcher != senario.linearf.matcher
                {
                    break;
                }
                if source
                    .reusable(
                        &senario.linearf.source,
                        (vars, flow.source_params()),
                        (senario.linearf, senario.source)
                    )
                    .await
                    && matcher
                        .reusable(
                            &senario.linearf.matcher,
                            (vars, flow.matcher_params()),
                            (senario.linearf, senario.matcher)
                        )
                        .await
                {
                    return Some((*sid, *fid));
                }
            }
        }
        None
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
            .session_mut(id)
            .ok_or_else(|| format!("session {} is not found", id.0))?;
        validate_senario(sess, &senario)?;
        let senario = parse_params(&source, &matcher, senario)?;
        sess.tick(rt, &source, &matcher, senario).await;
        Ok(sess.last_flow().0)
    }

    pub async fn resume(&mut self, id: SessionId) -> Result<FlowId, Error> {
        let s = self
            .remove_session(id)
            .ok_or_else(|| format!("session {:?} is not found", id))?;
        let fid = s.last_flow().0;
        self.sessions.push_back((id, s));
        Ok(fid)
    }

    pub fn remove_session(&mut self, session: SessionId) -> Option<Session> {
        if let Some(idx) = self
            .sessions
            .iter()
            .enumerate()
            .map(|(idx, (id, _))| (idx, id))
            .find(|(_, &id)| id == session)
            .map(|(idx, _)| idx)
        {
            self.sessions.remove(idx).map(|(_, s)| s)
        } else {
            None
        }
    }

    fn next_id(&mut self) -> SessionId {
        self.last_id = SessionId(self.last_id.0 + 1);
        self.last_id
    }

    pub fn session(&self, id: SessionId) -> Option<&Session> {
        let mut rev = self.sessions.iter().rev();
        rev.find(|s| s.0 == id).map(|(_, s)| s)
    }

    fn session_mut(&mut self, id: SessionId) -> Option<&mut Session> {
        let mut rev = self.sessions.iter_mut().rev();
        rev.find(|s| s.0 == id).map(|(_, s)| s)
    }
}

fn parse_params<'a, D, S, M>(
    source: &Arc<S>,
    matcher: &Arc<M>,
    senario: Senario<Vars, D>
) -> Result<Senario<Arc<Vars>, Arc<dyn Any + Send + Sync>>, Error>
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
    Ok(Senario {
        linearf: s_linearf,
        source: source_params,
        matcher: matcher_params
    })
}

fn validate_senario<D>(session: &Session, senario: &Senario<Vars, D>) -> Result<(), Error> {
    let flow = session.last_flow().1;
    let prev = flow.vars();
    if prev.source != senario.linearf.source {
        return Err(format!(
            r#"source "{}" != "{}""#,
            &prev.source, senario.linearf.source
        )
        .into());
    }
    if prev.matcher != senario.linearf.matcher {
        return Err(format!(
            r#"matcher "{}" != "{}""#,
            &prev.matcher, senario.linearf.matcher
        )
        .into());
    }
    Ok(())
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
