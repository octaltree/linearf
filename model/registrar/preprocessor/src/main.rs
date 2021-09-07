use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    error::Error as StdError,
    fs,
    path::{Path, PathBuf}
};

type StdResult<T> = Result<T, Box<dyn StdError>>;

fn main() -> StdResult<()> {
    let env_reg = env::var("LINEARF_REG");
    let env_dir = env!("CARGO_MANIFEST_DIR");
    let crates = input(env_reg)?;
    let (cargo_toml, main) = dest(env_dir);
    fs::write(main, format_main(&crates))?;
    fs::write(cargo_toml, format_cargo_toml(&crates)?)?;
    Ok(())
}

fn input(env_reg: Result<String, env::VarError>) -> StdResult<Vec<Crate>> {
    match env_reg {
        Ok(s) => Ok(serde_json::from_str(&s)?),
        Err(env::VarError::NotPresent) => Ok(Vec::new()),
        Err(e) => Err(e.into())
    }
}

fn dest(env_dir: &str) -> (PathBuf, PathBuf) {
    let here = Path::new(env_dir);
    let registrar = here.parent().unwrap().join("registrar");
    let cargo_toml = registrar.join("Cargo.toml");
    let main = registrar.join("src").join("main.rs");
    (cargo_toml, main)
}

fn format_cargo_toml(crates: &[Crate]) -> StdResult<String> {
    #[derive(Serialize)]
    struct Manifest {
        package: CargoPackage,
        dependencies: HashMap<String, serde_json::Value>
    }
    #[derive(Serialize)]
    struct CargoPackage {
        name: String,
        version: String,
        edition: String
    }
    let mut dependencies = HashMap::new();
    dependencies.insert("linearf".into(), {
        let mut m = serde_json::Map::new();
        m.insert("path".into(), "../../core".into());
        serde_json::Value::from(m)
    });
    for c in crates {
        dependencies.insert(c.name.clone(), c.dep.clone());
    }
    let manifest = Manifest {
        package: CargoPackage {
            name: "registrar".into(),
            version: "0.1.0".into(),
            edition: "2018".into()
        },
        dependencies
    };
    Ok(toml::to_string(&manifest)?)
}

fn format_main(crates: &[Crate]) -> String {
    let t = quote::quote! {
        use linearf::{AsyncRt, Shared, State};

        pub async fn register(state: &Shared<State>, handle: &AsyncRt) {}
    };
    todo!()
}

#[derive(Debug, Deserialize)]
struct Crate {
    name: String,
    dep: serde_json::Value,
    generators: Vec<GeneratorDescriptor>
}

#[derive(Debug, Deserialize)]
struct GeneratorDescriptor {
    path: String
}
