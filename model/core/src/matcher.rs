use crate::{session::Vars, Item, New, Shared};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::{cmp::Ordering, sync::Arc};

pub trait MatcherParams: DeserializeOwned + Serialize {}

pub trait IsMatcher {
    type Params: MatcherParams;
    type Score: Score;
}

#[async_trait]
pub trait SimpleScorer<P, S>: New + IsMatcher<Params = P, Score = S> {
    async fn score(&self, senario: (&Arc<Vars>, &Arc<P>), item: &Arc<Item>) -> S;

    async fn reusable(&self, prev: (&Arc<Vars>, &Arc<P>), senario: (&Arc<Vars>, &Arc<P>)) -> bool;
}

#[derive(Clone)]
pub enum Matcher<P, S> {
    Simple(Shared<dyn SimpleScorer<P, S> + Send + Sync>)
}

pub trait Score {}
