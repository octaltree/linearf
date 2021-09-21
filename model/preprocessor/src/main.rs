use preprocessor::{format_cargo_toml, format_lib, Recipe, StdResult};
use std::{
    env, fs,
    path::{Path, PathBuf}
};

fn main() -> StdResult<()> {
    let env_recipe = env::var("LINEARF_RECIPE");
    let env_dir = env!("CARGO_MANIFEST_DIR");
    let recipe = input(env_recipe)?;
    let (cargo_toml, lib) = dest(env_dir);
    fs::write(lib, format_lib(&recipe))?;
    fs::write(cargo_toml, format_cargo_toml(&recipe)?)?;
    Ok(())
}

fn input(env_reg: Result<String, env::VarError>) -> StdResult<Recipe> {
    match env_reg {
        Ok(s) => Ok(serde_json::from_str(&s)?),
        Err(env::VarError::NotPresent) => Ok(Recipe::default()),
        Err(e) => Err(e.into())
    }
}

fn dest(env_dir: &str) -> (PathBuf, PathBuf) {
    let here = Path::new(env_dir);
    let registry = here.parent().unwrap().join("registry");
    let cargo_toml = registry.join("Cargo.toml");
    let lib = registry.join("src").join("lib.rs");
    (cargo_toml, lib)
}
