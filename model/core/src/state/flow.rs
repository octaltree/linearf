mod cache;
mod chunks;
mod fuse;

pub use crate::{converter::MapConvertError as StartError, matcher::WithScore};
use crate::{
    state::{Senario, Shared},
    AsyncRt, ConverterRegistry, MatcherRegistry, SourceRegistry, Vars
};
use cache::CacheStream;
use chunks::Chunks;
use futures::{pin_mut, StreamExt};
use std::{any::Any, collections::HashSet, sync::Arc, time::Instant};
use tokio::{sync::RwLockReadGuard, task::JoinHandle};

pub struct Flow {
    at: Instant,
    senario: UsedSenario<Arc<Vars>, Arc<dyn Any + Send + Sync>>,
    cache: CacheStream<WithScore>,
    sorted: Option<Shared<Sorted>>,
    // source_handle: Option<JoinHandle<()>>,
    // matcher_handle: Option<JoinHandle<()>>,
    handles: Vec<JoinHandle<()>>
}

#[derive(Debug, Clone, Default)]
pub struct Sorted {
    pub done: bool,
    pub items: Vec<WithScore>,
    pub source_count: usize
}

impl Flow {
    #[inline]
    pub fn senario(&self) -> UsedSenario<&Arc<Vars>, &Arc<dyn Any + Send + Sync>> {
        self.senario.as_ref()
    }

    #[inline]
    pub(super) fn at(&self) -> Instant { self.at }

    pub(super) fn start<S, M, C>(
        rt: AsyncRt,
        source: &S,
        matcher: &M,
        converter: &C,
        reuse: Result<Reuse<&mut Flow>, Instant>,
        senario: Senario<Arc<Vars>, Arc<dyn Any + Send + Sync>>
    ) -> Result<Self, StartError>
    where
        S: SourceRegistry,
        M: MatcherRegistry,
        C: ConverterRegistry
    {
        let (at, senario, cache) = match reuse {
            Ok(Reuse::Matcher(flow)) if flow.sorted.is_some() => {
                let sorted = flow.sorted.take();
                let handles = std::mem::take(&mut flow.handles);
                return Ok(Flow {
                    at: flow.at,
                    senario: flow.senario.clone(),
                    cache: flow.cache.clone(),
                    sorted,
                    handles
                });
            }
            Ok(Reuse::Matcher(flow)) => (flow.at, flow.senario.clone(), flow.cache.clone()),
            Ok(Reuse::Source(flow)) => {
                let senario = UsedSenario {
                    matcher: senario.matcher,
                    sorted_vars: senario.linearf,
                    ..flow.senario.clone()
                };
                let a = flow.cache.reload();
                let b = a.map(|(i, _)| i);
                let scores = matcher.score(
                    &senario.sorted_vars.matcher,
                    (&senario.sorted_vars, &senario.matcher),
                    b
                );
                let cache = CacheStream::new(scores);
                (flow.at, senario, cache)
            }
            Err(started) => {
                let v = &senario.linearf;
                let a = source.stream(&v.source, (v, &senario.source));
                let b = converter.map_convert(&v.converters, a)?;
                let c = b.map(Arc::new);
                let scores = matcher.score(&v.matcher, (v, &senario.matcher), c);
                let cache = CacheStream::new(scores);
                (
                    started,
                    UsedSenario {
                        source: senario.source,
                        stream_vars: v.clone(),
                        matcher: senario.matcher,
                        sorted_vars: senario.linearf
                    },
                    cache
                )
            }
        };
        let sorted = Arc::default();
        let handles = run_sort(rt, Arc::clone(&sorted), &cache, senario.sorted_vars.clone());
        Ok(Flow {
            at,
            senario,
            cache,
            sorted: Some(sorted),
            handles
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum Reuse<R> {
    Source(R),
    Matcher(R)
}

impl<T> Reuse<T> {
    pub(super) fn map<U>(self, f: impl FnOnce(T) -> U) -> Reuse<U> {
        match self {
            Reuse::Matcher(x) => Reuse::Matcher(f(x)),
            Reuse::Source(x) => Reuse::Source(f(x))
        }
    }

    pub(super) fn as_ref(&self) -> Reuse<&T> {
        match self {
            Reuse::Matcher(ref x) => Reuse::Matcher(x),
            Reuse::Source(ref x) => Reuse::Source(x)
        }
    }
}

impl<T> Reuse<Option<T>> {
    pub(super) fn optional(self) -> Option<Reuse<T>> {
        match self {
            Reuse::Matcher(x) => x.map(Reuse::Matcher),
            Reuse::Source(x) => x.map(Reuse::Source)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UsedSenario<V, P> {
    pub source: P,
    pub stream_vars: V,
    pub matcher: P,
    pub sorted_vars: V
}

impl<V, P> UsedSenario<V, P> {
    fn as_ref(&self) -> UsedSenario<&V, &P> {
        UsedSenario {
            source: &self.source,
            stream_vars: &self.stream_vars,
            matcher: &self.matcher,
            sorted_vars: &self.sorted_vars
        }
    }
}

// TODO: improve
fn run_sort(
    rt: AsyncRt,
    sorted: Shared<Sorted>,
    cache: &CacheStream<WithScore>,
    vars: Arc<Vars>
) -> Vec<JoinHandle<()>> {
    let preload = cache.reload();
    let stream = cache.reload();
    let mut ret = Vec::with_capacity(2);
    ret.push(rt.spawn(async move {
        pin_mut!(preload);
        while preload.next().await.is_some() {}
    }));
    let first_size = std::cmp::max(vars.first_view, 1);
    let chunk_size = std::cmp::max(vars.chunk_size, 1);
    ret.push(rt.spawn(async move {
        let start = Instant::now();
        pin_mut!(stream);
        let mut chunks = Chunks::new(stream, first_size, chunk_size);
        while let Some(mut chunk) = chunks.next().await {
            let orig_size = chunk.len();
            let mut chunk = chunk
                .drain_filter(|(_, s)| !s.should_be_excluded())
                .collect::<Vec<_>>();
            // log::debug!("{}", chunk.len());
            chunk.sort_unstable_by(|a, b| a.1.cmp(&b.1));
            let sorted = &mut sorted.write().await;
            sorted.source_count += orig_size;
            sorted.items.append(&mut chunk);
            sorted.items.sort_by(|a, b| a.1.cmp(&b.1));
        }
        let sorted = &mut sorted.write().await;
        sorted.done = true;
        log::debug!("{:?}", start.elapsed());
    }));
    ret
}

impl Flow {
    pub async fn sorted(&self) -> Option<RwLockReadGuard<'_, Sorted>> {
        Some(self.sorted.as_ref()?.read().await)
    }
}

impl Sorted {
    pub fn id_items<'a>(&'a self, ids: &'a HashSet<u32>) -> impl Iterator<Item = WithScore> + 'a {
        self.items
            .iter()
            .filter(|(i, _)| ids.contains(&i.id))
            .cloned()
    }
}
