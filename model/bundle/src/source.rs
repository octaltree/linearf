use crate::Recipe;
use proc_macro2::{Ident, TokenStream};

struct A {
    name: String,
    field: Ident,
    path: TokenStream,
    params: TokenStream
}

pub fn format(recipe: &Recipe) -> TokenStream {
    let s = recipe.sources.iter().map(|s| {
        let field = quote::format_ident!("{}", &s.name);
        let p = s.path.split("::").map(|p| quote::format_ident!("{}", p));
        let path = quote::quote! { #(#p)::* };
        let params = quote::quote! { <#path<L> as IsSource>::Params };
        A {
            name: s.name.clone(),
            field,
            path,
            params
        }
    });
    macro_rules! let_sources {
        ($($i:ident),*) => {
            $(let $i = s.clone().map($i);)*
        };
    }
    let_sources! {fields, new_fields, parses, reusable, stream}
    quote::quote! {
        use linearf::source::*;

        pub struct Source<L> {
            #(#fields)*
        }

        impl<L> Source<L>
        where
            L: linearf::Linearf + Send + Sync + 'static
        {
            pub fn new(linearf: Weak<L>) -> Self
            where
                Self: Sized
            {
                Self {
                    #(#new_fields)*
                }
            }
        }

        impl<L> SourceRegistry for Source<L>
        where
            L: linearf::Linearf + Send + Sync
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

            fn stream(
                &self,
                name: &str,
                senario: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>),
            ) -> Pin<Box<dyn Stream<Item = Item> + Send +Sync>>
            {
                match name {
                    #(#stream)*
                    _ => Box::pin(empty())
                }
            }
        }
    }
}

fn fields(a: A) -> TokenStream {
    let A { field, params, .. } = a;
    quote::quote! {
        #field: linearf::source::Source<#params>,
    }
}

fn new_fields(a: A) -> TokenStream {
    let A { field, path, .. } = a;
    quote::quote! {
        #field: <#path<L> as NewSource<L>>::new(linearf.clone())
    }
}

fn parses(A { name, params, .. }: A) -> TokenStream {
    quote::quote! {
        #name => match #params::deserialize(deserializer) {
            Ok(x) => Some(Ok(Arc::new(x))),
            Err(e) => Some(Err(e))
        },
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
        let (prev_vars, prev_source) = prev;
        let (senario_vars, senario_source) = senario;
        if prev_source.is::<#params>()
            && senario_source.is::<#params>()
        {
            let prev_source: &Arc<#params> = unsafe { std::mem::transmute(prev_source) };
            let senario_source: &Arc<#params> = unsafe { std::mem::transmute(senario_source) };
            g.reusable((prev_vars, prev_source), (senario_vars, senario_source))
        } else {
            log::error!("mismatch source reusable params");
            Reusable::None
        }
    };
    quote::quote! {
        #name => match &self.#field {
            linearf::source::Source::Simple(g) => { #p }
        },
    }
}

fn stream(a: A) -> TokenStream {
    let A {
        name,
        field,
        params,
        ..
    } = a;
    let pre = quote::quote! {
        let (senario_vars, senario_source) = senario;
        if !senario_source.is::<#params>() {
            log::error!("mismatch source stream params");
            return Box::pin(empty());
        }
        let senario_source: &Arc<#params> = unsafe { std::mem::transmute(senario_source) };
    };
    quote::quote! {
        #name => match self.#field.clone() {
            linearf::source::Source::Simple(g) => {
                #pre
                g.stream((&senario_vars, &senario_source))
            }
        },
    }
}
