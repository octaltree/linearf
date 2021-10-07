use bundle::{format_cargo_toml, format_lib, Crate, Recipe, StdResult};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio}
};

fn main() -> StdResult<()> {
    let env_recipe = env::var("LINEARF_RECIPE");
    let env_dir = env!("CARGO_MANIFEST_DIR");
    let features = {
        let mut a = env::args();
        a.next();
        a.next()
    }
    .expect("The first argument is needed for \"features\"");
    println!("{} {:?}", features, env_recipe.as_ref().ok());
    let recipe = input(env_recipe)?;
    let here = Path::new(env_dir);
    let core = here.parent().unwrap().join("core");
    let (registry_toml, registry_lib) = registry(here);
    let crates = read_crates(&recipe.crates)?;
    let stash = Stash::stash(
        &registry_toml,
        &registry_lib,
        crates.iter().map(|(_, f)| f.clone()).collect()
    )?;
    preprocess(&recipe, registry_lib, registry_toml, crates, &core)?;
    let run = build(&features);
    stash.restore()?;
    std::process::exit(run?.code().ok_or("Process terminated by signal")?);
}

fn input(env_reg: Result<String, env::VarError>) -> StdResult<Recipe> {
    match env_reg {
        Ok(s) => Ok(serde_json::from_str(&s)?),
        Err(env::VarError::NotPresent) => Ok(Recipe::default()),
        Err(e) => Err(e.into())
    }
}

fn registry(here: &Path) -> (PathBuf, PathBuf) {
    let registry = here.parent().unwrap().join("registry");
    let cargo_toml = registry.join("Cargo.toml");
    let lib = registry.join("src").join("lib.rs");
    (cargo_toml, lib)
}

fn read_crates(crates: &[Crate]) -> std::io::Result<Vec<(&Crate, F)>> {
    crates
        .iter()
        .map(|c| {
            let f = F::read(&c.dir.join("Cargo.toml"))?;
            Ok((c, f))
        })
        .collect()
}

fn preprocess(
    recipe: &Recipe,
    registry_lib: PathBuf,
    registry_toml: PathBuf,
    crates: Vec<(&Crate, F)>,
    core: &Path
) -> StdResult<()> {
    fs::write(&registry_lib, format_lib(recipe))?;
    fs::write(&registry_toml, format_cargo_toml(recipe)?)?;
    for (c, F { p, s }) in crates.into_iter() {
        let mut manifest: toml::value::Table = toml::from_str(&s)?;
        let deps = manifest
            .get_mut("dependencies")
            .ok_or_else(|| format!("{:?} has no \"dependencies\" ", &p))?
            .as_table_mut()
            .ok_or_else(|| format!("{:?} has no \"dependencies\" ", &p))?;
        deps["linearf"] = {
            let mut m = toml::map::Map::new();
            m.insert(
                "path".to_string(),
                toml::Value::from(relative(&p, core).display().to_string())
            );
            toml::Value::Table(m)
        };
        let pac = manifest
            .get_mut("package")
            .ok_or_else(|| format!("{:?} has no \"package\" ", &p))?
            .as_table_mut()
            .ok_or_else(|| format!("{:?} has no \"package\" ", &p))?;
        pac["name"] = toml::Value::String(c.name.clone());
        fs::write(p, toml::to_string(&toml::Value::Table(manifest))?)?;
    }
    Ok(())
}

fn build(features: &str) -> std::io::Result<ExitStatus> {
    Command::new("cargo")
        .args(["fmt", "-p", "registry"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .ok();
    Command::new("cargo")
        .args(["build", "--features", features, "--release", "--lib=bridge"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
}

pub struct Stash {
    files: Vec<F>
}

#[derive(Clone)]
struct F {
    p: PathBuf,
    s: String
}

impl F {
    fn read(p: &Path) -> std::io::Result<Self> {
        Ok(Self {
            p: p.into(),
            s: fs::read_to_string(p)?
        })
    }
}

impl Stash {
    #[allow(clippy::self_named_constructors)]
    fn stash(cargo_toml: &Path, lib: &Path, crates: Vec<F>) -> StdResult<Self> {
        let mut files = crates.to_vec();
        files.push(F::read(cargo_toml)?);
        files.push(F::read(lib)?);
        Ok(Self { files })
    }

    fn restore(self) -> StdResult<()> {
        for F { p, s } in self.files {
            fs::write(p, s)?;
        }
        Ok(())
    }
}

fn relative(root: &Path, file: &Path) -> PathBuf {
    let l: Vec<_> = root.components().into_iter().collect();
    let r: Vec<_> = file.components().into_iter().collect();
    let (cnt, _) = root
        .components()
        .zip(file.components())
        .map(|(a, b)| a == b)
        .fold((0, true), |(cnt, success), same| {
            let success = success && same;
            let cnt = if success { cnt + 1 } else { cnt };
            (cnt, success)
        });
    use std::borrow::Cow;
    let prefix = if l.len() - cnt <= 1 {
        Cow::Borrowed("")
    } else {
        Cow::Owned((0..(l.len() - cnt - 1)).map(|_| "../").collect::<String>())
    };
    Path::new(&*prefix).join(r[cnt..].iter().collect::<PathBuf>())
}

#[test]
fn can_relative() {
    assert_eq!(
        relative(Path::new("/foo.ts"), Path::new("/bar.ts")),
        Path::new("bar.ts")
    );
    assert_eq!(
        relative(Path::new("/foo.ts"), Path::new("/foo/bar.ts")),
        Path::new("foo/bar.ts")
    );
    assert_eq!(
        relative(Path::new("/foo/bar.ts"), Path::new("/bar.ts")),
        Path::new("../bar.ts")
    );
}
