use futures::{pin_mut, Stream};
use std::{future::Future, pin::Pin, sync::Arc, task::Poll};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct CacheStream<I> {
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
    pub fn new(st: Pin<Box<dyn Stream<Item = I> + Send + Sync>>) -> CacheStream<I> {
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

    pub fn reload(&self) -> CacheStream<I> {
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
}
