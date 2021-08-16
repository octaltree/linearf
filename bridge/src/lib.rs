mod rpc;

use mlua::prelude::*;
use std::{cell::RefMut, process::Command, rc::Rc};
use tokio::runtime::Runtime;

const RT: &'static str = "_rt";

#[mlua::lua_module]
fn bridge(lua: &Lua) -> LuaResult<LuaTable> {
    initialize_log().unwrap();
    let rt = tokio::runtime::Runtime::new()?;
    lua.globals()
        .raw_set(RT, lua.create_userdata(Wrapper::new(rt))?)?;
    let exports = lua.create_table()?;
    exports.set("spawn", lua.create_function(spawn)?)?;
    exports.set("start_session", lua.create_function(start_session)?)?;
    Ok(exports)
}

fn spawn(lua: &Lua, _: ()) -> LuaResult<()> {
    let any: LuaAnyUserData = lua.globals().raw_get(RT)?;
    let rt: RefMut<Wrapper<Runtime>> = any.borrow_mut()?;
    rt.spawn(async {
        linearf::start().await;
    });
    Ok(())
}

fn start_session(lua: &Lua, source: LuaString) -> LuaResult<i32> {
    let any: LuaAnyUserData = lua.globals().raw_get(RT)?;
    let rt: RefMut<Wrapper<Runtime>> = any.borrow_mut()?;
    let sid = rt.block_on(async { 0 });
    Ok(sid)
}

fn initialize_log() -> Result<(), Box<dyn std::error::Error>> {
    if !cfg!(debug_assertions) {
        return Ok(());
    }
    use log::LevelFilter;
    use log4rs::{
        append::file::FileAppender,
        config::{Appender, Config, Root}
    };
    let logfile = FileAppender::builder().build("/tmp/bridge.log")?;
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(
            Root::builder()
                .appender("logfile")
                .build(LevelFilter::Debug)
        )?;
    log4rs::init_config(config)?;
    log::info!("initialize");
    Ok(())
}

struct Wrapper<T>(T);

impl<T> LuaUserData for Wrapper<T> {}

impl<T> std::ops::Deref for Wrapper<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<T> Wrapper<T> {
    fn new(inner: T) -> Self { Self(inner) }
}
