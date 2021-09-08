pub use crate::session::Sender;
use crate::{Flow, Item, New, Session, Shared};
use std::{stream::Stream, sync::Arc};
use tokio::sync::mpsc;

/// Source that are not affected by query
/// NOTE: Source have the potential to have itw own cache, so make them live longer.
#[async_trait]
pub trait Generator: New + Send + Sync {
    // TODO: error notification
    async fn generate(
        &mut self,
        tx: Sender<Arc<Item>>,
        flow: &Arc<Flow>
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn reusable(&self, _prev: &Session, _flow: &Arc<Flow>) -> bool;
}

/// Results change dependening on the query
#[async_trait]
pub trait DynamicGenerator: New + Send + Sync {
    async fn start(&mut self, flow: &Arc<Flow>);

    /// tx is different for every call
    fn query(&mut self, tx: Sender<Arc<Item>>, q: &str);
}

#[derive(Clone)]
pub enum Source {
    Static(Shared<dyn Generator>),
    Dynamic(Shared<dyn DynamicGenerator>)
}

struct UnboundedStream<T> {
    rx: mpsc::UnboundedReceiver<T>
}

impl<T> UnboundedStream<T> {
    fn new() -> (mpsc::UnboundedSender<T>, Self) {
        let (tx, rx) = mpsc::unbounded_channel::<T>();
        (tx, UnboundedStream { rx })
    }
}

impl<T> Stream for UnboundedStream<T> {
    type Item = T;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.get_mut();
        this.rx.poll_recv(cx)
    }
}

// pub mod builtin {
//    use crate::{Flow, Item, Session, State};
//    use std::{stream::Stream, sync::Arc};
//    use tokio::sync::{mpsc, RwLock};

//    #[derive(Debug)]
//    pub struct StateSource {
//        state: Arc<RwLock<State>>
//    }

//    #[derive(Debug)]
//    pub struct StateSession {
//        state: Arc<RwLock<State>>
//    }

//    #[derive(Debug)]
//    pub struct StateFlow {
//        state: Arc<RwLock<State>>
//    }

//    struct UnboundedStream<T> {
//        rx: mpsc::UnboundedReceiver<T>
//    }

//    impl<T> UnboundedStream<T> {
//        fn new() -> (mpsc::UnboundedSender<T>, Self) {
//            let (tx, rx) = mpsc::unbounded_channel::<T>();
//            (tx, UnboundedStream { rx })
//        }
//    }

//    impl<T> Stream for UnboundedStream<T> {
//        type Item = T;

//        fn poll_next(
//            self: std::pin::Pin<&mut Self>,
//            cx: &mut std::task::Context<'_>
//        ) -> std::task::Poll<Option<Self::Item>> {
//            let this = self.get_mut();
//            this.rx.poll_recv(cx)
//        }
//    }

//    #[async_trait]
//    impl super::Source for StateSource {
//        fn new() -> Self
//        where
//            Self: Sized
//        {
//            unimplemented!()
//        }

//        async fn start(&mut self, flow: &Arc<Flow>) -> Box<dyn Stream<Item = Item>> {
//            let (tx, st) = UnboundedStream::new();
//            Box::new(st)
//        }

//        async fn reusable(&self, _prev: &Session, _flow: &Arc<Flow>) -> bool { false}
//    }

//    #[async_trait]
//    impl super::Source for StateSession {
//        fn new() -> Self
//        where
//            Self: Sized
//        {
//            unimplemented!()
//        }

//        async fn start(&mut self, flow: &Arc<Flow>) -> Box<dyn Stream<Item = Item>> {
//            let (tx, st) = UnboundedStream::new();
//            Box::new(st)
//        }

//        async fn reusable(&self, _prev: &Session, _flow: &Arc<Flow>) -> bool { false }
//    }

//    #[async_trait]
//    impl super::Source for StateFlow {
//        fn new() -> Self
//        where
//            Self: Sized
//        {
//            unimplemented!()
//        }

//        async fn start(&mut self, flow: &Arc<Flow>) -> Box<dyn Stream<Item = Item>> {
//            let (tx, st) = UnboundedStream::new();
//            Box::new(st)
//        }

//        async fn reusable(&self, _prev: &Session, _flow: &Arc<Flow>) -> bool { false }
//    }

//    impl StateSource {
//        pub fn new(state: Arc<RwLock<State>>) -> Self { Self { state } }
//    }

//    impl StateSession {
//        pub fn new(state: Arc<RwLock<State>>) -> Self { Self { state } }
//    }

//    impl StateFlow {
//        pub fn new(state: Arc<RwLock<State>>) -> Self { Self { state } }
//    }
//}
