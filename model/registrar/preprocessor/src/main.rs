use std::{env, error::Error as StdError};
type StdResult<T> = Result<T, Box<dyn StdError>>;
use serde::Deserialize;

fn main() -> StdResult<()> {
    let x = input()?;
    let lib = quote::quote! {
        use linearf::{Shared, State, AsyncRt};

        pub async fn register(state: &Shared<State>, rt: &AsyncRt) {}
    };
    Ok(())
}

fn input() -> StdResult<Vec<Crate>> {
    match env::var("LINEARF_REG") {
        Ok(s) => Ok(serde_json::from_str(&s)?),
        Err(env::VarError::NotPresent) => Ok(Vec::new()),
        Err(e) => Err(e.into())
    }
}

#[derive(Debug, Deserialize)]
struct Crate {
    name: String
}
