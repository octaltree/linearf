pub use crate::{item::Item, Linearf, SmartString, Vars};
pub use futures::{stream::empty, Stream, StreamExt};
pub use serde::{de::DeserializeOwned, Deserialize, Serialize};
pub use std::{
    any::Any,
    pin::Pin,
    sync::{Arc, Weak}
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Reusable {
    /// Not reusable
    None,
    /// This means
    /// * Not necessarily the same
    /// * Not change often
    /// * Cost is large and cache is preferred
    Cache,
    /// It depends only on the argument and is always the same.
    Same
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum BlankParams {
    Unit(()),
    Obj {}
}
