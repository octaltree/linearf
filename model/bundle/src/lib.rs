mod converter;
mod matcher;
mod source;

use serde::{Deserialize, Serialize};
use std::{error::Error as StdError, path::PathBuf};

pub type StdResult<T> = Result<T, Box<dyn StdError>>;

#[derive(Debug, Deserialize, Default)]
pub struct Recipe {
    #[serde(default)]
    pub crates: Vec<Crate>,
    #[serde(default)]
    pub sources: Vec<SourceDescriptor>,
    #[serde(default)]
    pub matchers: Vec<MatchDescriptor>,
    #[serde(default)]
    pub converters: Vec<ConverterDescriptor>
}

#[derive(Debug, Deserialize)]
pub struct Crate {
    pub name: String,
    pub dir: PathBuf
}

#[derive(Debug, Deserialize)]
pub struct SourceDescriptor {
    pub name: String,
    pub path: String
}

#[derive(Debug, Deserialize)]
pub struct MatchDescriptor {
    pub name: String,
    pub path: String
}

#[derive(Debug, Deserialize)]
pub struct ConverterDescriptor {
    pub name: String,
    pub path: String
}

pub fn format_lib(recipe: &Recipe) -> String {
    let source = crate::source::format(recipe);
    let matcher = crate::matcher::format(recipe);
    let converter = crate::converter::format(recipe);
    quote::quote! (
        pub use source::Source;
        pub use matcher::Matcher;
        pub use converter::Converter;
        mod source {
            #source
        }
        mod matcher {
            #matcher
        }
        mod converter {
            #converter
        }
    )
    .to_string()
}

pub fn format_cargo_toml(recipe: &Recipe) -> StdResult<String> {
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
        d.insert("log".into(), {
            let mut m = toml::value::Map::new();
            m.insert("version".into(), "*".into());
            toml::Value::from(m)
        });
        d.insert("rayon".into(), {
            let mut m = toml::value::Map::new();
            m.insert("version".into(), "*".into());
            toml::Value::from(m)
        });
        for c in &recipe.crates {
            let mut m = toml::map::Map::new();
            m.insert(
                "path".into(),
                toml::Value::from(c.dir.display().to_string())
            );
            let dep = toml::Value::Table(m);
            d.insert(c.name.clone(), dep);
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
