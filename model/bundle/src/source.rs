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
        let params = quote::quote! { <#path as IsSource>::Params };
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
        use linearf::{Shared, New, Vars, RwLock, AsyncRt, Item};
        use linearf::source::*;
        use linearf::stream::*;
        use std::sync::Arc;
        use std::any::Any;
        use serde::Deserialize;

        pub struct Source {
            #(#fields)*
            state: Shared<linearf::State>,
        }

        impl<'de, D> linearf::source::SourceRegistry<'de, D> for Source
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

            fn stream(
                &self,
                name: &str,
                senario: (Arc<Vars>, Arc<dyn Any + Send + Sync>),
            ) -> Pin<Box<dyn Stream<Item = Item>>>
            where
                Self: Sized,
            {
                match name {
                    #(#stream)*
                    _ => Box::pin(linearf::stream::empty())
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
        #field: <#path as New>::new(&state, &rt).into_source(),
    }
}

fn parses(A { name, params, .. }: A) -> TokenStream {
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
        let (prev_vars, prev_source) = prev;
        let (senario_vars, senario_source) = senario;
        if prev_source.is::<#params>()
            && senario_source.is::<#params>()
        {
            let prev_source: &Arc<#params> =
                unsafe { std::mem::transmute(prev_source) };
            let senario_source: &Arc<#params> =
                unsafe { std::mem::transmute(senario_source) };
            g.reusable(ctx, (prev_vars, prev_source), (senario_vars, senario_source))
        } else {
            false
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
            log::error!("mismatch params type");
            return Box::pin(linearf::stream::empty());
        }
        let (senario_source, _): (Arc<#params>, usize) =
            unsafe { std::mem::transmute(senario_source) };
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
