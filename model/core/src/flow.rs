/// Setting sources and matches
/// Cache may be used when equal
#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Flow {
    pub source: String,
    pub matcher: String
}
