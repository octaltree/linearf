use serde_json::{Map, Value};

// TODO: rename request
/// Setting sources and matches
/// Cache may be used when equal
#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Flow {
    pub source: String,
    pub matcher: String
}

struct Plain {
    source: Option<String>,
    matcher: Option<String>,
    source_params: Option<Map<String, Value>>,
    matcher_params: Option<Map<String, Value>>,
    query: Option<String>
}

struct Snapshot {}

// fn merge
// struct Typed
