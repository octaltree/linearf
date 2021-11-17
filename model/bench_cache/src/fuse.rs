use core::pin::Pin;
use futures::{
    ready,
    stream::{FusedStream, Stream},
    task::{Context, Poll}
};
use pin_project_lite::pin_project;

pin_project! {
    /// Stream for the [`fuse`](super::StreamExt::fuse) method.
    #[derive(Debug)]
    #[must_use = "streams do nothing unless polled"]
    pub struct Fuse<St> {
        #[pin]
        stream: St,
        done: bool,
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

impl<St> Fuse<St> {
    pub(super) fn new(stream: St) -> Self {
        Self {
            stream,
            done: false
        }
    }

    ///// Returns whether the underlying stream has finished or not.
    /////
    ///// If this method returns `true`, then all future calls to poll are
    ///// guaranteed to return `None`. If this returns `false`, then the
    ///// underlying stream is still in use.
    // pub fn is_done(&self) -> bool { self.done }

    // delegate_access_inner!(stream, St, ());
}

impl<S: Stream> FusedStream for Fuse<S> {
    fn is_terminated(&self) -> bool { self.done }
}

impl<S: Stream> Stream for Fuse<S> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<S::Item>> {
        let this = self.project();

        if *this.done {
            return Poll::Ready(None);
        }

        let item = ready!(this.stream.poll_next(cx));
        if item.is_none() {
            *this.done = true;
        }
        Poll::Ready(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.done {
            (0, Some(0))
        } else {
            self.stream.size_hint()
        }
    }
}
