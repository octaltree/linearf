use linearf::Shared;
pub struct Source {
    _state: linearf::Shared<linearf::State>
}
impl<'de, D> linearf::SourceRegistry<'de, D> for Source
where
    D: serde::de::Deserializer<'de>
{
    fn new(state: linearf::Shared<linearf::State>) -> Self
    where
        Self: Sized
    {
        Self { _state: state }
    }
    fn parse(
        &self,
        _name: &str,
        _deserializer: D
    ) -> Result<Option<std::sync::Arc<dyn std::any::Any + Send + Sync>>, D::Error> {
        Ok(None)
    }
}
