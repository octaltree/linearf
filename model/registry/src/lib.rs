use linearf::{converter::*, matcher::*, source::*};

pub struct Source<L> {
    _linearf: Weak<L>
}

impl<L> SourceRegistry for Source<L> {}

impl<L> New<L> for Source<L>
where
    L: Linearf + Send + Sync
{
    fn new(linearf: Weak<L>) -> Self
    where
        Self: Sized
    {
        Self { _linearf: linearf }
    }
}

pub struct Matcher<L> {
    _linearf: Weak<L>
}

impl<L> MatcherRegistry for Matcher<L> {}

impl<L> New<L> for Matcher<L>
where
    L: Linearf + Send + Sync
{
    fn new(linearf: Weak<L>) -> Self
    where
        Self: Sized
    {
        Self { _linearf: linearf }
    }
}

pub struct Converter<L> {
    _linearf: Weak<L>
}

impl<L> ConverterRegistry for Converter<L> {}

impl<L> New<L> for Converter<L>
where
    L: Linearf + Send + Sync
{
    fn new(linearf: Weak<L>) -> Self
    where
        Self: Sized
    {
        Self { _linearf: linearf }
    }
}
