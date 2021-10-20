use super::fuse::Fuse;
use core::{mem, pin::Pin};
use futures::{
    ready,
    stream::{FusedStream, Stream},
    task::{Context, Poll}
};
use pin_project_lite::pin_project;

pin_project! {
    /// Stream for the [`chunks`](super::StreamExt::chunks) method.
    #[derive(Debug)]
    #[must_use = "streams do nothing unless polled"]
    pub struct Chunks<St: Stream> {
        #[pin]
        stream: Fuse<St>,
        items: Vec<St::Item>,
        is_first: bool,
        first_cap: usize,
        cap: usize, // https://github.com/rust-lang/futures-rs/issues/1475
    }
}

// macro_rules! delegate_access_inner {
//    ($field:ident, $inner:ty, ($($ind:tt)*)) => {
//        /// Acquires a reference to the underlying sink or stream that this combinator is
//        /// pulling from.
//        pub fn get_ref(&self) -> &$inner {
//            (&self.$field) $($ind get_ref())*
//        }

//        /// Acquires a mutable reference to the underlying sink or stream that this
//        /// combinator is pulling from.
//        ///
//        /// Note that care must be taken to avoid tampering with the state of the
//        /// sink or stream which may otherwise confuse this combinator.
//        pub fn get_mut(&mut self) -> &mut $inner {
//            (&mut self.$field) $($ind get_mut())*
//        }

//        /// Acquires a pinned mutable reference to the underlying sink or stream that this
//        /// combinator is pulling from.
//        ///
//        /// Note that care must be taken to avoid tampering with the state of the
//        /// sink or stream which may otherwise confuse this combinator.
//        pub fn get_pin_mut(self: core::pin::Pin<&mut Self>) -> core::pin::Pin<&mut $inner> {
//            self.project().$field $($ind get_pin_mut())*
//        }

//        /// Consumes this combinator, returning the underlying sink or stream.
//        ///
//        /// Note that this may discard intermediate state of this combinator, so
//        /// care should be taken to avoid losing resources when this is called.
//        pub fn into_inner(self) -> $inner {
//            self.$field $($ind into_inner())*
//        }
//    }
//}

impl<St: Stream> Chunks<St>
where
    St: Stream
{
    pub(super) fn new(stream: St, first: usize, capacity: usize) -> Self {
        assert!(capacity > 0);

        Self {
            stream: Fuse::new(stream),
            items: Vec::with_capacity(capacity),
            is_first: true,
            first_cap: first,
            cap: capacity
        }
    }

    fn take(self: Pin<&mut Self>, cap: usize) -> Vec<St::Item> {
        mem::replace(self.project().items, Vec::with_capacity(cap))
    }

    // delegate_access_inner!(stream, St, (.));
}

impl<St: Stream> Stream for Chunks<St> {
    type Item = Vec<St::Item>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.as_mut().project();
        loop {
            match ready!(this.stream.as_mut().poll_next(cx)) {
                // Push the item into the buffer and check whether it is full.
                // If so, replace our buffer with a new and empty one and return
                // the full one.
                Some(item) => {
                    this.items.push(item);
                    let cap = if *this.is_first {
                        *this.first_cap
                    } else {
                        *this.cap
                    };
                    if this.items.len() >= cap {
                        *this.is_first = false;
                        return Poll::Ready(Some(self.take(cap)));
                    }
                }

                // Since the underlying stream ran out of values, return what we
                // have buffered, if we have anything.
                None => {
                    let last = if this.items.is_empty() {
                        None
                    } else {
                        let full_buf = mem::take(this.items);
                        Some(full_buf)
                    };

                    return Poll::Ready(last);
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let chunk_len = if self.items.is_empty() { 0 } else { 1 };
        let (lower, upper) = self.stream.size_hint();
        let lower = lower.saturating_add(chunk_len);
        let upper = match upper {
            Some(x) => x.checked_add(chunk_len),
            None => None
        };
        (lower, upper)
    }
}

impl<St: FusedStream> FusedStream for Chunks<St> {
    fn is_terminated(&self) -> bool { self.stream.is_terminated() && self.items.is_empty() }
}
