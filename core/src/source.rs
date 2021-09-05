/// NOTE: Source and Match have the potential to have cache, so make them live longer.
// TODO: dynamic source: results change depending on the query
#[async_trait]
pub trait Source: std::fmt::Debug + Send + Sync {
    fn new() -> Self
    where
        Self: Sized;
    async fn start(&mut self);
}

pub mod builtin {
    use crate::State;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[derive(Debug)]
    pub struct Source {
        state: Arc<RwLock<State>>
    }

    #[derive(Debug)]
    pub struct Session {
        state: Arc<RwLock<State>>
    }

    #[derive(Debug)]
    pub struct Flow {
        state: Arc<RwLock<State>>
    }

    #[async_trait]
    impl super::Source for Source {
        fn new() -> Self
        where
            Self: Sized
        {
            unimplemented!()
        }

        async fn start(&mut self) { todo!() }
    }

    #[async_trait]
    impl super::Source for Session {
        fn new() -> Self
        where
            Self: Sized
        {
            unimplemented!()
        }

        async fn start(&mut self) { todo!() }
    }

    #[async_trait]
    impl super::Source for Flow {
        fn new() -> Self
        where
            Self: Sized
        {
            unimplemented!()
        }

        async fn start(&mut self) { todo!() }
    }

    impl Source {
        pub fn new(state: Arc<RwLock<State>>) -> Self { Self { state } }
    }

    impl Session {
        pub fn new(state: Arc<RwLock<State>>) -> Self { Self { state } }
    }

    impl Flow {
        pub fn new(state: Arc<RwLock<State>>) -> Self { Self { state } }
    }
}
