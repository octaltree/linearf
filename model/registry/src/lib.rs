pub struct Source {}

impl<'de, D> linearf::source::SourceRegistry<'de, D> for Source
where
    D: serde::de::Deserializer<'de>
{
    fn new(_state: linearf::Shared<linearf::State>, _rt: linearf::AsyncRt) -> Self
    where
        Self: Sized
    {
        Self {}
    }
}

pub struct Matcher {}

impl<'de, D> linearf::matcher::MatcherRegistry<'de, D> for Matcher
where
    D: serde::de::Deserializer<'de>
{
    fn new(_state: linearf::Shared<linearf::State>, _rt: linearf::AsyncRt) -> Self
    where
        Self: Sized
    {
        Self {}
    }
}

pub struct Converter {}

impl linearf::converter::ConverterRegistry for Converter {
    fn new(_state: linearf::Shared<linearf::State>, _rt: linearf::AsyncRt) -> Self
    where
        Self: Sized
    {
        Self {}
    }
}
