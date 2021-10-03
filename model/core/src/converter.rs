use crate::{stream::Stream, AsyncRt, Item, New, Shared, State, Vars};
use std::{pin::Pin, sync::Arc};

/// has no `reusable` and should have referential transparency
pub trait SimpleConverter: New {
    fn into_converter(self) -> Converter
    where
        Self: Sized + 'static + Send + Sync
    {
        Converter::Simple(Arc::new(self))
    }

    fn convert(&self, item: Item) -> Item;
}

#[derive(Clone)]
pub enum Converter {
    Simple(Arc<dyn SimpleConverter + Send + Sync>)
}

pub trait ConverterRegistry {
    fn new(state: Shared<State>, rt: AsyncRt) -> Self
    where
        Self: Sized;

    fn map_convert(
        &self,
        senario: Arc<Vars>,
        items: impl Stream<Item = Item> + Send + 'static
    ) -> Result<Pin<Box<dyn Stream<Item = Item>>>, MapConvertError> {
        if senario.converters.is_empty() {
            Ok(Box::pin(items))
        } else {
            Err(MapConvertError::ConverterNotFound)
        }
    }
}

pub enum MapConvertError {
    ConverterNotFound
}
