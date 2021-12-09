mod flow;

use crate::{source::Reusable, AsyncRt, ConverterRegistry, MatcherRegistry, SourceRegistry, Vars};
pub use flow::*;
use flow::{Reuse, StartError};
use serde::{Deserialize, Serialize};
use smartstring::alias::String as SmartString;
use std::{any::Any, collections::VecDeque, sync::Arc, time::Instant};
use tokio::sync::RwLock;

pub type Shared<T> = Arc<RwLock<T>>;

pub struct State {
    last_id: SessionId,
    // accepts only sessions that has one or more flows
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

    pub async fn start_flow<'a, D, S, M, C>(
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
                let sess = self.session(id).ok_or(Error::SessionNotFound(id))?;
                validate_scenario(sess, &request.scenario)?;
                let sess = self.take_session(id).unwrap();
                (id, sess)
            }
        };
        let scenario = parse_scenario(source, matcher, request.scenario)?;
        let reuse: Result<Reuse<(SessionId, FlowId)>, Instant> = match reusable(
            self,
            (id, &target),
            source,
            matcher,
            scenario.as_ref(),
            started
        ) {
            Some(r) => {
                log::debug!("reuse {:?}", r.map(|(s, f, _)| (s, f)));
                Ok(r.map(|(s, f, _)| (s, f)))
            }
            None => Err(started)
        };
        let reuse: Result<Reuse<&mut Flow>, Instant> = reuse.and_then(|r| {
            r.map(|(s, f)| flow_mut(self, (id, &mut target), s, f))
                .optional()
                .ok_or(started)
        });
        log::debug!(
            "reuse {:?}",
            reuse.as_ref().map(|r| r.as_ref().map(|f| f.scenario()))
        );
        let dispose_flow = scenario.linearf.dispose_flow;
        let flow =
            Flow::start(rt, source, matcher, converter, reuse, scenario).map_err(|e| match e {
                StartError::ConverterNotFound(n) => Error::ConverterNotFound(n)
            })?;
        if dispose_flow {
            dispose_flows(&mut target);
        }
        let fid = target.push(flow);
        self.sessions.push_back((id, target));
        Ok((id, fid))
    }
}

pub struct StartFlow<D> {
    pub id: Option<SessionId>,
    pub scenario: Scenario<Vars, D>
}

#[derive(Clone, Copy)]
pub struct Scenario<V, P> {
    pub linearf: V,
    pub source: P,
    pub matcher: P
}

impl<V, P> Scenario<V, P> {
    fn as_ref(&self) -> Scenario<&V, &P> {
        Scenario {
            linearf: &self.linearf,
            source: &self.source,
            matcher: &self.matcher
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(transparent)]
pub struct SessionId(pub i32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(transparent)]
pub struct FlowId(pub usize);

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Source {0:?} is not found")]
    SourceNotFound(SmartString),
    #[error("Matcher {0:?} is not found")]
    MatcherNotFound(SmartString),
    #[error("Converter {0:?} is not found")]
    ConverterNotFound(SmartString),
    #[error("Session {0:?} is not found")]
    SessionNotFound(SessionId),
    #[error("Flow {0:?} {1:?} is not found")]
    FlowNotFound(SessionId, FlowId),
    #[error("Flow must have the same source in session: {0:?} != {1:?}")]
    ScenarioSource(SmartString, SmartString),
    #[error("Flow must have the same matcher in session: {0:?} != {1:?}")]
    ScenarioMatcher(SmartString, SmartString),
    #[error("Flow must have the same converters in session: {0:?} != {1:?}")]
    ScenarioConverters(Vec<SmartString>, Vec<SmartString>),
    #[error("{0}")]
    Others(Box<dyn std::error::Error + Send + Sync>),
    #[error("Flow {0:?} {1:?} is disposed")]
    FlowDisposed(SessionId, FlowId)
}

pub struct Session {
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
fn validate_scenario<D>(session: &Session, scenario: &Scenario<Vars, D>) -> Result<(), Error> {
    let flow = session
        .flows()
        .next_back()
        .expect("Session must have one or more flows")
        .1;
    let prev = &flow.scenario();
    if prev.sorted_vars.source != scenario.linearf.source {
        return Err(Error::ScenarioSource(
            prev.sorted_vars.source.clone(),
            scenario.linearf.source.clone()
        ));
    }
    if prev.sorted_vars.matcher != scenario.linearf.matcher {
        return Err(Error::ScenarioMatcher(
            prev.sorted_vars.matcher.clone(),
            scenario.linearf.matcher.clone()
        ));
    }
    if prev.sorted_vars.converters != scenario.linearf.converters {
        return Err(Error::ScenarioConverters(
            prev.sorted_vars.converters.clone(),
            scenario.linearf.converters.clone()
        ));
    }
    Ok(())
}

fn parse_scenario<'a, D, S, M>(
    source: &S,
    matcher: &M,
    scenario: Scenario<Vars, D>
) -> Result<Scenario<Arc<Vars>, Arc<dyn Any + Send + Sync>>, Error>
where
    D: serde::de::Deserializer<'a>,
    <D as serde::de::Deserializer<'a>>::Error: Send + Sync + 'static,
    S: SourceRegistry,
    M: MatcherRegistry
{
    let Scenario {
        linearf: s_linearf,
        source: s_source,
        matcher: s_matcher
    } = scenario;
    let s_linearf = Arc::new(s_linearf);
    let source_params = source
        .parse(&s_linearf.source, s_source)
        .ok_or_else(|| Error::SourceNotFound(s_linearf.source.clone()))?
        .map_err(|e| Error::Others(e.into()))?;
    let matcher_params = matcher
        .parse(&s_linearf.matcher, s_matcher)
        .ok_or_else(|| Error::MatcherNotFound(s_linearf.matcher.clone()))?
        .map_err(|e| Error::Others(e.into()))?;
    Ok(Scenario {
        linearf: s_linearf,
        source: source_params,
        matcher: matcher_params
    })
}

/// Reusable::Same&&Reuse::Matcher >
/// Reuseable::Cache&&Reuse::Matcher && use it offered by vars >
/// Reusable::Same&&Reuse::Source >
/// Reusable::Cache&&Reuse::Source && use it offered by vars >
/// None
fn reusable<'a, S, M>(
    state: &'a State,
    target: (SessionId, &'a Session),
    source: &S,
    matcher: &M,
    scenario: Scenario<&Arc<Vars>, &Arc<dyn Any + Send + Sync>>,
    started: Instant
) -> Option<Reuse<(SessionId, FlowId, &'a Flow)>>
where
    S: SourceRegistry,
    M: MatcherRegistry
{
    let use_cache = |flow: &Flow| -> bool {
        (started - flow.at()).as_secs() < scenario.linearf.cache_sec.into()
    };
    let source_reusable = |flow: &Flow| {
        if !flow.has_cache()
            || flow.scenario().stream_vars.source != scenario.linearf.source
            || flow.scenario().stream_vars.converters != scenario.linearf.converters
        {
            return Reusable::None;
        }
        source.reusable(
            &scenario.linearf.source,
            (flow.scenario().stream_vars, flow.scenario().source),
            (scenario.linearf, scenario.source)
        )
    };
    let matcher_reusable = |flow: &Flow| {
        if flow.scenario().sorted_vars.matcher != scenario.linearf.matcher {
            return Reusable::None;
        }
        match (
            source_reusable(flow),
            matcher.reusable(
                &scenario.linearf.matcher,
                (flow.scenario().sorted_vars, flow.scenario().matcher),
                (scenario.linearf, scenario.matcher)
            )
        ) {
            (Reusable::Same, Reusable::Same) => Reusable::Same,
            (Reusable::Cache, Reusable::Same) => Reusable::Cache,
            (Reusable::Same, Reusable::Cache) => Reusable::Cache,
            (Reusable::Cache, Reusable::Cache) => Reusable::Cache,
            _ => Reusable::None
        }
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
    let traversal = Traversal { state, target };
    // should I find newest cache?
    macro_rules! return_found {
        ($e:expr) => {
            if let Some(x) = $e {
                return Some(x);
            }
        };
    }
    return_found!(traversal.find(matcher_same, true));
    return_found!(traversal.find(matcher_cache, scenario.linearf.cache_across_sessions));
    return_found!(traversal.find(source_same, true));
    return_found!(traversal.find(source_cache, scenario.linearf.cache_across_sessions));
    return None;

    struct Traversal<'a> {
        state: &'a State,
        target: (SessionId, &'a Session)
    }
    impl<'a> Traversal<'a> {
        fn find(
            &self,
            f: impl Fn(&Flow) -> Option<Reuse<()>>,
            across: bool
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
            if across {
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

fn flow_mut<'a>(
    state: &'a mut State,
    target: (SessionId, &'a mut Session),
    s: SessionId,
    f: FlowId
) -> Option<&'a mut Flow> {
    if target.0 == s {
        return target.1.flows.get_mut(f.0);
    }
    let mut rev = state.sessions.iter_mut().rev();
    let sess = rev.find(|x| x.0 == s).map(|(_, s)| s)?;
    sess.flows.get_mut(f.0)
}

fn dispose_flows(session: &mut Session) {
    for flow in &mut session.flows {
        flow.dispose();
    }
}

impl State {
    pub fn resume(&mut self, id: SessionId) -> Result<FlowId, Error> {
        let sess = self.take_session(id).ok_or(Error::SessionNotFound(id))?;
        let fid = sess
            .flows()
            .next_back()
            .expect("Session must have one or more flows")
            .0;
        self.sessions.push_back((id, sess));
        Ok(fid)
    }

    pub fn get_flow(&self, s: SessionId, f: FlowId) -> Option<&Flow> {
        let sess = self.session(s)?;
        let flow = sess.flow(f)?;
        Some(flow)
    }
    pub fn try_get_flow(&self, s: SessionId, f: FlowId) -> Result<&Flow, Error> {
        self.get_flow(s, f).ok_or(Error::FlowNotFound(s, f))
    }

    pub fn remove_session(&mut self, session: SessionId) { self.take_session(session); }

    pub fn remove_all_sesions(&mut self) { self.sessions = VecDeque::new(); }

    pub fn sessions(&self) -> impl DoubleEndedIterator<Item = &(SessionId, Session)> {
        self.sessions.iter()
    }
}
