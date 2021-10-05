use crate::{
    state::{Shared, State},
    AsyncRt,
};
use serde::{Deserialize, Serialize};

pub trait New {
    fn new(_state: &Shared<State>, _rt: &AsyncRt) -> Self
    where
        Self: Sized;
}

#[derive(Clone)]
pub struct ReusableContext {
    pub same_session: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum BlankParams {
    Unit(()),
    Obj {},
}
