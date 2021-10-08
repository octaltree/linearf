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

pub enum MapConvertError {
    ConverterNotFound(SmartString)
}

pub enum Converter<P> {
    Simple(Arc<dyn SimpleConverter + Send + Sync>),
    Reserve(std::marker::PhantomData<P>)
}

impl<P> Clone for Converter<P> {
    fn clone(&self) -> Self {
        match self {
            Self::Simple(x) => Converter::Simple(Arc::clone(x)),
            &Self::Reserve(x) => Converter::Reserve(x)
        }
    }
}

pub trait SimpleConverter {
    fn convert(&self, item: Item) -> Item;
}

pub trait IsConverter {
    type Params: ConverterParams;
}

pub trait ConverterParams: DeserializeOwned + Serialize {}

impl ConverterParams for BlankParams {}

#[derive(Deserialize, Serialize)]
pub enum Void {}

impl ConverterParams for Void {}

pub trait NewConverter<L>: IsConverter
where
    L: Linearf + Send + Sync + 'static
{
    fn new(linearf: Weak<L>) -> Converter<<Self as IsConverter>::Params>;
}

impl<P> Converter<P> {
    pub fn from_simple<T>(x: T) -> Self
    where
        T: SimpleConverter + Send + Sync + 'static
    {
        Converter::Simple(Arc::new(x))
    }
}
