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

mod registry;
pub use registry::Registry;
