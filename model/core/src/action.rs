pub use super::common_interface::*;

pub trait ActionRegistry {
    // TODO: あとでなおす
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

    fn run(&self, _name: &str, _params: Arc<dyn Any + Send + Sync>) -> Arc<dyn Any + Send + Sync> {
        Arc::new(())
    }
}

pub enum Action<P, R> {
    Simple(Arc<dyn SimpleTask<Params = P, Result = R> + Send + Sync>)
}

impl<P, R> Clone for Action<P, R> {
    fn clone(&self) -> Self {
        match self {
            Self::Simple(x) => Self::Simple(Arc::clone(x))
        }
    }
}

pub trait SimpleTask: IsAction {
    fn run(&self, params: <Self as IsAction>::Params) -> <Self as IsAction>::Result;
}

pub trait IsAction {
    type Params: ActionParams;
    type Result: ActionResult;
}

pub trait ActionParams: DeserializeOwned + Serialize {}
pub trait ActionResult: DeserializeOwned + Serialize {}

impl ActionParams for BlankParams {}
impl ActionResult for BlankResult {}

pub trait NewAction<L>: IsAction
where
    L: Linearf + Send + Sync + 'static
{
    fn new(linearf: Weak<L>) -> Action<<Self as IsAction>::Params, <Self as IsAction>::Result>;
}

impl<P, R> Action<P, R> {
    pub fn from_simple<T>(x: T) -> Self
    where
        T: SimpleTask<Params = P, Result = R> + Send + Sync + 'static
    {
        Action::Simple(Arc::new(x))
    }
}
