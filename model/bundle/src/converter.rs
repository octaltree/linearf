use crate::Recipe;
use proc_macro2::{Ident, TokenStream};

struct A {
    name: String,
    field: Ident,
    path: TokenStream
}

pub fn format(recipe: &Recipe) -> TokenStream {
    let m = recipe.converters.iter().map(|s| {
        let field = quote::format_ident!("{}", &s.name);
        let p = s.path.split("::").map(|p| quote::format_ident!("{}", p));
        let path = quote::quote! { #(#p)::* };
        A {
            name: s.name.clone(),
            field,
            path
        }
    });
    macro_rules! let_converters {
        ($($i:ident),*) => {
            $(let $i = m.clone().map($i);)*
        };
    }
    let_converters! {fields, new_fields, map_convert}
    quote::quote! {
        use linearf::{Shared, New, Vars, RwLock, AsyncRt, Item};
        use linearf::stream::*;
        use linearf::converter::*;
        use std::sync::Arc;
        use std::any::Any;
        use std::collections::HashMap;

        pub struct Converter {
            #(#fields)*
            state: Shared<linearf::State>,
        }

        impl linearf::converter::ConverterRegistry for Converter {
            fn new(state: linearf::Shared<linearf::State>, rt: linearf::AsyncRt) -> Self
            where
                Self: Sized,
            {
                Self {
                    #(#new_fields)*
                    state,
                }
            }

            fn map_convert(
                &self,
                senario: Arc<Vars>,
                items: impl Stream<Item = Item> + Send + 'static,
            ) -> Result<Pin<Box<dyn Stream<Item = Item>>>, MapConvertError> {
                let cs: Vec<&linearf::converter::Converter> = senario.converters.iter()
                    .map(|name| -> Result<_, MapConvertError> {
                        match name {
                            #(#map_convert)*
                            _ => Err(MapConvertError::ConverterNotFound)
                        }
                    }).try_fold(Vec::new(), |mut cs, r| {
                        cs.push(r?);
                        Ok(cs)
                    })?;
                let f = move |item: Item| -> Item {
                    let mut item = item;
                    for c in &cs {
                        match c {
                            linearf::converter::Converter::Simple(c) => {
                                item = c.convert(item);
                            }
                        }
                    }
                    item
                };
                Ok(Box::pin(items.map(f)))
            }
        }
    }
}

fn fields(a: A) -> TokenStream {
    let A { field, .. } = a;
    quote::quote! {
        #field: linearf::converter::Converter,
    }
}

fn new_fields(a: A) -> TokenStream {
    let A { field, path, .. } = a;
    quote::quote! {
        #field: <#path as New>::new(&state, &rt).into_converter(),
    }
}

fn map_convert(a: A) -> TokenStream {
    let A { name, field, .. } = a;
    quote::quote! {
        #name => Ok(&self.#field),
    }
}
