mod score;

pub use super::common_interface::*;
pub use score::Score;

pub trait MatcherRegistry {
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

    fn score(
        &self,
        _name: &str,
        _senario: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>),
        items: impl Stream<Item = Arc<Item>> + Send + Sync + 'static
    ) -> Pin<Box<dyn Stream<Item = WithScore> + Send + Sync>> {
        Box::pin(items.map(|i| {
            let score = Arc::new(Score::new_excluded());
            (i, score)
        }))
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

pub type WithScore = (Arc<Item>, Arc<Score>);

pub enum Matcher<P> {
    Simple(Arc<dyn SimpleScorer<Params = P> + Send + Sync>)
}

impl<P> Clone for Matcher<P> {
    fn clone(&self) -> Self {
        match self {
            Self::Simple(x) => Self::Simple(Arc::clone(x))
        }
    }
}

pub trait SimpleScorer: IsMatcher {
    fn score(
        &self,
        senario: (&Arc<Vars>, &Arc<<Self as IsMatcher>::Params>),
        item: &Arc<Item>
    ) -> Score;

    /// It will be called for every flow and may be reused across sessions.
    fn reusable(
        &self,
        prev: (&Arc<Vars>, &Arc<<Self as IsMatcher>::Params>),
        senario: (&Arc<Vars>, &Arc<<Self as IsMatcher>::Params>)
    ) -> Reusable;
}

pub trait IsMatcher {
    type Params: MatcherParams;
}

pub trait MatcherParams: DeserializeOwned + Serialize {}

impl MatcherParams for BlankParams {}

pub trait NewMatcher<L>: IsMatcher
where
    L: Linearf + Send + Sync + 'static
{
    fn new(linearf: Weak<L>) -> Matcher<<Self as IsMatcher>::Params>;
}

impl<P> Matcher<P> {
    pub fn from_simple<T>(x: T) -> Self
    where
        T: SimpleScorer<Params = P> + Send + Sync + 'static
    {
        Matcher::Simple(Arc::new(x))
    }
}
