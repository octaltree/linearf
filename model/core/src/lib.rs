pub mod item;
pub mod session;
pub mod source;

pub use crate::{
    item::{Item, MaybeUtf8},
    session::{Session, Vars}
};

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
        source: Arc<S>,
        senario: Senario<D, D>
    ) -> Result<(SessionId, &Shared<Session>), Error>
    where
        D: serde::de::Deserializer<'a>,
        S: SourceRegistry<'a, D> + 'static,
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
        let sess = Session::start(s_linearf, source_params, source);
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

pub trait New {
    fn new(_state: &Shared<State>) -> Self
    where
        Self: Sized;
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
    ) -> Result<Option<Arc<dyn std::any::Any + Send + Sync>>, D::Error>;
}

pub struct Senario<S, M> {
    pub linearf: Vars,
    pub source: S,
    pub matcher: M
}
