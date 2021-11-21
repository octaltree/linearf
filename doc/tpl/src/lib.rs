pub mod linearf_txt;

use std::{ffi::OsStr, path::Path};

pub const LINEARF_ROOT: &'static Path = as_path("../../");

const fn as_path(s: &'static str) -> &'static Path {
    unsafe { &*(s as *const str as *const OsStr as *const Path) }
}
