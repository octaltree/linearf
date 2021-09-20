use serde::{Deserialize, Serialize};
use std::{
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
    let registry = here.parent().unwrap().join("registry");
    let cargo_toml = registry.join("Cargo.toml");
    let lib = registry.join("src").join("lib.rs");
    (cargo_toml, lib)
}

fn format_cargo_toml(recipe: &Recipe) -> StdResult<String> {
    #[derive(Serialize)]
    struct Manifest {
        package: CargoPackage,
        dependencies: toml::value::Map<String, toml::Value>
    }
    #[derive(Serialize)]
    struct CargoPackage {
        name: String,
        version: String,
        edition: String
    }
    let dependencies = {
        let mut d = toml::value::Map::new();
        d.insert("linearf".into(), {
            let mut m = toml::value::Map::new();
            m.insert("path".into(), "../core".into());
            toml::Value::from(m)
        });
        d.insert("serde".into(), {
            let mut m = toml::value::Map::new();
            m.insert("version".into(), "*".into());
            m.insert("features".into(), toml::Value::Array(vec!["derive".into()]));
            toml::Value::from(m)
        });
        for c in &recipe.crates {
            let m: toml::value::Map<String, toml::Value> =
                toml::from_str(&format!("{} = {}", &c.name, &c.dep))?;
            let t = m.into_iter().next().unwrap();
            d.insert(t.0, t.1);
        }
        d
    };
    let manifest = Manifest {
        package: CargoPackage {
            name: "registry".into(),
            version: "0.1.0".into(),
            edition: "2018".into()
        },
        dependencies
    };
    Ok(toml::to_string(&manifest)?)
}

fn format_lib(recipe: &Recipe) -> String {
    let sources = recipe.sources.iter().map(|s| {
        let name = quote::format_ident!("{}", &s.name);
        let p = s.path.split("::").map(|p| quote::format_ident!("{}", p));
        let path = quote::quote! {
            #(#p)::*
        };
        (s.name.clone(), name, path)
    });
    let fields = sources.clone().map(|(_, name, path)| {
        quote::quote! {
            #name: linearf::source::Source<<#path as HasSourceParams>::Params>
        }
    });
    let new_fields = sources.clone().map(|(_, name, path)| {
        quote::quote! {
            #name: <#path as New>::new(&state).into_source()
        }
    });
    let parses = sources.clone().map(|(name, _, path)| {
        quote::quote! {
            #name => Ok(Some(Arc::new(
                        <#path as HasSourceParams>::Params::deserialize(deserializer)?)))
        }
    });
    let source_types = sources.clone().map(|(name, field, _)| {
        quote::quote! {
            #name => Some((&self.#field).into())
        }
    });
    let t = quote::quote! {
        use linearf::Shared;
        use linearf::New;
        use linearf::source::{SimpleGenerator, FlowGenerator, HasSourceParams, SourceType};
        use std::sync::Arc;
        use serde::Deserialize;

        pub struct Source {
            #(#fields),*,
            state: linearf::Shared<linearf::State>
        }

        impl<'de, D> linearf::SourceRegistry<'de, D> for Source
        where
            D: serde::de::Deserializer<'de>
        {
            fn new(state: linearf::Shared<linearf::State>) -> Self
            where
                Self: Sized
            {
                Self { #(#new_fields),*, state }
            }

            fn parse(
                &self,
                name: &str,
                deserializer: D
            ) -> Result<Option<std::sync::Arc<dyn std::any::Any + Send + Sync>>, D::Error> {
                match name {
                    #(#parses),*,
                    _ => Ok(None)
                }
            }

            fn source_type(&self, name: &str) -> Option<SourceType> {
                match name {
                    #(#source_types),*,
                    _ => None
                }
            }
        }
    };
    t.to_string()
}

#[derive(Debug, Deserialize, Default)]
struct Recipe {
    #[serde(default)]
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
    path: String
}

#[derive(Debug, Deserialize)]
struct MatchDescriptor {
    name: String,
    path: String
}
