use serde_json::{Map, Value};
use std::{borrow::Cow, ffi::OsString};

// TODO: into CStr
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaybeUtf8 {
    Utf8(String),
    Os(OsString),
    Bytes(Vec<u8>)
}

#[derive(Debug)]
pub struct Item {
    /// id must not be 0
    pub id: u32,
    pub r#type: &'static str,
    pub value: MaybeUtf8,
    pub info: Option<Map<String, Value>>,
    pub view: Option<String>,
    pub view_for_matcing: Option<String>
}

impl Item {
    pub fn new(id: u32, r#type: &'static str, value: MaybeUtf8) -> Self {
        Self {
            id,
            r#type,
            value,
            info: None,
            view: None,
            view_for_matcing: None
        }
    }

    #[inline]
    pub fn view(&self) -> Cow<'_, str> {
        let opt = self.view.as_deref().map(Cow::Borrowed);
        opt.unwrap_or_else(|| match &self.value {
            MaybeUtf8::Utf8(s) => Cow::Borrowed(s),
            MaybeUtf8::Os(s) => match s.to_string_lossy() {
                Cow::Owned(s) => Cow::Owned(s),
                Cow::Borrowed(s) => Cow::Borrowed(s)
            },
            MaybeUtf8::Bytes(b) => match String::from_utf8_lossy(b) {
                Cow::Owned(s) => Cow::Owned(s),
                Cow::Borrowed(s) => Cow::Borrowed(s)
            }
        })
    }

    #[inline]
    pub fn view_for_matcing(&self) -> Cow<'_, str> {
        let opt = self.view_for_matcing.as_deref().map(Cow::Borrowed);
        opt.unwrap_or_else(|| self.view())
    }
}
