pub use serde_json::{Map, Value};
use std::{borrow::Cow, ffi::OsString};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaybeUtf8 {
    Utf8(String),
    Os(OsString),
    Bytes(Vec<u8>)
}

#[derive(Debug, PartialEq)]
pub struct Item {
    /// id must not be 0
    pub id: u32,
    pub value: MaybeUtf8,
    // json value cannot represent luastring
    pub info: Option<Map<String, Value>>,
    pub view: Option<String>,
    pub view_for_matcing: Option<String>
}
impl Item {
    pub fn new(id: u32, value: MaybeUtf8) -> Self {
        Self {
            id,
            value,
            info: None,
            view: None,
            view_for_matcing: None
        }
    }

    pub fn value_lossy(&self) -> Cow<'_, str> {
        match &self.value {
            MaybeUtf8::Utf8(s) => Cow::Borrowed(s),
            MaybeUtf8::Os(s) => s.to_string_lossy(),
            MaybeUtf8::Bytes(b) => String::from_utf8_lossy(b)
        }
    }

    #[inline]
    pub fn view(&self) -> Cow<'_, str> {
        let opt = self.view.as_deref().map(Cow::Borrowed);
        opt.unwrap_or_else(|| self.value_lossy())
    }

    #[inline]
    pub fn view_for_matcing(&self) -> Cow<'_, str> {
        let opt = self.view_for_matcing.as_deref().map(Cow::Borrowed);
        opt.unwrap_or_else(|| self.value_lossy())
    }
}
