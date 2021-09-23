use crate::Recipe;
use proc_macro2::{Ident, TokenStream};

struct A {
    name: String,
    field: Ident,
    path: TokenStream,
    params: TokenStream
}

pub fn format(recipe: &Recipe) -> TokenStream {
    let m = recipe.matchers.iter().map(|s| {
        let field = quote::format_ident!("{}", &s.name);
        let p = s.path.split("::").map(|p| quote::format_ident!("{}", p));
        let path = quote::quote! { #(#p)::* };
        let params = quote::quote! { <#path as IsSource>::Params };
        A {
            name: s.name.clone(),
            field,
            path,
            params
        }
    });
    macro_rules! let_matchers {
        ($($i:ident),*) => {
            $(let $i = m.clone().map($i);)*
        };
    }
    let_matchers! {fields, new_fields, parses, reusable}
    quote::quote! {
        use linearf::{Shared, New, Vars, RwLock, AsyncRt};
        use linearf::matcher::*;
        use std::sync::Arc;
        use std::any::Any;
        use serde::Deserialize;
        use async_trait::async_trait;


        pub struct Matcher {
            #(#fields)*
            state: linearf::Shared<linearf::State>
        }

        #[async_trait]
        impl<'de, D> linearf::matcher::MatcherRegistry<'de, D> for Matcher
        where
            D: serde::de::Deserializer<'de>
        {
            fn new(state: linearf::Shared<linearf::State>) -> Self
            where
                Self: Sized
            {
                Self {
                    #(#new_fields)*
                    state
                }
            }

            fn parse(
                &self,
                name: &str,
                deserializer: D
            ) -> Result<Option<Arc<dyn Any + Send + Sync>>, D::Error> {
                match name {
                    #(#parses)*
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
                    #(#reusable)*
                    _ => false
                }
            }
        }
    }
}

fn fields(a: A) -> TokenStream {
    quote::quote! {}
}

fn new_fields(a: A) -> TokenStream {
    quote::quote! {}
}

fn parses(a: A) -> TokenStream {
    quote::quote! {}
}

fn reusable(a: A) -> TokenStream {
    quote::quote! {}
}
