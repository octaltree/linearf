use crate::{session::Sender, Flow, Item, New, Session, Shared};
use std::{future::Future, sync::Arc};

pub struct Transmitter {
    tx: Sender<Vec<Item>>
}

impl Transmitter {
    pub fn new(tx: Sender<Vec<Item>>) -> Self { Self { tx } }

    #[inline]
    pub fn item(&self, i: Item) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.tx.send(vec![i])?)
    }

    #[inline]
    pub fn chunk<A: Into<Vec<Item>>>(
        &self,
        xs: A
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.tx.send(xs.into())?)
    }
}

// TODO: error notification

/// Source that are not affected by query
/// NOTE: Source have the potential to have itw own cache, so make them live longer.
#[async_trait]
pub trait Generator: New + Send + Sync {
    async fn generate(
        &mut self,
        tx: Transmitter,
        flow: &Arc<Flow>
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn reusable(&self, prev: &Session, flow: &Arc<Flow>) -> bool;
}

/// Results change dependening on the query
#[async_trait]
pub trait DynamicGenerator: New + Send + Sync {
    async fn generate(
        &mut self,
        tx: Transmitter,
        flow: &Arc<Flow>,
        query: &str
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn reusable(&self, prev: &Session, flow: &Arc<Flow>, query: &str) -> bool;
}

#[derive(Clone)]
pub enum Source {
    Static(Shared<dyn Generator>),
    Dynamic(Shared<dyn DynamicGenerator>)
}

// trait S: Stream<Item = (&Request, Item)> {}

// struct SourceRunner {
//    sources: HashMap<String, Source>,
//    requests: VecDeque<Arc<Request>>
//}

// impl SourceRunner {
//    async fn start(request: mpsc::UnboundedReceiver<Arc<Request>>) {}
//}

// trait FlowGenerator {
//    async fn run(
//        &mut self,
//        request: mpsc::UnboundedReceiver<(
//            Arc<Request>,
//            Transmitter<Item = Result<Arc<Item>, Box<dyn std::error::Error + Send + Sync>>>
//        )>
//    );

//    // reusable
//}

// trait SimpleGenerator {
//    async fn generate(
//        &self,
//        tx: Transmitter<Item = Result<Arc<Item>, Box<dyn std::error::Error + Send + Sync>>>,
//        request: Arc<Request>
//    );
//    // reusable
//}

// impl Future for SourceRunner {
//    type Output = i32;

//    fn poll(
//        self: std::pin::Pin<&mut Self>,
//        cx: &mut std::task::Context<'_>
//    ) -> std::task::Poll<Self::Output> {
//        todo!()
//    }
//}
