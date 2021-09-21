use crate::Recipe;
use proc_macro2::TokenStream;

pub fn format(recipe: &Recipe) -> TokenStream {
    struct A<I, T> {
        name: String,
        field: I,
        path: T,
        params: T,
        sender: I
    }
    let s = recipe.sources.iter().map(|s| {
        let field = quote::format_ident!("{}", &s.name);
        let p = s.path.split("::").map(|p| quote::format_ident!("{}", p));
        let path = quote::quote! { #(#p)::* };
        let params = quote::quote! { <#path as HasSourceParams>::Params };
        let sender = quote::format_ident!("{}_sender", &s.name);
        A {
            name: s.name.clone(),
            field,
            path,
            params,
            sender
        }
    });
    let fields = s.clone().map(
        |A {
             name,
             params,
             sender,
             ..
         }| {
            quote::quote! {
                #name: linearf::source::Source<#params>,
                #sender: Option<Sender<(Transmitter, (&Arc<Vars>, &Arc<#params>))>>
            }
        }
    );
    let new_fields = s.clone().map(
        |A {
             name, path, sender, ..
         }| {
            quote::quote! {
                #name: <#path as New>::new(&state).into_source(),
                #sender: None
            }
        }
    );
    let parses = s.clone().map(|A { name, params, .. }| {
        quote::quote! {
            #name => Ok(Some(Arc::new(#params::deserialize(deserializer)?)))
        }
    });
    let reusable = s.clone().map(
        |A {
             name,
             field,
             params,
             ..
         }| {
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
                    g.read().await.reusable(
                        (prev_vars, prev_source), (senario_vars, senario_source)).await
                } else {
                    false
                }
            };
            quote::quote! {
                #name => match &self.#field {
                    linearf::source::Source::Simple(g) => { #p }
                    linearf::source::Source::Flow(g) => { #p }
                }
            }
        }
    );
    let on_session_start = s.clone().map(
        |A {
             name,
             field,
             path,
             sender,
             params,
             ..
         }| {
            // let pre = quote::quote! {
            //    let (senario_vars, senario_source) = senario;
            //    if !senario_source.is::<#params>() {
            //        return; // drop and close the channel
            //    }
            //    let senario_source: &Arc<#params> =
            //        unsafe { std::mem::transmute(senario_source) };
            //};
            // quote::quote! {
            //    #name => match &self.#field {
            //        linearf::source::Source::Simple(g) => {
            //            #pre
            //            g.generate(tx, (senario_vars, senario_source))
            //        }
            //        linearf::source::Source::Flow(g) => {
            //            #pre
            //        }
            //    }
            //}
        }
    );
    quote::quote! {
        use linearf::Shared;
        use linearf::New;
        use linearf::Vars;
        use linearf::session::Sender;
        use linearf::source::{SimpleGenerator, FlowGenerator, HasSourceParams, SourceType, Transmitter};
        use std::sync::Arc;
        use std::any::Any;
        use serde::Deserialize;
        use async_trait::async_trait;

        pub struct Source {
            #(#fields),*,
            state: linearf::Shared<linearf::State>
        }

        #[async_trait]
        impl<'de, D> linearf::source::SourceRegistry<'de, D> for Source
        where
            D: serde::de::Deserializer<'de>
        {
            fn new(state: linearf::Shared<linearf::State>) -> Self
            where
                Self: Sized
            {
                Self { #(#new_fields),*, state }
            }

            fn parse(
                &self,
                name: &str,
                deserializer: D
            ) -> Result<Option<Arc<dyn Any + Send + Sync>>, D::Error> {
                match name {
                    #(#parses),*,
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
                    #(#reusable),*,
                    _ => false
                }
            }

            async fn on_session_start(
                &self,
                name: &str,
                tx: linearf::source::Transmitter,
                senario: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>)
            ) where
                Self: Sized
            {
                match name {
                    _ => {}
                }
            }

            async fn on_flow_start(
                &self,
                name: &str,
                tx: linearf::source::Transmitter,
                senario: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>)
            ) where
                Self: Sized
            {
                match name {
                    _ => {}
                }
            }
        }
    }
}
