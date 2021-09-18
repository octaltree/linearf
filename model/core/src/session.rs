use std::{any::Any, sync::Arc};
use tokio::sync::mpsc;

pub(crate) type Sender<T> = mpsc::UnboundedSender<T>;
pub(crate) type Receiver<T> = mpsc::UnboundedReceiver<T>;

pub struct Vars {}

pub struct Session {
    vars: Arc<Vars>,
    source_params: Arc<dyn Any>,
    _matcher_params: Arc<dyn Any>
}
