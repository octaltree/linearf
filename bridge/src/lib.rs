mod rpc;

use linearf::{Flow, State};
use mlua::prelude::*;
use std::{cell::RefMut, process::Command, rc::Rc, sync::Arc};
use tokio::{
    runtime::{Handle, Runtime},
    sync::RwLock
};

const RT: &'static str = "_rt";
const ST: &'static str = "_state";

#[mlua::lua_module]
fn bridge(lua: &Lua) -> LuaResult<LuaTable> {
    initialize_log().unwrap();
    let rt = Runtime::new()?;
    let handle = rt.handle().clone();
    let st = State::new_shared(rt);
    lua.globals()
        .raw_set(RT, lua.create_userdata(Wrapper::new(handle))?)?;
    lua.globals()
        .raw_set(ST, lua.create_userdata(Wrapper::new(st))?)?;
    let exports = lua.create_table()?;
    exports.set("start_session", lua.create_function(start_session)?)?;
    Ok(exports)
}

// fn spawn(lua: &Lua, _: ()) -> LuaResult<()> {
//    let any: LuaAnyUserData = lua.globals().raw_get(RT)?;
//    let rt: RefMut<Wrapper<Runtime>> = any.borrow_mut()?;
//    let st = {
//        let any: LuaAnyUserData = lua.globals().raw_get(ST)?;
//        let st: RefMut<Wrapper<Arc<RwLock<State>>>> = any.borrow_mut()?;
//        Arc::clone(&**st)
//    };
//    rt.spawn(async {});
//    Ok(())
//}

fn shutdown(lua: &Lua, _: ()) -> LuaResult<()> { Ok(()) }

fn start_session(lua: &Lua, flow: LuaString) -> LuaResult<Option<i32>> {
    let any: LuaAnyUserData = lua.globals().raw_get(RT)?;
    let rt: RefMut<Wrapper<Handle>> = any.borrow_mut()?;
    let st = {
        let any: LuaAnyUserData = lua.globals().raw_get(ST)?;
        let st: RefMut<Wrapper<Arc<RwLock<State>>>> = any.borrow_mut()?;
        Arc::clone(&**st)
    };
    let name = flow.to_string_lossy();
    rt.block_on(async {
        let st = &mut st.write().await;
        let id = st.start_session(&name).await.map(|(id, _)| id);
        Ok(id)
    })
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
