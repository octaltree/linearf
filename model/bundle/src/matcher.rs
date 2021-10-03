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
        let params = quote::quote! { <#path as IsMatcher>::Params };
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
    let_matchers! {fields, new_fields, parses, reusable, score}
    quote::quote! {
        use linearf::{Shared, New, Vars, AsyncRt, Item};
        use linearf::matcher::*;
        use linearf::source;
        use linearf::stream::*;
        use std::sync::Arc;
        use std::any::Any;
        use serde::Deserialize;

        pub struct Matcher {
            #(#fields)*
            state: linearf::Shared<linearf::State>,
        }

        impl<'de, D> linearf::matcher::MatcherRegistry<'de, D> for Matcher
        where
            D: serde::de::Deserializer<'de>
        {
            fn new(state: linearf::Shared<linearf::State>, rt: AsyncRt) -> Self
            where
                Self: Sized
            {
                Self {
                    #(#new_fields)*
                    state,
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

            fn reusable(
                &self,
                name: &str,
                ctx: ReusableContext<'_>,
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

            fn score<'a>(
                &self,
                name: &str,
                senario: (Arc<Vars>, Arc<dyn Any + Send + Sync>),
                items: impl Stream<Item = Arc<Item>> + Send + 'static
            ) -> Pin<Box<dyn Stream<Item = WithScore>>> {
                match name {
                    #(#score)*
                    _ => {
                        Box::pin(linearf::stream::empty())
                    }
                }
            }
        }
    }
}

fn fields(a: A) -> TokenStream {
    let A { field, params, .. } = a;
    quote::quote! {
        #field: linearf::matcher::Matcher<#params>,
    }
}

fn new_fields(a: A) -> TokenStream {
    let A { field, path, .. } = a;
    quote::quote! {
        #field: <#path as New>::new(&state, &rt).into_matcher(),
    }
}

fn parses(a: A) -> TokenStream {
    let A { name, params, .. } = a;
    quote::quote! {
        #name => Ok(Some(Arc::new(#params::deserialize(deserializer)?))),
    }
}

fn reusable(a: A) -> TokenStream {
    let A {
        name,
        field,
        params,
        ..
    } = a;
    let p = quote::quote! {
        let (prev_vars, prev_matcher) = prev;
        let (senario_vars, senario_matcher) = senario;
        if prev_matcher.is::<#params>()
            && senario_matcher.is::<#params>()
        {
            let prev_matcher: &Arc<#params> =
                unsafe { std::mem::transmute(prev_matcher) };
            let senario_matcher: &Arc<#params> =
                unsafe { std::mem::transmute(senario_matcher) };
            s.reusable(ctx, (prev_vars, prev_matcher), (senario_vars, senario_matcher))
        } else {
            false
        }
    };
    quote::quote! {
        #name => match &self.#field {
            linearf::matcher::Matcher::Simple(s) => { #p }
        },
    }
}

fn score(a: A) -> TokenStream {
    let A {
        name,
        field,
        params,
        ..
    } = a;
    quote::quote! {
        #name => match &self.#field {
            linearf::matcher::Matcher::Simple(s) => {
                // TODO: channel is none if buffer is empty
                let (senario_vars, senario_matcher) = senario;
                if senario_matcher.is::<#params>() {
                    let (senario_matcher, _): (Arc<#params>, usize) =
                                               unsafe { std::mem::transmute(senario_matcher) };
                    Box::pin(items.map(move |x| {
                        let score = s.score((senario_vars, senario_matcher), &x);
                        (x, Arc::new(score))
                    }))
                } else {
                    log::error!("mismatch params type");
                    Box::pin(linearf::stream::empty())
                }
            }
        },
    }
}
