use mlua::prelude::*;
use std::process::Command;
use tokio::runtime::Runtime;

#[mlua::lua_module]
fn bridge(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    let rt = tokio::runtime::Runtime::new()?;
    exports.set("spawn", lua.create_function(spawn)?)?;
    exports.set("_rt", lua.create_userdata(Dummy(Some(rt)))?)?;
    // let key = lua.create_registry_value(rt)?;
    Ok(exports)
}

fn spawn(_: &Lua, _: ()) -> LuaResult<()> {
    initialize_log().unwrap();
    let mut rt = tokio::runtime::Runtime::new()?;
    log::debug!("bar");
    // TODO
    rt.spawn(async {
        log::debug!("foo");
    });
    Ok(())
}

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

struct Dummy(Option<Runtime>);

impl LuaUserData for Dummy {}

impl From<Runtime> for Dummy {
    fn from(inner: Runtime) -> Self { Self(Some(inner)) }
}

// impl Clone for Dummy {
//    fn clone(&self) -> Self { Self(None) }
//}
