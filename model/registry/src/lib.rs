pub struct Source {}

impl<'de, D> linearf::SourceRegistry<'de, D> for Source
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
