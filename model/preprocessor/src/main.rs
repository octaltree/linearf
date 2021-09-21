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
    struct A<I, T> {
        name: String,
        field: I,
        path: T,
        params: T,
        sender: I
    }
    let s = recipe.sources.iter().map(|s| {
        let field = quote::format_ident!("{}", &s.name);
        let p = s.path.split("::").map(|p| quote::format_ident!("{}", p));
        let path = quote::quote! { #(#p)::* };
        let params = quote::quote! { <#path as HasSourceParams>::Params };
        let sender = quote::format_ident!("{}_sender", &s.name);
        A {
            name: s.name.clone(),
            field,
            path,
            params,
            sender
        }
    });
    let fields = s.clone().map(
        |A {
             name,
             params,
             sender,
             ..
         }| {
            quote::quote! {
                #name: linearf::source::Source<#params>,
                #sender: Option<Sender<(Transmitter, (&Arc<Vars>, &Arc<#params>))>>
            }
        }
    );
    let new_fields = s.clone().map(
        |A {
             name, path, sender, ..
         }| {
            quote::quote! {
                #name: <#path as New>::new(&state).into_source(),
                #sender: None
            }
        }
    );
    let parses = s.clone().map(|A { name, params, .. }| {
        quote::quote! {
            #name => Ok(Some(Arc::new(#params::deserialize(deserializer)?)))
        }
    });
    let reusable = s.clone().map(
        |A {
             name,
             field,
             path,
             params,
             ..
         }| {
            let p = quote::quote! {
                let (prev_vars, prev_source) = prev;
                let (senario_vars, senario_source) = senario;
                if prev_source.is::<#params>()
                    && senario_source.is::<#params>()
                {
                    let prev_source: &Arc<#params> =
                        unsafe { std::mem::transmute(prev_source) };
                    let senario_source: &Arc<#params> =
                        unsafe { std::mem::transmute(senario_source) };
                    g.read().await.reusable(
                        (prev_vars, prev_source), (senario_vars, senario_source)).await
                } else {
                    false
                }
            };
            quote::quote! {
                #name => match &self.#field {
                    linearf::source::Source::Simple(g) => { #p }
                    linearf::source::Source::Flow(g) => { #p }
                }
            }
        }
    );
    let on_session_start = s.clone().map(
        |A {
             name,
             field,
             path,
             sender,
             params,
             ..
         }| {
            // let pre = quote::quote! {
            //    let (senario_vars, senario_source) = senario;
            //    if !senario_source.is::<#params>() {
            //        return; // drop and close the channel
            //    }
            //    let senario_source: &Arc<#params> =
            //        unsafe { std::mem::transmute(senario_source) };
            //};
            // quote::quote! {
            //    #name => match &self.#field {
            //        linearf::source::Source::Simple(g) => {
            //            #pre
            //            g.generate(tx, (senario_vars, senario_source))
            //        }
            //        linearf::source::Source::Flow(g) => {
            //            #pre
            //        }
            //    }
            //}
        }
    );
    let t = quote::quote! {
        use linearf::Shared;
        use linearf::New;
        use linearf::Vars;
        use linearf::session::Sender;
        use linearf::source::{SimpleGenerator, FlowGenerator, HasSourceParams, SourceType, Transmitter};
        use std::sync::Arc;
        use std::any::Any;
        use serde::Deserialize;
        use async_trait::async_trait;

        pub struct Source {
            #(#fields),*,
            state: linearf::Shared<linearf::State>
        }

        #[async_trait]
        impl<'de, D> linearf::source::SourceRegistry<'de, D> for Source
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
            ) -> Result<Option<Arc<dyn Any + Send + Sync>>, D::Error> {
                match name {
                    #(#parses),*,
                    _ => Ok(None)
                }
            }

            async fn reusable(
                &self,
                name: &str,
                prev: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>),
                senario: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>)
            ) -> bool
            where
                Self: Sized
            {
                match name {
                    #(#reusable),*,
                    _ => false
                }
            }

            async fn on_session_start(
                &self,
                name: &str,
                tx: linearf::source::Transmitter,
                senario: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>)
            ) where
                Self: Sized
            {
                match name {
                    _ => {}
                }
            }

            async fn on_flow_start(
                &self,
                name: &str,
                tx: linearf::source::Transmitter,
                senario: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>)
            ) where
                Self: Sized
            {
                match name {
                    _ => {}
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
