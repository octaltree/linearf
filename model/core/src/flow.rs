use serde_json::{Map, Value};
use std::sync::Arc;

/// Setting sources and matches
/// Cache may be used when equal
#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Flow {
    pub source: String,
    pub matcher: String
}

#[derive(Debug)]
pub struct Snapshot {
    pub query: Query,
    pub value: Map<String, Value>
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Query {
    NotSpecified,
    S(Arc<String>)
}
