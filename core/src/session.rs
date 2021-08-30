use crate::Item;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc
};

/// State being calculated based on flow
#[derive(Debug)]
pub struct Session {
    should_stop: Arc<AtomicBool>,
    flow: Arc<Flow>,
    query: Option<String>,
    items: Vec<Item>
}

/// Setting sources and matches
/// Cache may be used when equal
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct Flow {}

impl Session {
    pub async fn start(flow: Arc<Flow>) -> Self {
        Self {
            should_stop: Arc::new(false.into()),
            flow,
            query: None,
            items: Vec::new()
        }
    }

    pub fn count(&self) -> usize { self.items.len() }

    pub fn items(&self, start: usize, stop: usize) -> Option<&[Item]> {
        let l = self.items.len();
        if start <= l && stop <= l {
            Some(&self.items[start..stop])
        } else {
            None
        }
    }

    pub fn query<S: Into<String>>(&mut self, s: S) {
        self.query = Some(s.into());
        todo!()
    }
}

impl Drop for Session {
    fn drop(&mut self) { self.should_stop.store(true, Ordering::Relaxed); }
}
