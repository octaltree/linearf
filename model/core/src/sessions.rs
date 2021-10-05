use linearf::Vars;
use std::{collections::VecDeque, sync::Arc};

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Shared<T> = Arc<RwLock<T>>;

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

impl FlowId {
    const FIRST: FlowId = FlowId(1);
}

pub struct Sessions {
    last_id: SessionId,
    sessions: VecDeque<(SessionId, Session)>,
}

impl Sessions {
    pub fn new_shared() -> Shared<Self> {
        let this = Self {
            last_id: SessionId(0),
            sessions: VecDeque::new(),
        };
        Arc::new(RwLock::new(this))
    }

    pub async fn start_session<'a, D, S, M>(
        &mut self,
        rt: AsyncRt,
        source: Arc<S>,
        matcher: Arc<M>,
        senario: Senario<Vars, D>,
    ) -> Result<(SessionId, FlowId), Error>
    where
        D: serde::de::Deserializer<'a>,
        S: SourceRegistry<'a, D> + 'static + Send + Sync,
        M: MatcherRegistry<'a, D> + 'static + Send + Sync,
        <D as serde::de::Deserializer<'a>>::Error: Send + Sync + 'static,
    {
        let senario = parse_params(&source, &matcher, senario)?;
        let next_id = self.next_id();
        let (id, sess) = Session::start(rt, source, matcher, self, senario).await;

        let reusable = self.reusable(&source, &matcher, senario.as_ref(), next_id, FlowId::FIRST);
        let (id, sess) = match reusable {
            Some((sid, fid)) => {
                // unwrap: reusable returns the id that exists
                let mut s = self.remove_session(sid).unwrap();
                s.resume_flow(fid).unwrap();
                (sid, s)
            }
            None => {
                let sess = Session::start(rt, source, matcher, senario).await;
                self.last_id = next_id;
                (next_id, sess)
            }
        };
        let (fid, _) = sess.last_flow();
        self.sessions.push_back((id, sess));
        Ok((id, fid))
    }

    fn reusable<'a, D, S, M>(
        &self,
        source: &Arc<S>,
        matcher: &Arc<M>,
        senario: Senario<&Arc<Vars>, &Arc<dyn Any + Send + Sync>>,
        next_sid: SessionId,
        next_fid: FlowId,
    ) -> Option<(SessionId, FlowId)>
    where
        D: serde::de::Deserializer<'a>,
        S: SourceRegistry<'a, D> + 'static + Send + Sync,
        M: MatcherRegistry<'a, D> + 'static + Send + Sync,
    {
        let f = |sid: &SessionId, fid: &FlowId, flow: &Flow| {
            let vars = flow.vars();
            if vars.source != senario.linearf.source || vars.matcher != senario.linearf.matcher {
                return None;
            }
            let ctx = ReusableContext {
                same_session: sid == &next_sid,
            };
            if source.reusable(
                &senario.linearf.source,
                ctx.clone(),
                (vars, flow.source_params()),
                (senario.linearf, senario.source),
            ) && matcher.reusable(
                &senario.linearf.matcher,
                ctx,
                (vars, flow.matcher_params()),
                (senario.linearf, senario.matcher),
            ) {
                return Some((*sid, *fid));
            }
            None
        };
        // give priority to same session
        for (sid, sess) in self.sessions.iter().rev() {
            if sid != &next_sid {
                continue;
            }
            for (fid, flow) in sess.flows().iter().rev() {
                if let Some(t) = f(sid, fid, flow) {
                    return Some(t);
                }
            }
        }
        for (sid, sess) in self.sessions.iter().rev() {
            for (fid, flow) in sess.flows().iter().rev() {
                if let Some(t) = f(sid, fid, flow) {
                    return Some(t);
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
        senario: Senario<Vars, D>,
    ) -> Result<(SessionId, FlowId), Error>
    where
        D: serde::de::Deserializer<'a>,
        S: SourceRegistry<'a, D> + 'static + Send + Sync,
        M: MatcherRegistry<'a, D> + 'static + Send + Sync,
        <D as serde::de::Deserializer<'a>>::Error: Send + Sync + 'static,
    {
        todo!()
        //{
        //    let sess = self
        //        .session_mut(id)
        //        .ok_or_else(|| format!("session {} is not found", id.0))?;
        //    validate_senario(sess, &senario)?;
        //}
        // let senario = parse_params(&source, &matcher, senario)?;
        // let reusable = self.reusable(&source, &matcher, senario.as_ref());
        // std::mem::drop(sess);
        // let (id, sess) = match reusable {
        //    Some((sid, fid)) => {
        //        // unwrap: reusable returns the id that exists
        //        let mut s = self.remove_session(sid).unwrap();
        //        s.resume_flow(fid).unwrap();
        //        (sid, s)
        //    }
        //    None => {
        //        let mut s = self.remove_session(id).unwrap();
        //        let fid = s.tick(rt, source, matcher, senario).await;
        //        (id, s)
        //    }
        //};
        // let (fid, _) = sess.last_flow();
        // self.sessions.push_back((id, sess));
        // Ok((id, fid))
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

    pub fn next_id(&self) -> SessionId {
        SessionId(self.last_id.0 + 1)
    }

    fn use_next_id(&mut self) -> SessionId {
        self.last_id = self.next_id();
        self.last_id
    }

    pub fn session(&self, id: SessionId) -> Option<&Session> {
        let mut rev = self.sessions.iter().rev();
        rev.find(|s| s.0 == id).map(|(_, s)| s)
    }

    pub fn sessions(&self) -> impl Iterator<Item = &(SessionId, Session)> + DoubleEndedIterator {
        self.sessions.iter()
    }

    // fn session_mut(&mut self, id: SessionId) -> Option<&mut Session> {
    //    let mut rev = self.sessions.iter_mut().rev();
    //    rev.find(|s| s.0 == id).map(|(_, s)| s)
    //}
}

fn parse_params<'a, D, S, M>(
    source: &Arc<S>,
    matcher: &Arc<M>,
    senario: Senario<Vars, D>,
) -> Result<Senario<Arc<Vars>, Arc<dyn Any + Send + Sync>>, Error>
where
    D: serde::de::Deserializer<'a>,
    S: SourceRegistry<'a, D> + 'static + Send + Sync,
    M: MatcherRegistry<'a, D> + 'static + Send + Sync,
    <D as serde::de::Deserializer<'a>>::Error: Send + Sync + 'static,
{
    let Senario {
        linearf: s_linearf,
        source: s_source,
        matcher: s_matcher,
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
        matcher: matcher_params,
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

#[derive(Clone)]
pub struct Senario<V, P> {
    pub linearf: V,
    pub source: P,
    pub matcher: P,
}

impl<V, P> Senario<V, P> {
    fn as_ref(&self) -> Senario<&V, &P> {
        Senario {
            linearf: &self.linearf,
            source: &self.source,
            matcher: &self.matcher,
        }
    }
}
