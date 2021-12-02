pub use super::common_interface::*;

pub trait SourceRegistry {
    fn names(&self) -> &[SmartString] { &[] }

    fn parse<'de, D>(
        &self,
        _name: &str,
        _deserializer: D
    ) -> Option<Result<Arc<dyn Any + Send + Sync>, D::Error>>
    where
        D: serde::de::Deserializer<'de>
    {
        None
    }

    fn stream(
        &self,
        _name: &str,
        _scenario: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>)
    ) -> Pin<Box<dyn Stream<Item = Item> + Send + Sync>> {
        Box::pin(empty())
    }

    fn reusable(
        &self,
        _name: &str,
        _prev: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>),
        _scenario: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>)
    ) -> Reusable {
        Reusable::Same
    }
}

pub enum Source<P> {
    // Simple has no additional context: we may need information based on
    // the locking state and stack variables
    Simple(Arc<dyn SimpleGenerator<Params = P> + Send + Sync>)
}

impl<P> Clone for Source<P> {
    fn clone(&self) -> Self {
        match self {
            Self::Simple(x) => Self::Simple(Arc::clone(x))
        }
    }
}

pub trait SimpleGenerator: IsSource {
    fn stream(
        &self,
        scenario: (&Arc<Vars>, &Arc<<Self as IsSource>::Params>)
    ) -> Pin<Box<dyn Stream<Item = Item> + Send + Sync>>;

    /// It will be called for every flow and may be reused across sessions.
    fn reusable(
        &self,
        prev: (&Arc<Vars>, &Arc<<Self as IsSource>::Params>),
        scenario: (&Arc<Vars>, &Arc<<Self as IsSource>::Params>)
    ) -> Reusable;
}

pub trait IsSource {
    type Params: SourceParams;
}

pub trait SourceParams: DeserializeOwned + Serialize {}

impl SourceParams for BlankParams {}

pub trait NewSource<L>: IsSource
where
    L: Linearf + Send + Sync + 'static
{
    fn new(linearf: Weak<L>) -> Source<<Self as IsSource>::Params>;
}

impl<P> Source<P> {
    pub fn from_simple<T>(x: T) -> Self
    where
        T: SimpleGenerator<Params = P> + Send + Sync + 'static
    {
        Source::Simple(Arc::new(x))
    }
}
