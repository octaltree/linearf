pub(crate) mod common_interface;
pub mod converter;
pub mod item;
pub mod matcher;
pub mod source;
pub mod state;

pub use converter::ConverterRegistry;
pub use matcher::MatcherRegistry;
pub use smartstring::alias::String as SmartString;
pub use source::SourceRegistry;
pub use state::{Shared, State};

use serde::{Deserialize, Serialize};

pub trait Linearf {
    type Source: SourceRegistry;
    type Matcher: MatcherRegistry;
    type Converter: ConverterRegistry;

    fn state(&self) -> &Shared<State>;

    fn runtime(&self) -> &AsyncRt;

    fn source(&self) -> &Self::Source;

    fn matcher(&self) -> &Self::Matcher;

    fn converter(&self) -> &Self::Converter;
}

pub type AsyncRt = tokio::runtime::Handle;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Vars {
    pub source: SmartString,
    pub matcher: SmartString,
    pub converters: Vec<SmartString>,
    pub query: String,
    /// How many seconds before you can reuse it
    pub cache_sec: u32,
    /// Whether to reuse flows from different sessions
    pub cache_across_sessions: bool,
    pub chunk_size: usize
}
