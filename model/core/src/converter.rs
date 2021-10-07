pub use super::common_interface::*;

pub trait ConverterRegistry {
    fn names(&self) -> &[SmartString] { &[] }

    fn map_convert(
        &self,
        names: &[SmartString],
        items: impl Stream<Item = Item> + Send + Sync + 'static
    ) -> Result<Pin<Box<dyn Stream<Item = Item> + Send + Sync>>, MapConvertError> {
        if names.is_empty() {
            Ok(Box::pin(items))
        } else {
            let first = names[0].clone();
            Err(MapConvertError::ConverterNotFound(first))
        }
    }
}

#[derive(Clone)]
pub enum Converter<L> {
    Simple(Arc<dyn SimpleConverter<L> + Send + Sync>)
}

pub trait SimpleConverter<L>
where
    L: Linearf + Send + Sync
{
    fn convert(&self, item: Item) -> Item;
}

pub enum MapConvertError {
    ConverterNotFound(SmartString)
}
