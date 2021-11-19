use super::fuse::Fuse;
use futures::{pin_mut, Stream};
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker}
};
use tokio::sync::{Mutex, RwLock};

type Shared<T> = Arc<RwLock<T>>;

const N: usize = 10;

#[derive(Clone)]
pub struct CacheChunks<I> {
    cached: Shared<Vec<Option<Vec<I>>>>,
    wakers: Arc<Mutex<Vec<Waker>>>,
    idx: usize
}

pub struct Load<S, I>
where
    S: Stream<Item = I>
{
    cached: Shared<Vec<Option<Vec<I>>>>,
    wakers: Arc<Mutex<Vec<Waker>>>,
    stream: Fuse<S>,
    chunk: Vec<I>,
    waits_write: bool,
    last_chunk: bool,
    cap: usize,
    size: usize
}

pub fn new_cache_chunks<S, I>(
    stream: S,
    first_size: usize,
    cap: usize
) -> (Load<S, I>, CacheChunks<I>)
where
    S: Stream<Item = I>,
    I: Clone
{
    assert!(first_size > 0);
    assert!(cap > 0);
    let cached = Arc::default();
    let wakers = Arc::default();
    let size = first_size;
    (
        Load {
            cached: Arc::clone(&cached),
            wakers: Arc::clone(&wakers),
            stream: Fuse::new(stream),
            chunk: Vec::with_capacity(size + N),
            waits_write: false,
            last_chunk: false,
            cap,
            size
        },
        CacheChunks {
            cached,
            wakers,
            idx: 0
        }
    )
}

impl<I> CacheChunks<I> {
    pub fn renew(&self) -> Self {
        Self {
            cached: self.cached.clone(),
            wakers: self.wakers.clone(),
            idx: 0
        }
    }
}

impl<S, I> Future for Load<S, I>
where
    S: Stream<Item = I> + std::marker::Unpin,
    I: std::marker::Unpin
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<<Self as Future>::Output> {
        let this = self.get_mut();
        if !this.last_chunk {
            this.fetch(cx);
        }
        if this.waits_write {
            this.write(cx)
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

impl<S, I> Load<S, I>
where
    S: Stream<Item = I> + std::marker::Unpin,
    I: std::marker::Unpin
{
    #[inline]
    fn write(&mut self, cx: &mut Context<'_>) -> Poll<<Self as futures::Future>::Output> {
        let cached = self.cached.write();
        let wakers = self.wakers.lock();
        pin_mut!(cached);
        pin_mut!(wakers);
        let (mut cached, mut wakers) = match (cached.poll(cx), wakers.poll(cx)) {
            (Poll::Ready(cached), Poll::Ready(wakers)) => (cached, wakers),
            _ => {
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
        };
        self.waits_write = false;
        self.size = self.cap;
        cached.push(Some(std::mem::replace(
            &mut self.chunk,
            Vec::with_capacity(self.size + N)
        )));
        if self.last_chunk {
            cached.push(None);
        }
        std::mem::drop(cached);
        let wakers = std::mem::take(&mut *wakers);
        for w in wakers {
            w.wake();
        }
        if self.last_chunk {
            Poll::Ready(())
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }

    #[inline]
    fn fetch(&mut self, cx: &mut Context<'_>) {
        loop {
            let s = &mut self.stream;
            pin_mut!(s);
            let x: Option<_> = match s.poll_next(cx) {
                Poll::Pending => return,
                Poll::Ready(x) => x
            };
            if let Some(x) = x {
                self.chunk.push(x);
                if self.size <= self.chunk.len() {
                    self.waits_write = true;
                    break;
                }
            } else {
                self.last_chunk = true;
                self.waits_write = true;
                break;
            }
        }
    }
}

impl<I> Stream for CacheChunks<I>
where
    I: Clone
{
    type Item = Vec<I>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if let Some(poll) = this.fetch(cx) {
            poll
        } else {
            this.schedule(cx)
        }
    }
}

impl<I> CacheChunks<I>
where
    I: Clone
{
    #[inline]
    fn fetch(&mut self, cx: &mut Context<'_>) -> Option<Poll<Option<<Self as Stream>::Item>>> {
        let cached = self.cached.read();
        pin_mut!(cached);
        let chunks = match cached.poll(cx) {
            Poll::Pending => {
                cx.waker().wake_by_ref();
                return Some(Poll::Pending);
            }
            Poll::Ready(chunks) => chunks
        };
        if let Some(x) = chunks.get(self.idx) {
            if x.is_some() {
                self.idx += 1;
            }
            Some(Poll::Ready(x.clone()))
        } else {
            None
        }
    }

    #[inline]
    fn schedule(&self, cx: &mut Context<'_>) -> Poll<Option<<Self as Stream>::Item>> {
        let waker = cx.waker().clone();
        let wakers = self.wakers.lock();
        pin_mut!(wakers);
        let mut wakers = match wakers.poll(cx) {
            Poll::Ready(wakers) => wakers,
            Poll::Pending => {
                waker.wake_by_ref();
                return Poll::Pending;
            }
        };
        wakers.push(waker);
        Poll::Pending
    }
}
