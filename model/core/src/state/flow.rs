mod cache_chunks;
mod fuse;

pub use crate::{converter::MapConvertError as StartError, matcher::WithScore};
use crate::{
    state::{Scenario, Shared},
    AsyncRt, ConverterRegistry, MatcherRegistry, SourceRegistry, Vars
};
use cache_chunks::CacheChunks;
use futures::{pin_mut, StreamExt};
use std::{any::Any, cmp::Ordering, collections::HashSet, mem, sync::Arc, time::Instant};
use tokio::{sync::RwLockReadGuard, task::JoinHandle};

pub struct Flow {
    at: Instant,
    scenario: UsedScenario<Arc<Vars>, Arc<dyn Any + Send + Sync>>,
    source: Option<Disposable<CacheChunks<WithScore>>>,
    matcher: Option<Disposable<Shared<Sorted>>>
}

#[derive(Clone)]
struct Disposable<T> {
    data: T,
    handles: Vec<Arc<JoinHandle<()>>>
}

#[derive(Debug, Clone, Default)]
pub struct Sorted {
    pub done: bool,
    pub items: Vec<WithScore>,
    pub source_count: usize
}

impl<T> Drop for Disposable<T> {
    fn drop(&mut self) {
        let handles = mem::take(&mut self.handles);
        for arc in handles.into_iter().rev() {
            if let Ok(handle) = Arc::try_unwrap(arc) {
                handle.abort()
            }
        }
    }
}

impl Drop for Flow {
    fn drop(&mut self) {
        mem::drop(self.matcher.take());
        mem::drop(self.source.take());
    }
}

impl Flow {
    #[inline]
    pub fn scenario(&self) -> UsedScenario<&Arc<Vars>, &Arc<dyn Any + Send + Sync>> {
        self.scenario.as_ref()
    }

    #[inline]
    pub(super) fn at(&self) -> Instant { self.at }

    #[inline]
    pub(super) fn has_cache(&self) -> bool { self.source.is_some() }

    pub(super) fn start<S, M, C>(
        rt: AsyncRt,
        source_registry: &S,
        matcher_registry: &M,
        converter_registry: &C,
        reuse: Result<Reuse<&mut Flow>, Instant>,
        scenario: Scenario<Arc<Vars>, Arc<dyn Any + Send + Sync>>
    ) -> Result<Self, StartError>
    where
        S: SourceRegistry,
        M: MatcherRegistry,
        C: ConverterRegistry
    {
        let (at, scenario, source) = match reuse {
            Ok(Reuse::Matcher(flow)) if flow.matcher.is_some() => {
                let matcher = flow.matcher.take();
                let source = flow.source.take();
                return Ok(Flow {
                    at: flow.at,
                    scenario: flow.scenario.clone(),
                    source,
                    matcher
                });
            }
            Ok(Reuse::Matcher(flow)) if flow.source.is_some() => {
                (flow.at, flow.scenario.clone(), flow.source.take().unwrap())
            }
            Ok(Reuse::Matcher(_)) => unreachable!(),
            Ok(Reuse::Source(flow)) if flow.source.is_some() => {
                let source = flow.source.clone().unwrap();
                let scenario = UsedScenario {
                    matcher: scenario.matcher,
                    sorted_vars: scenario.linearf,
                    ..flow.scenario.clone()
                };
                let a = source.data.renew();
                let b = a.flat_map(|chunk| {
                    futures::stream::unfold(chunk.into_iter(), |mut it| async {
                        it.next().map(|(i, _score)| (i, it))
                    })
                });
                let scores = matcher_registry.score(
                    &scenario.sorted_vars.matcher,
                    (&scenario.sorted_vars, &scenario.matcher),
                    b
                );
                let first_size = std::cmp::max(scenario.sorted_vars.first_view, 1);
                let chunk_size = std::cmp::max(scenario.sorted_vars.chunk_size, 1);
                let (load, cache) = cache_chunks::new_cache_chunks(scores, first_size, chunk_size);
                let source_handle = rt.spawn(load);
                let new_source = Disposable {
                    data: cache,
                    handles: {
                        let mut h = source.handles.clone();
                        h.push(Arc::new(source_handle));
                        h
                    }
                };
                (flow.at, scenario, new_source)
            }
            Ok(Reuse::Source(_)) => unreachable!(),
            Err(started) => {
                let v = &scenario.linearf;
                let a = source_registry.stream(&v.source, (v, &scenario.source));
                let b = if v.converters.is_empty() {
                    a
                } else {
                    converter_registry.map_convert(&v.converters, a)?
                };
                let c = b.map(Arc::new);
                let scores = matcher_registry.score(&v.matcher, (v, &scenario.matcher), c);
                let first_size = std::cmp::max(v.first_view, 1);
                let chunk_size = std::cmp::max(v.chunk_size, 1);
                let (load, cache) = cache_chunks::new_cache_chunks(scores, first_size, chunk_size);
                let source_handle = rt.spawn(load);
                let new_source = Disposable {
                    data: cache,
                    handles: vec![Arc::new(source_handle)]
                };
                (
                    started,
                    UsedScenario {
                        source: scenario.source,
                        stream_vars: v.clone(),
                        matcher: scenario.matcher,
                        sorted_vars: scenario.linearf
                    },
                    new_source
                )
            }
        };
        let sorted = Arc::default();
        let matcher_handle = run_sort(rt, Arc::clone(&sorted), source.data.renew());
        let handles = {
            let mut h = source.handles.clone();
            h.push(Arc::new(matcher_handle));
            h
        };
        Ok(Flow {
            at,
            scenario,
            source: Some(source),
            matcher: Some(Disposable {
                data: sorted,
                handles
            })
        })
    }

    pub(super) fn dispose(&mut self) {
        mem::drop(self.matcher.take());
        mem::drop(self.source.take());
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
pub struct UsedScenario<V, P> {
    pub source: P,
    pub stream_vars: V,
    pub matcher: P,
    pub sorted_vars: V
}

impl<V, P> UsedScenario<V, P> {
    fn as_ref(&self) -> UsedScenario<&V, &P> {
        UsedScenario {
            source: &self.source,
            stream_vars: &self.stream_vars,
            matcher: &self.matcher,
            sorted_vars: &self.sorted_vars
        }
    }
}

// TODO: improve
fn run_sort(rt: AsyncRt, sorted: Shared<Sorted>, chunks: CacheChunks<WithScore>) -> JoinHandle<()> {
    rt.spawn(async move {
        let start = Instant::now();
        pin_mut!(chunks);
        while let Some(mut chunk) = chunks.next().await {
            // +50ms desc
            let orig_size = chunk.len();
            let mut chunk = chunk
                .drain_filter(|(_, s)| !s.should_be_excluded())
                .collect::<Vec<_>>();
            // log::debug!("{}", chunk.len());
            // +1ms
            chunk.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
            // +0~1ms
            let sorted = &mut sorted.write().await;
            sorted.source_count += orig_size;
            merge(&mut sorted.items, &mut chunk, |a, b| {
                a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal)
            });
        }
        let sorted = &mut sorted.write().await;
        sorted.items.shrink_to_fit();
        sorted.done = true;
        log::debug!("{:?}", start.elapsed());
    })
}

fn merge<T>(a: &mut Vec<T>, b: &mut Vec<T>, cmp: impl Fn(&T, &T) -> Ordering) {
    if b.is_empty() {
        return;
    }
    let orig_len = a.len();
    a.reserve(b.len());
    unsafe {
        let a: &mut Vec<mem::MaybeUninit<T>> = mem::transmute(a);
        let b: &mut Vec<mem::MaybeUninit<T>> = mem::transmute(b);
        a.set_len(a.len() + b.len());
        let (mut n, mut m) = (
            (0..orig_len).rev().peekable(),
            (0..b.len()).rev().peekable()
        );
        for dst in (0..a.len()).rev() {
            match (n.peek(), m.peek()) {
                (Some(&i), Some(&j))
                    if cmp(mem::transmute(&a[i]), mem::transmute(&b[j])) == Ordering::Greater =>
                {
                    n.next();
                    a[dst] = mem::replace(&mut a[i], mem::MaybeUninit::uninit());
                }
                (Some(_), Some(&_j)) | (None, Some(&_j)) => {
                    m.next();
                    a[dst] = b.pop().unwrap();
                }
                (Some(_), None) => break,
                (None, None) => break
            }
        }
        assert_eq!(b.len(), 0);
        let _a: &mut Vec<T> = mem::transmute(a);
        let _b: &mut Vec<T> = mem::transmute(b);
    }
}

impl Flow {
    pub async fn sorted(&self) -> Option<RwLockReadGuard<'_, Sorted>> {
        Some(self.matcher.as_ref()?.data.read().await)
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
