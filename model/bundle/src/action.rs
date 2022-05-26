use crate::Recipe;
use proc_macro2::{Ident, TokenStream};

struct A {
    name: String,
    field: Ident,
    path: TokenStream,
    params: TokenStream,
    result: TokenStream
}

pub fn format(recipe: &Recipe) -> TokenStream {
    let a = recipe.actions.iter().map(|a| {
        let field = quote::format_ident!("{}", &a.name);
        let p = a.path.split("::").map(|p| quote::format_ident!("{}", p));
        let path = quote::quote! { #(#p)::* };
        let params = quote::quote! { <#path<L> as IsAction>::Params };
        let result = quote::quote! { <#path<L> as IsAction>::Result };
        A {
            name: a.name.clone(),
            field,
            path,
            params,
            result
        }
    });
    macro_rules! let_actions {
        ($($i:ident),*) => {
            $(let $i = a.clone().map($i);)*
        };
    }
    let_actions! {fields, new_fields, parses, run, serialize}
    quote::quote! {
        use linearf::action::*;
        use std::marker::PhantomData;

        pub struct Action<L> {
            phantom: PhantomData<L>,
            #(#fields)*
        }

        impl<L> Action<L>
        where
            L: linearf::Linearf + Send + Sync + 'static
        {
            pub fn new(linearf: Weak<L>) -> Self
            {
                Self {
                    phantom: PhantomData,
                    #(#new_fields)*
                }
            }
        }

        impl<L> ActionRegistry for Action<L>
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

            fn run(
                &self,
                name: &str,
                params: &Arc<dyn Any + Send + Sync>
            ) -> Arc<dyn Any + Send + Sync>
            {
                match name {
                    #(#run)*
                    _ => Arc::new(())
                }
            }

            fn serialize<S>(
                &self,
                name: &str,
                result: &Arc<dyn Any + Send + Sync>,
                serializer: S
            ) -> Option<Result<S::Ok, S::Error>>
            where
                S: serde::ser::Serializer
            {
                match name {
                    #(#serialize)*
                    _ => None
                }
            }
        }
    }
}

fn fields(a: A) -> TokenStream {
    let A {
        field,
        params,
        result,
        ..
    } = a;
    quote::quote! {
        #field: linearf::action::Action<#params, #result>,
    }
}

fn new_fields(a: A) -> TokenStream {
    let A { field, path, .. } = a;
    quote::quote! {
        #field: <#path<L> as NewAction<L>>::new(linearf.clone()),
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

fn run(a: A) -> TokenStream {
    let A {
        name,
        field,
        params,
        ..
    } = a;
    let p = quote::quote! {
        if params.is::<#params>() {
            let params: &Arc<#params> = unsafe { std::mem::transmute(params) };
            Arc::new(t.run(params))
        } else {
            log::error!("mismatch action run params");
            Arc::new(())
        }
    };
    quote::quote! {
        #name => match &self.#field {
            linearf::action::Action::Simple(t) => { #p }
        },
    }
}

fn serialize(a: A) -> TokenStream {
    let A {
        name,
        field,
        result,
        ..
    } = a;
    let p = quote::quote! {
        if result.is::<#result>() {
            let result: &Arc<#result> = unsafe { std::mem::transmute(result) };
            Some(result.serialize(serializer))
        } else {
            log::error!("mismatch action result");
            None
        }
    };
    quote::quote! {
        #name => match &self.#field {
            linearf::action::Action::Simple(_t) => { #p }
        },
    }
}
