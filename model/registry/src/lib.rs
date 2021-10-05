use linearf::{converter::*, matcher::*, source::*, stream::Stream, Item, Vars};
use std::{
    any::Any,
    pin::Pin,
    sync::{Arc, Weak}
};

pub struct Source {}

impl linearf::source::SourceRegistry for Source {}

impl Source {
    pub fn new(_linearf: Weak<dyn linearf::Linearf<Registry> + Send + Sync>) -> Self
    where
        Self: Sized
    {
        Self {}
    }
}

pub struct Matcher {}

impl linearf::matcher::MatcherRegistry for Matcher {}

impl Matcher {
    pub fn new(_linearf: Weak<dyn linearf::Linearf<Registry> + Send + Sync>) -> Self
    where
        Self: Sized
    {
        Self {}
    }
}

pub struct Converter {}

impl linearf::converter::ConverterRegistry for Converter {}

impl Converter {
    pub fn new(_linearf: Weak<dyn linearf::Linearf<Registry> + Send + Sync>) -> Self
    where
        Self: Sized
    {
        Self {}
    }
}

pub struct Registry {
    pub source: Source,
    pub matcher: Matcher,
    pub converter: Converter
}

impl linearf::source::SourceRegistry for Registry {
    fn names(&self) -> &[String] { self.source.names() }

    fn parse<'de, D>(
        &self,
        name: &str,
        deserializer: D
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, D::Error>
    where
        D: serde::de::Deserializer<'de>
    {
        self.source.parse(name, deserializer)
    }

    fn reusable(
        &self,
        name: &str,
        ctx: ReusableContext,
        prev: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>),
        senario: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>)
    ) -> bool {
        self.source.reusable(name, ctx, prev, senario)
    }

    fn stream(
        &self,
        name: &str,
        senario: (Arc<Vars>, Arc<dyn Any + Send + Sync>)
    ) -> Pin<Box<dyn Stream<Item = Item>>> {
        self.source.stream(name, senario)
    }
}

impl linearf::converter::ConverterRegistry for Registry {
    fn names(&self) -> &[String] { self.converter.names() }

    fn map_convert(
        &self,
        senario: Arc<linearf::Vars>,
        items: impl linearf::stream::Stream<Item = linearf::Item> + Send + 'static
    ) -> Result<
        Pin<Box<dyn linearf::stream::Stream<Item = linearf::Item>>>,
        linearf::converter::MapConvertError
    > {
        self.converter.map_convert(senario, items)
    }
}

impl linearf::matcher::MatcherRegistry for Registry {
    fn names(&self) -> &[String] { self.matcher.names() }

    fn parse<'de, D>(
        &self,
        name: &str,
        deserializer: D
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, D::Error>
    where
        D: serde::de::Deserializer<'de>
    {
        self.matcher.parse(name, deserializer)
    }

    fn reusable(
        &self,
        name: &str,
        ctx: ReusableContext,
        prev: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>),
        senario: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>)
    ) -> bool {
        self.matcher.reusable(name, ctx, prev, senario)
    }

    fn score<'a>(
        &self,
        name: &str,
        senario: (Arc<Vars>, Arc<dyn Any + Send + Sync>),
        items: impl Stream<Item = Arc<Item>> + Send + 'static
    ) -> Pin<Box<dyn Stream<Item = WithScore>>> {
        self.matcher.score(name, senario, items)
    }
}

impl linearf::Registry for Registry {}
