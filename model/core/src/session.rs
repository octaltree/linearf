use std::{any::Any, sync::Arc};
use tokio::sync::mpsc;

pub(crate) type Sender<T> = mpsc::UnboundedSender<T>;
pub(crate) type Receiver<T> = mpsc::UnboundedReceiver<T>;

#[derive(Debug, PartialEq)]
pub struct Vars {
    source: String,
    matcher: String,
    query: String
}

pub struct Session {
    vars: Arc<Vars>,
    source_params: Arc<dyn Any + Send + Sync>,
    _matcher_params: Arc<dyn Any + Send + Sync>
}
