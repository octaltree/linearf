use crate::{session::Sender, AsyncRt, Flow, Item, Session, Shared, State};
use std::{stream::Stream, sync::Arc};

/// Source that are not affected by query
/// NOTE: Source have the potential to have itw own cache, so make them live longer.
#[async_trait]
pub trait Generator: std::fmt::Debug + Send + Sync {
    fn new(_state: &Shared<State>, _rt: &AsyncRt) -> Self
    where
        Self: Sized;

    async fn generate(
        &mut self,
        tx: Sender<Item>,
        flow: &Arc<Flow>
    ) -> Box<dyn Stream<Item = Item>>;

    async fn reusable(&self, _prev: &Session, _flow: &Arc<Flow>) -> bool;
}

/// Results change dependening on the query
#[async_trait]
pub trait DynamicGenerator: std::fmt::Debug + Send + Sync {
    fn new(_state: &Shared<State>, _rt: &AsyncRt) -> Self
    where
        Self: Sized;

    async fn start(&mut self, flow: &Arc<Flow>);

    fn query(&mut self, q: &str) -> Box<dyn Stream<Item = Item>>;
}

pub fn new_source_static<G: 'static + Generator>(state: &Shared<State>, rt: &AsyncRt) -> Source {
    Source::Static(Arc::new(G::new(&state, &rt)))
}

pub fn new_source_dynamic<G: 'static + DynamicGenerator>(
    state: &Shared<State>,
    rt: &AsyncRt
) -> Source {
    Source::Dynamic(Arc::new(G::new(&state, &rt)))
}

#[derive(Debug)]
pub enum Source {
    Static(Arc<dyn Generator>),
    Dynamic(Arc<dyn DynamicGenerator>)
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
