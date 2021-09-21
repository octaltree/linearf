mod source;

use serde::{Deserialize, Serialize};
use std::error::Error as StdError;

pub type StdResult<T> = Result<T, Box<dyn StdError>>;

#[derive(Debug, Deserialize, Default)]
pub struct Recipe {
    #[serde(default)]
    pub crates: Vec<Crate>,
    #[serde(default)]
    pub sources: Vec<SourceDescriptor>,
    #[serde(default)]
    pub matchers: Vec<MatchDescriptor>
}

#[derive(Debug, Deserialize)]
pub struct Crate {
    pub name: String,
    pub dep: String
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

pub fn format_lib(recipe: &Recipe) -> String { crate::source::format(recipe).to_string() }

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
        d.insert("async-trait".into(), {
            let mut m = toml::value::Map::new();
            m.insert("version".into(), "*".into());
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
