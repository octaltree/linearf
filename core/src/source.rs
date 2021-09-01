use crate::State;
use std::sync::Weak;
use tokio::sync::RwLock;

/// NOTE: Source and Match have the potential to have cache, so make them live longer.
#[async_trait]
pub trait Source: std::fmt::Debug + Send + Sync {
    fn new() -> Self
    where
        Self: Sized;
    async fn start(&mut self);
}

#[derive(Debug)]
struct Sources {
    state: Weak<RwLock<State>>
}

#[async_trait]
impl Source for Sources {
    fn new() -> Self
    where
        Self: Sized
    {
        unimplemented!()
    }

    async fn start(&mut self) { todo!() }
}
