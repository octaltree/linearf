use linearf::{converter::*, matcher::*, source::*};

pub struct Source<L> {
    _linearf: Weak<L>
}

impl<L> SourceRegistry for Source<L> {}

impl<L> Source<L>
where
    L: Linearf + Send + Sync
{
    pub fn new(linearf: Weak<L>) -> Self { Self { _linearf: linearf } }
}

pub struct Matcher<L> {
    _linearf: Weak<L>
}

impl<L> MatcherRegistry for Matcher<L> {}

impl<L> Matcher<L>
where
    L: Linearf + Send + Sync
{
    pub fn new(linearf: Weak<L>) -> Self { Self { _linearf: linearf } }
}

pub struct Converter<L> {
    _linearf: Weak<L>
}

impl<L> ConverterRegistry for Converter<L> {}

impl<L> Converter<L>
where
    L: Linearf + Send + Sync
{
    pub fn new(linearf: Weak<L>) -> Self { Self { _linearf: linearf } }
}
