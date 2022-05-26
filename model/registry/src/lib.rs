use linearf::{action::*, converter::*, matcher::*, source::*};

pub struct Source<L> {
    _linearf: Weak<L>
}

impl<L> SourceRegistry for Source<L> {}

impl<L> Source<L>
where
    L: linearf::Linearf + Send + Sync + 'static
{
    pub fn new(linearf: Weak<L>) -> Self { Self { _linearf: linearf } }
}

pub struct Matcher<L> {
    _linearf: Weak<L>
}

impl<L> MatcherRegistry for Matcher<L> {}

impl<L> Matcher<L>
where
    L: linearf::Linearf + Send + Sync + 'static
{
    pub fn new(linearf: Weak<L>) -> Self { Self { _linearf: linearf } }
}

pub struct Converter<L> {
    _linearf: Weak<L>
}

impl<L> ConverterRegistry for Converter<L> {}

impl<L> Converter<L>
where
    L: linearf::Linearf + Send + Sync + 'static
{
    pub fn new(linearf: Weak<L>) -> Self { Self { _linearf: linearf } }
}

pub struct Action<L> {
    _linearf: Weak<L>
}

impl<L> ActionRegistry for Action<L> {}

impl<L> Action<L>
where
    L: linearf::Linearf + Send + Sync + 'static
{
    pub fn new(linearf: Weak<L>) -> Self { Self { _linearf: linearf } }
}
