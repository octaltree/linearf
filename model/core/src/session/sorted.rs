use crate::{
    session::{Receiver, Score},
    AsyncRt, Item
};
use std::sync::Arc;

// TODO: improve algorithm
pub struct Sorted {
    rt: AsyncRt
}

impl Sorted {
    pub(crate) fn new(rt: AsyncRt) -> Self { Self { rt } }

    pub(crate) fn start(&mut self, rx: Receiver<(Arc<Item>, Score)>) {}

    pub(crate) fn is_done(&self) -> bool { todo!() }

    pub(crate) fn count(&self) -> u32 { todo!() }

    pub(crate) fn get_range(&self, start: usize, end: usize) -> impl Iterator<Item = Arc<Item>> {
        vec![].into_iter()
    }

    pub(crate) fn get_all(&self) -> impl Iterator<Item = Arc<Item>> { vec![].into_iter() }
}
