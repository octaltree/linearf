use futures::{pin_mut, Stream};
use std::{
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc
    },
    task::Poll
};
use tokio::sync::RwLock;

pub type AsyncRt = tokio::runtime::Handle;
pub type Shared<T> = Arc<RwLock<T>>;

pub struct CacheStream<I> {
    inner: Shared<Cached<I>>,
    len: Arc<AtomicUsize>,
    idx: usize,
    rt: AsyncRt
}

struct Cached<I> {
    buf: Vec<Option<I>>,
    done: bool
}

impl<I> CacheStream<I> {
    pub fn new(rt: AsyncRt, st: Pin<Box<dyn Stream<Item = I> + Send + Sync>>) -> (Load<I>, Self) {
        let buf = if let Some(x) = st.size_hint().1 {
            Vec::with_capacity(x)
        } else {
            Vec::with_capacity(1024)
        };
        let inner = Arc::new(RwLock::new(Cached { buf, done: false }));
        let inner2 = inner.clone();
        let len = Arc::new(AtomicUsize::new(0));
        (
            Load {
                gen: st,
                inner,
                len: len.clone(),
                write: Vec::with_capacity(100)
            },
            CacheStream {
                inner: inner2,
                len,
                idx: 0,
                rt
            }
        )
    }

    pub fn new_reload_only(&self) -> Self {
        CacheStream {
            inner: self.inner.clone(),
            len: self.len.clone(),
            idx: 0,
            rt: self.rt.clone()
        }
    }
}

pub struct Load<I> {
    gen: Pin<Box<dyn Stream<Item = I> + Send + Sync>>,
    inner: Shared<Cached<I>>,
    len: Arc<AtomicUsize>,
    write: Vec<Option<I>>
}

impl<I> Future for Load<I>
where
    I: Unpin
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let write = this.inner.write();
        pin_mut!(write);
        let gen = this.gen.as_mut();
        if this.write.is_empty() {
            let x: Option<_> = match gen.poll_next(cx) {
                Poll::Pending => {
                    // cx.waker().wake_by_ref();
                    return Poll::Pending;
                }
                Poll::Ready(x) => x
            };
            let mut inner = match write.poll(cx) {
                Poll::Pending => {
                    this.write.push(x);
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                }
                Poll::Ready(inner) => inner
            };
            if x.is_none() {
                inner.done = true;
            }
            inner.buf.push(x);
            this.len.store(inner.buf.len(), Ordering::Relaxed);
            // this.tx.send(inner.buf.len()).ok();
            if inner.done {
                Poll::Ready(())
            } else {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        } else if this.write.iter().any(|x| x.is_none()) {
            let mut inner = match write.poll(cx) {
                Poll::Pending => {
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                }
                Poll::Ready(inner) => inner
            };
            inner.done = true;
            inner.buf.append(&mut this.write);
            this.len.store(inner.buf.len(), Ordering::Relaxed);
            // this.tx.send(inner.buf.len()).ok();
            Poll::Ready(())
        } else {
            let (next, inner) = match (gen.poll_next(cx), write.poll(cx)) {
                (Poll::Pending, Poll::Pending) => {
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                }
                (Poll::Ready(x), Poll::Pending) => (Some(x), None),
                (Poll::Pending, Poll::Ready(inner)) => (None, Some(inner)),
                (Poll::Ready(x), Poll::Ready(inner)) => (Some(x), Some(inner))
            };
            if let Some(x) = next {
                this.write.push(x);
            }
            if let Some(mut inner) = inner {
                if let Some(None) = this.write.last() {
                    inner.done = true;
                }
                inner.buf.append(&mut this.write);
                this.len.store(inner.buf.len(), Ordering::Relaxed);
                // this.tx.send(inner.buf.len()).ok();
                if inner.done {
                    return Poll::Ready(());
                }
            }
            cx.waker().wake_by_ref();
            Poll::Pending
        }
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
        let this = self.get_mut();
        if this.idx > this.len.load(Ordering::Relaxed) {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        let read = this.inner.read();
        pin_mut!(read);
        let inner = match read.poll(cx) {
            Poll::Pending => {
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
            Poll::Ready(inner) => inner
        };
        if let Some(x) = inner.buf.get(this.idx) {
            this.idx += 1;
            return Poll::Ready(x.clone());
        }
        if inner.done {
            return Poll::Ready(None);
        }
        cx.waker().wake_by_ref();
        // let waker = cx.waker().clone();
        // let mut len = this.len.clone();
        // let idx = this.idx;
        // this.rt.spawn(async move {
        //    loop {
        //        match len.changed().await {
        //            Ok(()) => {
        //                if idx <= *len.borrow() {
        //                    waker.wake();
        //                    break;
        //                }
        //            }
        //            Err(_) => break
        //        }
        //    }
        //});
        Poll::Pending
    }
}
