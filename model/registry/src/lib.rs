pub struct Source {
    state: linearf::Shared<linearf::State>
}

impl<'de, D> linearf::SourceRegistry<'de, D> for Source
where
    D: serde::de::Deserializer<'de>
{
    fn new(state: linearf::Shared<linearf::State>) -> Self
    where
        Self: Sized
    {
        Self { state }
    }

    fn parse(
        &self,
        name: &str,
        deserializer: D
    ) -> Result<std::sync::Arc<dyn std::any::Any + Send + Sync>, D::Error> {
        todo!()
    }
}
