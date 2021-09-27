use crate::Recipe;
use proc_macro2::{Ident, TokenStream};

struct A {
    name: String,
    field: Ident,
    path: TokenStream,
    params: TokenStream,
    sender: Ident
}

pub fn format(recipe: &Recipe) -> TokenStream {
    let s = recipe.sources.iter().map(|s| {
        let field = quote::format_ident!("{}", &s.name);
        let p = s.path.split("::").map(|p| quote::format_ident!("{}", p));
        let path = quote::quote! { #(#p)::* };
        let params = quote::quote! { <#path as IsSource>::Params };
        let sender = quote::format_ident!("{}_sender", &s.name);
        A {
            name: s.name.clone(),
            field,
            path,
            params,
            sender
        }
    });
    macro_rules! let_sources {
        ($($i:ident),*) => {
            $(let $i = s.clone().map($i);)*
        };
    }
    let_sources! {fields, new_fields, parses, reusable, on_session_start, on_flow_start}
    quote::quote! {
        use linearf::{Shared, New, Vars, RwLock, AsyncRt};
        use linearf::session::{Sender, new_channel};
        use linearf::source::*;
        use std::sync::Arc;
        use std::any::Any;
        use serde::Deserialize;
        use async_trait::async_trait;

        pub struct Source {
            #(#fields)*
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

            async fn on_session_start(
                &self,
                rt: &AsyncRt,
                name: &str,
                tx: linearf::source::Transmitter,
                senario: (Arc<Vars>, Arc<dyn Any + Send + Sync>)
            ) where
                Self: Sized
            {
                match name {
                    #(#on_session_start)*
                    _ => {}
                }
            }

            async fn on_flow_start(
                &self,
                rt: &AsyncRt,
                name: &str,
                tx: linearf::source::Transmitter,
                senario: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>)
            ) -> bool
            where
                Self: Sized
            {
                match name {
                    #(#on_flow_start)*
                    _ => false
                }
            }
        }
    }
}

fn fields(a: A) -> TokenStream {
    let A {
        field,
        params,
        sender,
        ..
    } = a;
    quote::quote! {
        #field: linearf::source::Source<#params>,
        #sender: RwLock<Option<Sender<(Transmitter, (Arc<Vars>, Arc<#params>))>>>,
    }
}

fn new_fields(a: A) -> TokenStream {
    let A {
        field,
        path,
        sender,
        ..
    } = a;
    quote::quote! {
        #field: <#path as New>::new(&state).into_source(),
        #sender: RwLock::new(None),
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
        },
    }
}

fn on_session_start(a: A) -> TokenStream {
    let A {
        name,
        field,
        sender,
        params,
        ..
    } = a;
    let pre = quote::quote! {
        let (senario_vars, senario_source) = senario;
        if !senario_source.is::<#params>() {
            log::error!("mismatch params type");
            return; // drop and close the channel
        }
        let (senario_source, _): (Arc<#params>, usize) =
            unsafe { std::mem::transmute(senario_source) };
    };
    quote::quote! {
        #name => match self.#field.clone() {
            linearf::source::Source::Simple(g) => {
                #pre
                rt.spawn(async move {
                    let start = std::time::Instant::now();
                    let g = g.read().await;
                    g.generate(tx, (&senario_vars, &senario_source)).await;
                    log::debug!("source {:?}", std::time::Instant::now() - start);
                });
            }
            linearf::source::Source::Flow(g) => {
                #pre
                let is_initialized = {
                    let s = self.#sender.read().await;
                    s.is_some()
                };
                if !is_initialized {
                    let mut place = self.#sender.write().await;
                    if place.is_none() {
                        let (tx, rx) = new_channel();
                        *place = Some(tx);
                        let rt2 = rt.clone();
                        rt.spawn_blocking(move || {
                            rt2.block_on(async {
                                let mut g = g.write().await;
                                g.run(rx).await;
                            });
                        });
                    }
                }
                if let Some(sender) = &*self.#sender.read().await {
                    let s = (senario_vars, senario_source);
                    if let Err(e) = sender.send((tx, s)).await {
                        log::error!("{:?}", e);
                    }
                }
            }
        },
    }
}

fn on_flow_start(a: A) -> TokenStream {
    let A {
        name,
        field,
        sender,
        params,
        ..
    } = a;
    let pre = quote::quote! {
        let (senario_vars, senario_source) = senario;
        if !senario_source.is::<#params>() {
            log::error!("mismatch params type");
            return false; // drop and close the channel
        }
        let senario_source: &Arc<#params> =
            unsafe { std::mem::transmute(senario_source) };
    };
    quote::quote! {
        #name => match self.#field.clone() {
            linearf::source::Source::Simple(_) => {
                false
            }
            linearf::source::Source::Flow(g) => {
                #pre
                // unwrap: sender must be set on session start
                let maybe_sender = self.#sender.read().await;
                let sender = maybe_sender.as_ref().expect("FlowGenerator is not running");
                let s = (senario_vars.clone(), senario_source.clone());
                if let Err(e) = sender.send((tx, s)).await {
                    log::error!("{:?}", e);
                }
                true
            }
        },
    }
}
