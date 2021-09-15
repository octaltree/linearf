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
    let registrar = here.parent().unwrap().join("registrar");
    let cargo_toml = registrar.join("Cargo.toml");
    let lib = registrar.join("src").join("lib.rs");
    (cargo_toml, lib)
}

fn format_cargo_toml(recipe: &Recipe) -> StdResult<String> {
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
    // dependencies.insert("linearf".into(), {
    //    let mut m = serde_json::Map::new();
    //    m.insert("path".into(), "../../core".into());
    //    serde_json::Value::from(m)
    //});
    for c in &recipe.crates {
        let m = serde_json::from_str(&c.dep)?;
        dependencies.insert(c.name.clone(), m);
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

fn format_lib(recipe: &Recipe) -> String {
    let registrations = recipe
        .sources
        .iter()
        .map(|s| {
            let name = &s.name;
            let p = s.path.split("::").map(|p| quote::format_ident!("{}", p));
            let path = quote::quote! {
                #(#p)::*
            };
            let t = quote::format_ident!(
                "{}",
                match &s.r#type {
                    GeneratorType::Static => "Static",
                    GeneratorType::Dynamic => "Dynamic"
                }
            );
            quote::quote! {
                let g = Arc::new(RwLock::new(#path::new(state, handle)));
                let s = Source::#t(g);
                State::register_source(state, #name, s).await;
            }
        })
        .chain(recipe.matchers.iter().map(|m| {
            let name = &m.name;
            let p = m.path.split("::").map(|p| quote::format_ident!("{}", p));
            let path = quote::quote! {
                #(#p)::*
            };
            quote::quote! {
                let m = Arc::new(RwLock::new(#path::new(state, handle)));
                State::register_matcher(state, #name, m).await;
            }
        }));
    let t = quote::quote! {
        pub async fn register() {
            #(#registrations)*
        }
    };
    t.to_string()
}

#[derive(Debug, Deserialize, Default)]
struct Recipe {
    crates: Vec<Crate>,
    #[serde(default)]
    sources: Vec<SourceDescriptor>,
    #[serde(default)]
    matchers: Vec<MatchDescriptor>
}

#[derive(Debug, Deserialize)]
struct Crate {
    name: String,
    dep: String
}

#[derive(Debug, Deserialize)]
struct SourceDescriptor {
    name: String,
    path: String,
    r#type: GeneratorType
}

#[derive(Debug, Deserialize)]
struct MatchDescriptor {
    name: String,
    path: String
}

#[derive(Debug, Deserialize)]
enum GeneratorType {
    Static,
    Dynamic
}
