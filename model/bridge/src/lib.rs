use linearf::{SourceRegistry, State};
use mlua::prelude::*;
use std::{cell::RefMut, sync::Arc};
use tokio::{runtime::Runtime, sync::RwLock};

const RT: &str = "_lienarf_rt";
const ST: &str = "_linearf_st";
const SOURCE: &str = "_linearf_source";
const MATCHER: &str = "_linearf_matcher";

#[mlua::lua_module]
fn bridge(lua: &Lua) -> LuaResult<LuaTable> {
    initialize_log().map_err(LuaError::external)?;
    let rt = Runtime::new()?;
    let st = State::new_shared();
    let source = Arc::new(<registry::Source as SourceRegistry<
        mlua::serde::Deserializer<'_>
    >>::new(st.clone()));
    {
        lua.globals()
            .raw_set(RT, lua.create_userdata(Wrapper::new(rt))?)?;
        lua.set_named_registry_value(ST, Wrapper::new(st))?;
        lua.set_named_registry_value(SOURCE, Wrapper::new(source))?;
    }
    let exports = lua.create_table()?;
    exports.set("format_error", lua.create_function(format_error)?)?;
    exports.set("run", lua.create_function(run)?)?;
    // exports.set("tick", lua.create_function(tick)?)?;
    Ok(exports)
}

fn format_error(_lua: &Lua, (name, e): (LuaString, LuaError)) -> LuaResult<String> {
    log::error!("[{}] {:?}", name.to_string_lossy(), e);
    Ok(format!("{:?}", e))
}

fn run(lua: &Lua, senario: LuaTable) -> LuaResult<i32> {
    let any: LuaAnyUserData = lua.globals().raw_get(RT)?;
    let rt: RefMut<Wrapper<Runtime>> = any.borrow_mut()?;
    rt.block_on(async { Ok(42) })
}

// fn tick(lua: &Lua, (id, senario): (i32, LuaTable)) -> LuaResult<i32> {
//    let id = linearf::SessionId(id);
//    let any: LuaAnyUserData = lua.globals().raw_get(RT)?;
//    let rt: RefMut<Wrapper<Runtime>> = any.borrow_mut()?;
//    rt.block_on(async { Ok(42) })
//}

#[derive(Clone)]
struct Wrapper<T>(T);

impl<T> LuaUserData for Wrapper<T> {}

impl<T> std::ops::Deref for Wrapper<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<T> Wrapper<T> {
    fn new(inner: T) -> Self { Self(inner) }
}

fn initialize_log() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !(cfg!(unix) || cfg!(debug_assertions)) {
        return Ok(());
    }
    use log::LevelFilter;
    use log4rs::{
        append::file::FileAppender,
        config::{Appender, Config, Root}
    };
    let p = std::env::temp_dir().join("vim_linearf.log");
    let logfile = FileAppender::builder().build(p)?;
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(
            Root::builder()
                .appender("logfile")
                .build(LevelFilter::Trace)
        )?;
    log4rs::init_config(config)?;
    log::info!("initialize");
    Ok(())
}
