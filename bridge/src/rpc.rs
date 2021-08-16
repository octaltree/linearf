use std::{any::Any, collections::VecDeque, sync::Arc};
use tokio::sync::RwLock;

//#[derive(Default)]
// struct C {
//    id: i32,
//    requests: VecDeque<(i32, Box<dyn Request<Response = Box<dyn Any>>>)>,
//    requests
//    responses: VecDeque<(i32, Box<dyn Any>)>
//}

// pub struct Connection(Arc<RwLock<C>>);

// trait Request {
//    type Response;
//}

// impl Connection {
//    pub fn new() -> Self { Self(Arc::new(RwLock::new(C::default()))) }

//    pub async fn request<R>(&self, req: R) -> R::Response {
//        {
//            let mut c = self.0.write().await;
//            c.id += 1;
//            c.requests.push_back((c.id, req));
//        }
//        todo!()
//    }
//}
