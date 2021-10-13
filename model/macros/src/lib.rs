use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use std::env;
use syn::{parse_macro_input, AttributeArgs, Error, ItemFn};

#[proc_macro_attribute]
pub fn lua_module(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as AttributeArgs);
    let func = parse_macro_input!(item as ItemFn);

    if !args.is_empty() {
        let err = Error::new(Span::call_site(), "the macro does not support arguments")
            .to_compile_error();
        return err.into();
    }

    let func_name = func.sig.ident.clone();
    let suffix = match env::var("LINEARF_BRIDGE_SUFFIX") {
        Ok(s) => s,
        Err(env::VarError::NotPresent) => "".to_string(),
        Err(e) => panic!("{:?}", e)
    };
    let ext_entrypoint_name = Ident::new(
        &format!("luaopen_{}{}", func_name, suffix),
        Span::call_site()
    );

    let wrapped = quote! {
        ::mlua::require_module_feature!();

        #func

        #[no_mangle]
        unsafe extern "C" fn #ext_entrypoint_name(state: *mut ::mlua::lua_State) -> ::std::os::raw::c_int {
            ::mlua::Lua::init_from_ptr(state)
                .entrypoint1(#func_name)
                .expect("cannot initialize module")
        }
    };

    wrapped.into()
}
