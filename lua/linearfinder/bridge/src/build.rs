use std::{env, fs::File, io::Write, path::Path};

fn main() {
    let rust_output = Path::new(&env::var("OUT_DIR").unwrap()).join("ffi.rs");

    let output = generator::generate(
        &env::current_dir().unwrap().as_path().join("src/lib.rs"),
        "fuzzy_filter_lua_ffi",
        false
    );

    File::create(rust_output.clone())
        .unwrap()
        .write_all(output.as_bytes())
        .unwrap();

    assert!(rust_output.exists());
}
