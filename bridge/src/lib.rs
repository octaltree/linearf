use mlua::prelude::*;
use std::process::Command;
use tokio::runtime::Runtime;

#[mlua::lua_module]
fn bridge(lua: &Lua) -> LuaResult<LuaTable> {
    initialize_log().unwrap();
    let exports = lua.create_table()?;
    let rt = tokio::runtime::Runtime::new()?;
    exports.set("spawn", lua.create_function(spawn)?)?;
    // exports.set("_rt", lua.create_userdata(Dummy(Some(rt)))?)?;
    // let key = lua.create_registry_value(rt)?;
    lua.globals()
        .raw_set("_rt", lua.create_userdata(Dummy(Some(rt)))?)?;
    Ok(exports)
}

fn spawn(lua: &Lua, _: ()) -> LuaResult<()> {
    let mut rt = tokio::runtime::Runtime::new()?;
    let r: LuaAnyUserData = lua.globals().raw_get("_rt")?;
    let dummy = r.borrow_mut::<Dummy>()?;
    let rt = dummy.0.as_ref().unwrap();
    log::debug!("bar");
    // rt.spawn(async {
    //    // tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    //    for i in 0..10000000 {}
    //    log::debug!("foo");
    //});
    std::thread::spawn(|| {
        for i in 0..100000000 {
            log::debug!("{}", i);
        }
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
