mod flow;

use crate::{source::Reusable, AsyncRt, ConverterRegistry, MatcherRegistry, SourceRegistry, Vars};
pub use flow::Flow;
use flow::{Reuse, StartError};
use serde::{Deserialize, Serialize};
use std::{any::Any, collections::VecDeque, sync::Arc, time::Instant};
use tokio::sync::RwLock;

pub type Shared<T> = Arc<RwLock<T>>;

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

    pub fn start_flow<'a, D, S, M, C>(
        &mut self,
        rt: AsyncRt,
        source: &S,
        matcher: &M,
        converter: &C,
        request: StartFlow<D>
    ) -> Result<(SessionId, FlowId), Error>
    where
        D: serde::de::Deserializer<'a>,
        <D as serde::de::Deserializer<'a>>::Error: Send + Sync + 'static,
        S: SourceRegistry,
        M: MatcherRegistry,
        C: ConverterRegistry
    {
        let started = Instant::now();
        let (id, mut target) = match request.id {
            None => {
                self.last_id = SessionId(self.last_id.0 + 1);
                (self.last_id, Session::empty())
            }
            Some(id) => {
                let sess = self
                    .session(id)
                    .ok_or_else(|| format!("session {:?} is not found", id))?;
                validate_senario(sess, &request.senario)?;
                let sess = self.take_session(id).unwrap();
                (id, sess)
            }
        };
        let senario = parse_senario(source, matcher, request.senario)?;
        let reuse = match self.reusable((id, &target), source, matcher, senario.as_ref(), started) {
            Some(r) => Ok(r.map(|(_, _, flow)| flow)),
            None => Err(started)
        };
        let flow =
            Flow::start(rt, source, matcher, converter, reuse, senario).map_err(|e| match e {
                StartError::ConverterNotFound(n) => {
                    format!("converter {:?} is not found", n)
                }
            })?;
        let fid = target.push(flow);
        self.sessions.push_back((id, target));
        Ok((id, fid))
    }
}

pub struct StartFlow<D> {
    pub id: Option<SessionId>,
    pub senario: Senario<Vars, D>
}

#[derive(Clone, Copy)]
pub struct Senario<V, P> {
    pub linearf: V,
    pub source: P,
    pub matcher: P
}

impl<V, P> Senario<V, P> {
    fn as_ref(&self) -> Senario<&V, &P> {
        Senario {
            linearf: &self.linearf,
            source: &self.source,
            matcher: &self.matcher
        }
    }
}

type Error = Box<dyn std::error::Error + Send + Sync>;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(transparent)]
pub struct SessionId(pub i32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(transparent)]
pub struct FlowId(pub usize);

struct Session {
    flows: Vec<Flow>
}

impl Session {
    fn empty() -> Self { Self { flows: Vec::new() } }

    fn push(&mut self, flow: Flow) -> FlowId {
        self.flows.push(flow);
        FlowId(self.flows.len() - 1)
    }

    fn flows(&self) -> impl Iterator<Item = (FlowId, &Flow)> + DoubleEndedIterator {
        self.flows.iter().enumerate().map(|(i, f)| (FlowId(i), f))
    }

    fn flow(&self, id: FlowId) -> Option<&Flow> { self.flows.get(id.0) }
}

impl State {
    fn session(&self, id: SessionId) -> Option<&Session> {
        let mut rev = self.sessions.iter().rev();
        rev.find(|s| s.0 == id).map(|(_, s)| s)
    }

    fn take_session(&mut self, session: SessionId) -> Option<Session> {
        let idx = self
            .sessions
            .iter()
            .enumerate()
            .map(|(idx, (id, _))| (idx, id))
            .find(|(_, &id)| id == session)
            .map(|(idx, _)| idx)?;
        self.sessions.remove(idx).map(|(_, s)| s)
    }
}

/// Panic: if session has no flows
fn validate_senario<D>(session: &Session, senario: &Senario<Vars, D>) -> Result<(), Error> {
    let flow = session
        .flows()
        .next_back()
        .expect("Session must have one or more flows")
        .1;
    let prev = &flow.senario();
    if prev.sorted_vars.source != senario.linearf.source {
        return Err(format!(
            "source {:?} != {:?}",
            &prev.sorted_vars.source, senario.linearf.source
        )
        .into());
    }
    if prev.sorted_vars.source != senario.linearf.matcher {
        return Err(format!(
            "matcher {:?} != {:?}",
            &prev.sorted_vars.matcher, senario.linearf.matcher
        )
        .into());
    }
    if prev.sorted_vars.converters != senario.linearf.converters {
        return Err(format!(
            "converters {:?} != {:?}",
            &prev.sorted_vars.converters, senario.linearf.converters
        )
        .into());
    }
    Ok(())
}

fn parse_senario<'a, D, S, M>(
    source: &S,
    matcher: &M,
    senario: Senario<Vars, D>
) -> Result<Senario<Arc<Vars>, Arc<dyn Any + Send + Sync>>, Error>
where
    D: serde::de::Deserializer<'a>,
    <D as serde::de::Deserializer<'a>>::Error: Send + Sync + 'static,
    S: SourceRegistry,
    M: MatcherRegistry
{
    let Senario {
        linearf: s_linearf,
        source: s_source,
        matcher: s_matcher
    } = senario;
    let s_linearf = Arc::new(s_linearf);
    let source_params = source
        .parse(&s_linearf.source, s_source)
        .ok_or_else(|| format!("source {:?} is not found", &s_linearf.source))??;
    let matcher_params = matcher
        .parse(&s_linearf.matcher, s_matcher)
        .ok_or_else(|| format!("matcher {:?} is not found", &s_linearf.matcher))??;
    Ok(Senario {
        linearf: s_linearf,
        source: source_params,
        matcher: matcher_params
    })
}

impl State {
    /// Reusable::Same&&Reuse::Matcher >
    /// Reuseable::Cache&&Reuse::Matcher && use it offered by vars >
    /// Reusable::Same&&Reuse::Source >
    /// Reusable::Cache&&Reuse::Source && use it offered by vars >
    /// None
    fn reusable<'a, S, M>(
        &'a self,
        target: (SessionId, &'a Session),
        source: &S,
        matcher: &M,
        senario: Senario<&Arc<Vars>, &Arc<dyn Any + Send + Sync>>,
        started: Instant
    ) -> Option<Reuse<(SessionId, FlowId, &'a Flow)>>
    where
        S: SourceRegistry,
        M: MatcherRegistry
    {
        let use_cache = |flow: &Flow| -> bool {
            (started - flow.at()).as_secs() < senario.linearf.cache_sec.into()
        };
        let source_reusable = |flow: &Flow| {
            source.reusable(
                &senario.linearf.source,
                (flow.senario().stream_vars, flow.senario().source),
                (senario.linearf, senario.source)
            )
        };
        let matcher_reusable = |flow: &Flow| match (
            source_reusable(flow),
            matcher.reusable(
                &senario.linearf.matcher,
                (flow.senario().sorted_vars, flow.senario().matcher),
                (senario.linearf, senario.source)
            )
        ) {
            (Reusable::Same, Reusable::Same) => Reusable::Same,
            (Reusable::Cache, Reusable::Same) => Reusable::Cache,
            (Reusable::Same, Reusable::Cache) => Reusable::Cache,
            (Reusable::Cache, Reusable::Cache) => Reusable::Cache,
            _ => Reusable::None
        };
        let matcher_same = |flow: &Flow| -> Option<Reuse<()>> {
            let go = matcher_reusable(flow) == Reusable::Same;
            go.then(|| Reuse::Matcher(()))
        };
        let matcher_cache = |flow: &Flow| -> Option<Reuse<()>> {
            let go = use_cache(flow) && matcher_reusable(flow) == Reusable::Cache;
            go.then(|| Reuse::Matcher(()))
        };
        let source_same = |flow: &Flow| -> Option<Reuse<()>> {
            let go = source_reusable(flow) == Reusable::Same;
            go.then(|| Reuse::Source(()))
        };
        let source_cache = |flow: &Flow| -> Option<Reuse<()>> {
            let go = use_cache(flow) && source_reusable(flow) == Reusable::Cache;
            go.then(|| Reuse::Source(()))
        };
        let traversal = Traversal {
            state: self,
            target,
            senario
        };
        // should I find newest cache?
        traversal.find(matcher_same)?;
        traversal.find(matcher_cache)?;
        traversal.find(source_same)?;
        traversal.find(source_cache)?;
        return None;

        struct Traversal<'a, 'b> {
            state: &'a State,
            target: (SessionId, &'a Session),
            senario: Senario<&'b Arc<Vars>, &'b Arc<dyn Any + Send + Sync>>
        }
        impl<'a, 'b> Traversal<'a, 'b> {
            fn find(
                &self,
                f: impl Fn(&Flow) -> Option<Reuse<()>>
            ) -> Option<Reuse<(SessionId, FlowId, &'a Flow)>> {
                {
                    let sid = self.target.0;
                    let sess = self.target.1;
                    for (fid, flow) in sess.flows().rev() {
                        if let Some(r) = f(flow) {
                            return Some(r.map(|_| (sid, fid, flow)));
                        }
                    }
                }
                if self.senario.linearf.cache_across_sessions {
                    for &(sid, ref sess) in self.state.sessions.iter().rev() {
                        for (fid, flow) in sess.flows().rev() {
                            if let Some(r) = f(flow) {
                                return Some(r.map(|_| (sid, fid, flow)));
                            }
                        }
                    }
                }
                None
            }
        }
    }
}

impl State {
    pub fn resume(&mut self, id: SessionId) -> Result<FlowId, Error> {
        let sess = self
            .take_session(id)
            .ok_or_else(|| format!("session {:?} is not found", id))?;
        let fid = sess
            .flows()
            .next_back()
            .ok_or_else(|| format!("session {:?} has no flows", id))?
            .0;
        self.sessions.push_back((id, sess));
        Ok(fid)
    }

    pub fn get_flow(&self, s: SessionId, f: FlowId) -> Option<&Flow> {
        let sess = self.session(s)?;
        let flow = sess.flow(f)?;
        Some(flow)
    }

    pub fn remove_session(&mut self, session: SessionId) { self.take_session(session); }
}
