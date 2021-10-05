use crate::{stream::Stream, AsyncRt, Item, New, Shared, State, Vars};
use std::{pin::Pin, sync::Arc};

/// has no `reusable` and should have referential transparency
pub trait SimpleConverter<L, R>: New<L, R>
where
    L: crate::Linearf<R> + Send + Sync,
    R: crate::Registry
{
    fn into_converter(self) -> Converter<L, R>
    where
        Self: Sized + 'static + Send + Sync
    {
        Converter::Simple(Arc::new(self))
    }

    fn convert(&self, item: Item) -> Item;
}

#[derive(Clone)]
pub enum Converter<L, R> {
    Simple(Arc<dyn SimpleConverter<L, R> + Send + Sync>)
}

pub trait ConverterRegistry {
    fn names(&self) -> &[String] { &[] }

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
