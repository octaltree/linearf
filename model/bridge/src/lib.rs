use mlua::prelude::*;
use std::cell::RefMut;
use tokio::runtime::Runtime;

const RT: &str = "_lienarf_rt";

#[mlua::lua_module]
fn bridge(lua: &Lua) -> LuaResult<LuaTable> {
    initialize_log().map_err(LuaError::external)?;
    let rt = Runtime::new()?;
    lua.globals()
        .raw_set(RT, lua.create_userdata(Wrapper::new(rt))?)?;
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
