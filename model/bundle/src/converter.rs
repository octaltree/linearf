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
        use linearf::converter::*;
        use std::collections::HashMap;
        use std::marker::PhantomData;

        pub struct Converter<L> {
            phantom: PhantomData<L>,
            #(#fields)*
        }

        impl<L> Converter<L>
        where
            L: linearf::Linearf + Send + Sync
        {
            fn new(linearf: Weak<L>) -> Self
            {
                Self {
                    phantom: PhantomData,
                    #(#new_fields)*
                }
            }
        }

        impl<L> ConverterRegistry for Converter<L>
        where
            L : linearf::Linearf + Send + Sync
        {
            fn map_convert(
                &self,
                names: &[SmartString],
                items: impl Stream<Item = Item> + Send + Sync + 'static,
            ) -> Result<Pin<Box<dyn Stream<Item = Item> + Send + Sync>>, MapConvertError> {
                let cs: Vec<&linearf::converter::Converter<L>> = names.iter()
                    .map(|name| -> Result<_, MapConvertError> {
                        match name {
                            #(#map_convert)*
                            _ => Err(MapConvertError::ConverterNotFound(name.clone()))
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
        #field: linearf::converter::Converter<L>,
    }
}

fn new_fields(a: A) -> TokenStream {
    let A { field, path, .. } = a;
    quote::quote! {
        #field: <#path as New<L>>::new(&state, &rt).into_converter(),
    }
}

fn map_convert(a: A) -> TokenStream {
    let A { name, field, .. } = a;
    quote::quote! {
        #name => Ok(&self.#field),
    }
}
