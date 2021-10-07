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
        let params = quote::quote! { <#path<L> as IsMatcher>::Params };
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
        use linearf::matcher::*;

        pub struct Matcher<L> {
            #(#fields)*
        }

        impl<L> Matcher<L>
        where
            L: linearf::Linearf + Send + Sync
        {
            fn new(linearf: Weak<L>) -> Self
            {
                Self {
                    #(#new_fields)*
                }
            }
        }

        impl<L> MatcherRegistry for Matcher<L>
        where
            L : linearf::Linearf + Send + Sync
        {
            fn parse<'de, D>(
                &self,
                name: &str,
                deserializer: D
            ) -> Option<Result<Arc<dyn Any + Send + Sync>, D::Error>>
            where
                D: serde::de::Deserializer<'de>
            {
                match name {
                    #(#parses)*
                    _ => None
                }
            }

            fn reusable(
                &self,
                name: &str,
                prev: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>),
                senario: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>)
            ) -> Reusable
            {
                match name {
                    #(#reusable)*
                    _ => Reusable::None
                }
            }

            fn score(
                &self,
                name: &str,
                senario: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>),
                items: impl Stream<Item = Arc<Item>> + Send + Sync + 'static
            ) -> Pin<Box<dyn Stream<Item = WithScore> + Send + Sync>> {
                match name {
                    #(#score)*
                    _ => {
                        Box::pin(empty())
                    }
                }
            }
        }
    }
}

fn fields(a: A) -> TokenStream {
    let A { field, params, .. } = a;
    quote::quote! {
        #field: linearf::matcher::Matcher<L, #params>,
    }
}

fn new_fields(a: A) -> TokenStream {
    let A { field, path, .. } = a;
    quote::quote! {
        #field: <#path<L> as New<L>>::new(linearf.clone()).into_matcher(),
    }
}

fn parses(a: A) -> TokenStream {
    let A { name, params, .. } = a;
    quote::quote! {
        #name => match #params::deserialize(deserializer) {
            Ok(x) => Some(Ok(Arc::new(x))),
            Err(e) => Some(Err(e))
        }
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
            let prev_matcher: &Arc<#params> = unsafe { std::mem::transmute(prev_matcher) };
            let senario_matcher: &Arc<#params> = unsafe { std::mem::transmute(senario_matcher) };
            s.reusable((prev_vars, prev_matcher), (senario_vars, senario_matcher))
        } else {
            log::error!("mismatch matcher reusable params");
            Reusable::None
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
                    let senario_matcher: &Arc<#params> =
                        unsafe { std::mem::transmute(senario_matcher) };
                    Box::pin(items.map(move |x| {
                        let score = s.score((senario_vars, senario_matcher), &x);
                        (x, Arc::new(score))
                    }))
                } else {
                    log::error!("mismatch matcher score params");
                    Box::pin(empty())
                }
            }
        },
    }
}
