use std::ffi::{CStr, CString};

use libc::c_char;

#[no_mangle]
extern "cdecl" fn base64_encode(x: *const c_char) -> *const c_char {
    let c_str: &CStr = unsafe { CStr::from_ptr(x) };
    let bytes: &[u8] = c_str.to_bytes();
    //// TODO: free
    let s = base64::encode(bytes);
    let cstring = unsafe { CString::from_vec_unchecked(s.into_bytes()) };
    cstring.into_raw()
}

#[no_mangle]
extern "cdecl" fn base64_decode(x: *const c_char) -> *const c_char {
    let c_str: &CStr = unsafe { CStr::from_ptr(x) };
    let bytes: &[u8] = c_str.to_bytes();
    //// TODO: free
    let b = base64::decode(bytes).expect("Failed to decode base64");
    let cstring = unsafe { CString::from_vec_unchecked(b) };
    cstring.into_raw()
}

// TODO
// fn free
