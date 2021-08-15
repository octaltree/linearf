use mlua::prelude::*;
use std::{cell::RefMut, process::Command};
use tokio::runtime::Runtime;

#[mlua::lua_module]
fn bridge(lua: &Lua) -> LuaResult<LuaTable> {
    initialize_log().unwrap();
    let exports = lua.create_table()?;
    let rt = tokio::runtime::Runtime::new()?;
    exports.set("spawn", lua.create_function(spawn)?)?;
    exports.set("send", lua.create_function(send)?)?;
    lua.globals()
        .raw_set("_rt", lua.create_userdata(UserDataWrapper::new(rt))?)?;
    Ok(exports)
}

fn spawn(lua: &Lua, _: ()) -> LuaResult<()> {
    let r: LuaAnyUserData = lua.globals().raw_get("_rt")?;
    let rt = r.borrow_mut::<UserDataWrapper<Runtime>>()?;
    log::debug!("bar");
    rt.spawn(async {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        log::debug!("foo");
    });
    Ok(())
}

fn send(lua: &Lua, xs: LuaString) -> LuaResult<()> { Ok(()) }

#[cfg(debug_assertions)]
fn initialize_log() -> Result<(), Box<dyn std::error::Error>> {
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

#[cfg(not(debug_assertions))]
fn initialize_log() -> Result<(), Box<dyn std::error::Error>> { Ok(()) }

struct UserDataWrapper<T>(T);

impl<T> LuaUserData for UserDataWrapper<T> {}

impl<T> std::ops::Deref for UserDataWrapper<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<T> UserDataWrapper<T> {
    fn new(inner: T) -> Self { Self(inner) }
}
