pub struct Source {}

#[async_trait::async_trait]
impl<'de, D> linearf::source::SourceRegistry<'de, D> for Source
where
    D: serde::de::Deserializer<'de>
{
    fn new(_state: linearf::Shared<linearf::State>) -> Self
    where
        Self: Sized
    {
        Self {}
    }
}

pub struct Matcher {}

#[async_trait::async_trait]
impl<'de, D> linearf::matcher::MatcherRegistry<'de, D> for Matcher
where
    D: serde::de::Deserializer<'de>
{
    fn new(_state: linearf::Shared<linearf::State>) -> Self
    where
        Self: Sized
    {
        Self {}
    }
}
