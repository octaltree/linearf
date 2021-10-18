mod chunks;
mod fuse;

pub use crate::{converter::MapConvertError as StartError, matcher::WithScore};
use crate::{
    state::{Senario, Shared},
    AsyncRt, ConverterRegistry, MatcherRegistry, SourceRegistry, Vars
};
use chunks::Chunks;
use futures::{pin_mut, Stream, StreamExt};
use std::{
    any::Any, collections::BinaryHeap, future::Future, pin::Pin, sync::Arc, task::Poll,
    time::Instant
};
use tokio::sync::{RwLock, RwLockReadGuard};

#[derive(Clone)]
pub struct Flow {
    at: Instant,
    senario: UsedSenario<Arc<Vars>, Arc<dyn Any + Send + Sync>>,
    cache: CacheStream<WithScore>,
    sorted: Shared<(bool, Vec<WithScore>)>
}

impl Flow {
    #[inline]
    pub(super) fn senario(&self) -> UsedSenario<&Arc<Vars>, &Arc<dyn Any + Send + Sync>> {
        self.senario.as_ref()
    }

    #[inline]
    pub(super) fn at(&self) -> Instant { self.at }

    pub(super) fn start<S, M, C>(
        rt: AsyncRt,
        source: &S,
        matcher: &M,
        converter: &C,
        reuse: Result<Reuse<&Flow>, Instant>,
        senario: Senario<Arc<Vars>, Arc<dyn Any + Send + Sync>>
    ) -> Result<Self, StartError>
    where
        S: SourceRegistry,
        M: MatcherRegistry,
        C: ConverterRegistry
    {
        if let Ok(Reuse::Matcher(flow)) = reuse {
            return Ok(flow.clone());
        }
        let (at, scores, senario) = if let Ok(Reuse::Source(flow)) = reuse {
            let senario = UsedSenario {
                matcher: senario.matcher,
                sorted_vars: senario.linearf,
                ..flow.senario.clone()
            };
            let a = flow.cache.reload();
            let b = a.map(|WithScore { item: i, .. }| i);
            let scores = matcher.score(
                &senario.sorted_vars.matcher,
                (&senario.sorted_vars, &senario.matcher),
                b
            );
            (flow.at, scores, senario)
        } else if let Err(started) = reuse {
            let v = &senario.linearf;
            let a = source.stream(&v.source, (v, &senario.source));
            let b = converter.map_convert(&v.converters, a)?;
            let c = b.map(Arc::new);
            let scores = matcher.score(&v.matcher, (v, &senario.matcher), c);
            (
                started,
                scores,
                UsedSenario {
                    source: senario.source,
                    stream_vars: v.clone(),
                    matcher: senario.matcher,
                    sorted_vars: senario.linearf
                }
            )
        } else {
            unreachable!()
        };
        let cache = CacheStream::new(scores);
        let sorted = Arc::default();
        run_sort(rt, Arc::clone(&sorted), &cache, senario.sorted_vars.clone());
        Ok(Flow {
            at,
            senario,
            cache,
            sorted
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum Reuse<R> {
    Source(R),
    Matcher(R)
}

impl<T> Reuse<T> {
    pub(super) fn map<U>(self, f: impl Fn(T) -> U) -> Reuse<U> {
        match self {
            Reuse::Matcher(x) => Reuse::Matcher(f(x)),
            Reuse::Source(x) => Reuse::Source(f(x))
        }
    }
}

#[derive(Clone, Copy)]
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

#[derive(Clone)]
struct CacheStream<I> {
    inner: Arc<RwLock<CacheStreamImpl<I>>>,
    idx: usize,
    need_write: bool
}

struct CacheStreamImpl<I> {
    buf: Vec<Option<I>>,
    gen: Pin<Box<dyn Stream<Item = I> + Send + Sync>>,
    done: bool
}

impl<I> CacheStream<I> {
    fn new(st: Pin<Box<dyn Stream<Item = I> + Send + Sync>>) -> CacheStream<I> {
        let buf = if let Some(x) = st.size_hint().1 {
            Vec::with_capacity(x)
        } else {
            Vec::with_capacity(1024)
        };
        CacheStream {
            inner: Arc::new(RwLock::new(CacheStreamImpl {
                buf,
                gen: st,
                done: false
            })),
            idx: 0,
            need_write: false
        }
    }

    fn reload(&self) -> CacheStream<I> {
        CacheStream {
            inner: self.inner.clone(),
            idx: 0,
            need_write: false
        }
    }
}

impl<I> CacheStream<I>
where
    I: Clone
{
    fn write_impl(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<I>> {
        let this = self.get_mut();
        let write = this.inner.write();
        pin_mut!(write);
        let mut inner = match write.poll(cx) {
            Poll::Pending => {
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
            Poll::Ready(x) => x
        };
        if inner.done {
            // I just want to recur, not pending.
            this.need_write = false;
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        let gen = inner.gen.as_mut();
        let x: Option<_> = match gen.poll_next(cx) {
            Poll::Pending => {
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
            Poll::Ready(x) => x
        };
        if x.is_none() {
            inner.done = true;
        }
        inner.buf.push(x);
        if let Some(x) = inner.buf.get(this.idx) {
            this.idx += 1;
            this.need_write = false;
            Poll::Ready(x.clone())
        } else {
            //
            this.need_write = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }

    fn read_impl(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<I>> {
        let this = self.get_mut();
        let read = this.inner.read();
        pin_mut!(read);
        let inner = match read.poll(cx) {
            Poll::Pending => {
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
            Poll::Ready(x) => x
        };
        if let Some(x) = inner.buf.get(this.idx) {
            this.idx += 1;
            Poll::Ready(x.clone())
        } else {
            //
            this.need_write = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }

    async fn _size_hint(&self) -> usize {
        let inner = self.inner.read().await;
        inner.buf.len() + inner.gen.size_hint().0
    }
}

impl<I> Stream for CacheStream<I>
where
    I: Clone
{
    type Item = I;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>
    ) -> Poll<Option<Self::Item>> {
        if self.need_write {
            self.write_impl(cx)
        } else {
            self.read_impl(cx)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) { (0, None) }
}

// TODO: improve
fn run_sort(
    rt: AsyncRt,
    sorted: Shared<(bool, Vec<WithScore>)>,
    cache: &CacheStream<WithScore>,
    vars: Arc<Vars>
) {
    let preload = cache.reload();
    let stream = cache.reload();
    rt.spawn(async move {
        pin_mut!(preload);
        while preload.next().await.is_some() {}
    });
    let first_size = std::cmp::max(vars.first_view, 1);
    let chunk_size = std::cmp::max(vars.chunk_size, 1);
    rt.spawn(async move {
        let start = Instant::now();
        pin_mut!(stream);
        let mut heap = BinaryHeap::with_capacity(stream._size_hint().await);
        let mut i = 0;
        while let Some(ws) = stream.next().await {
            if !ws.score.should_be_excluded() {
                heap.push(ws);
            }
            i += 1;
            if i >= chunk_size {
                i = 0;
                let xs = heap.clone().into_sorted_vec();
                let sorted = &mut sorted.write().await;
                sorted.1 = xs;
            }
        }

        // let mut chunks = Chunks::new(stream, first_size, chunk_size);
        // while let Some(mut chunk) = chunks.next().await {
        //    let mut chunk = chunk
        //        .drain_filter(|WithScore { score: s, .. }| !s.should_be_excluded())
        //        .collect::<Vec<_>>();
        //    log::debug!("{}", chunk.len());
        //    chunk.sort_unstable();
        //    let sorted = &mut sorted.write().await;
        //    sorted.1.append(&mut chunk);
        //    sorted.1.sort();
        //}
        let xs = heap.clone().into_sorted_vec();
        let sorted = &mut sorted.write().await;
        sorted.1 = xs;
        sorted.0 = true;
        log::debug!("{:?}", start.elapsed());
    });
}

impl Flow {
    pub fn sorted(&self) -> impl Future<Output = RwLockReadGuard<(bool, Vec<WithScore>)>> {
        self.sorted.read()
    }

    // pub async fn sorted_status(&self) -> (bool, usize) {
    //    let sorted = self.sorted().await;
    //    (sorted.0, sorted.1.len())
    //}

    // pub async fn sorted_items(&self, start: usize, end: usize) -> Vec<Arc<Item>> {
    //    let sorted = self.sorted().await;
    //    let xs = &sorted.1;
    //    xs[start..std::cmp::min(end, xs.len())]
    //        .iter()
    //        .map(|(i, _)| i.clone())
    //        .collect::<Vec<_>>()
    //}
}
