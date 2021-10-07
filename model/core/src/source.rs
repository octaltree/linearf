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
        _senario: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>)
    ) -> Pin<Box<dyn Stream<Item = Item> + Send + Sync>> {
        Box::pin(empty())
    }

    fn reusable(
        &self,
        _name: &str,
        _prev: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>),
        _senario: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>)
    ) -> Reusable {
        Reusable::Same
    }
}

#[derive(Clone)]
pub enum Source<L, P> {
    // Simple has no additional context: we may need information based on
    // the locking state and stack variables
    Simple(Arc<dyn SimpleGenerator<L, P> + Send + Sync>)
}

pub trait SimpleGenerator<L, P>: IsSource<Params = P>
where
    L: Linearf + Send + Sync
{
    fn stream(
        &self,
        senario: (&Arc<Vars>, &Arc<P>)
    ) -> Pin<Box<dyn Stream<Item = Item> + Send + Sync>>;

    /// It will be called for every flow and may be reused across sessions.
    fn reusable(&self, prev: (&Arc<Vars>, &Arc<P>), senario: (&Arc<Vars>, &Arc<P>)) -> Reusable;
}

pub trait IsSource {
    type Params: SourceParams;
}

pub trait SourceParams: DeserializeOwned + Serialize {}

impl SourceParams for BlankParams {}
