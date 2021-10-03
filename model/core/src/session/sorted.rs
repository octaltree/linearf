use crate::{
    matcher,
    session::{Receiver, Score},
    AsyncRt, Item, Shared
};
use std::{collections::VecDeque, convert::TryInto, sync::Arc};

type WithScore = (Arc<Item>, Score);

// TODO: improve algorithm
pub struct Sorted {
    rt: AsyncRt,
    inner: Shared<(bool, Vec<WithScore>)>
}

impl Sorted {
    pub(crate) fn new(rt: AsyncRt) -> Self {
        Self {
            rt,
            inner: Default::default()
        }
    }

    pub(crate) fn start(&self, mut rx: Receiver<matcher::Output>) {
        let inner = self.inner.clone();
        let chunk_size = 100000;
        self.rt.spawn(async move {
            let start = std::time::Instant::now();
            loop {
                println!("chunk");
                let mut chunk = VecDeque::with_capacity(chunk_size);
                let done = loop {
                    if chunk.len() >= chunk_size {
                        break false;
                    }
                    // TODO: channel is none if buffer is empty
                    match rx.recv().await {
                        Some(matcher::Output::Item(t)) => {
                            let insert = std::time::Instant::now();
                            try_insert(&mut chunk, t);
                            log::debug!("insert {:?}", std::time::Instant::now() - insert);
                        }
                        Some(matcher::Output::Chunk(ts)) => {
                            for t in ts {
                                try_insert(&mut chunk, t);
                            }
                        }
                        None => break true
                    }
                };
                Self::merge(&inner, chunk).await;
                log::debug!("{}", inner.read().await.1.len());
                if done {
                    inner.write().await.0 = true;
                    break;
                }
            }
            log::debug!("sorted {:?}", std::time::Instant::now() - start);
            log::debug!("{}", inner.read().await.1.len());
        });
    }

    async fn merge(inner: &Shared<(bool, Vec<WithScore>)>, chunk: VecDeque<WithScore>) {
        let (_, xs) = &mut *inner.write().await;
        println!("{} {}", xs.len(), chunk.len());
        let mut left = 0;
        for x in chunk {
            let ys = &xs[left..];
            let idx = match ys.binary_search_by_key(&&x.1, |y| &y.1) {
                Ok(i) => left + i,
                Err(i) => left + i
            };
            left = idx;
            xs.insert(idx, x);
        }
    }

    pub(crate) async fn is_done(&self) -> bool { self.inner.read().await.0 }

    pub(crate) async fn count(&self) -> u32 { self.inner.read().await.1.len().try_into().unwrap() }

    pub(crate) async fn get_range(
        &self,
        start: usize,
        end: usize
    ) -> impl Iterator<Item = Arc<Item>> {
        let xs = &self.inner.read().await.1;
        xs[start..std::cmp::min(end, xs.len())]
            .iter()
            .map(|(i, _)| i.clone())
            .collect::<Vec<_>>()
            .into_iter()
    }

    pub(crate) async fn get_all(&self) -> impl Iterator<Item = Arc<Item>> {
        let xs = &self.inner.read().await.1;
        xs.iter()
            .map(|(i, _)| i.clone())
            .collect::<Vec<_>>()
            .into_iter()
    }
}

fn try_insert(xs: &mut VecDeque<WithScore>, x: WithScore) {
    if x.1.should_be_excluded() {
        return;
    }
    let idx = match xs.binary_search_by_key(&&x.1, |y| &y.1) {
        Ok(i) => i,
        Err(i) => i
    };
    xs.insert(idx, x);
}
