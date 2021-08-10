pub mod extern_ffi {
    pub fn hello_world() -> String { "Hello World!".to_owned() }
}

include!(concat!(env!("OUT_DIR"), "/ffi.rs"));
