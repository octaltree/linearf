use crate::Recipe;
use proc_macro2::{Ident, TokenStream};

struct A {
    name: String,
    field: Ident,
    path: TokenStream,
    params: TokenStream
}

pub fn format(recipe: &Recipe) -> TokenStream {
    let m = recipe.converters.iter().map(|s| {
        let field = quote::format_ident!("{}", &s.name);
        let p = s.path.split("::").map(|p| quote::format_ident!("{}", p));
        let path = quote::quote! { #(#p)::* };
        let params = quote::quote! { <#path<L> as IsConverter>::Params };
        A {
            name: s.name.clone(),
            field,
            path,
            params
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
            L: linearf::Linearf + Send + Sync + 'static
        {
            pub fn new(linearf: Weak<L>) -> Self
            {
                Self {
                    phantom: PhantomData,
                    #(#new_fields),*
                }
            }
        }

        impl<L> ConverterRegistry for Converter<L>
        where
            L: linearf::Linearf + Send + Sync + 'static
        {
            fn map_convert(
                &self,
                names: &[SmartString],
                items: impl Stream<Item = Item> + Send + Sync + 'static,
            ) -> Result<Pin<Box<dyn Stream<Item = Item> + Send + Sync>>, MapConvertError> {
                let fs = names.iter()
                    .map(|n| -> &str { &n })
                    .map(|name| -> Result<Box<dyn Fn(Item) -> Item + Send + Sync>, MapConvertError> {
                        match name {
                            #(#map_convert)*
                            _ => Err(MapConvertError::ConverterNotFound(SmartString::from(name)))
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let f = move |mut item: Item| -> Item {
                    for f in &fs {
                        item = f(item);
                    }
                    item
                };
                Ok(Box::pin(items.map(f)))
            }
        }
    }
}

fn fields(a: A) -> TokenStream {
    let A { field, params, .. } = a;
    quote::quote! {
        #field: linearf::converter::Converter<#params>,
    }
}

fn new_fields(a: A) -> TokenStream {
    let A { field, path, .. } = a;
    quote::quote! {
        #field: <#path<L> as NewConverter<L>>::new(linearf.clone())
    }
}

fn map_convert(a: A) -> TokenStream {
    let A { name, field, .. } = a;
    quote::quote! {
        #name => {
            match &self.#field {
                linearf::converter::Converter::Simple(c) => {
                    let c = Arc::clone(c);
                    Ok(Box::new(move |item| c.convert(item)))
                }
                linearf::converter::Converter::Reserve(_) => {
                    Err(MapConvertError::ConverterNotFound(SmartString::from(name)))
                }
            }
        }
    }
}
