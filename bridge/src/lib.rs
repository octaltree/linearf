use linearf::{Flow, State};
use mlua::prelude::*;
use std::{cell::RefMut, process::Command, rc::Rc, sync::Arc};
use tokio::{
    runtime::{Handle, Runtime},
    sync::RwLock
};

const RT: &'static str = "_lienarf_rt";
const ST: &'static str = "_linearf_state";

#[mlua::lua_module]
fn bridge(lua: &Lua) -> LuaResult<LuaTable> {
    initialize_log().unwrap();
    let rt = Runtime::new()?;
    let st = State::new_shared(rt.handle().clone());
    lua.globals()
        .raw_set(RT, lua.create_userdata(Wrapper::new(rt))?)?;
    lua.globals()
        .raw_set(ST, lua.create_userdata(Wrapper::new(st))?)?;
    let exports = lua.create_table()?;
    exports.set("start", lua.create_function(start)?)?;
    exports.set("terminate", lua.create_function(terminate)?)?;
    exports.set("count", lua.create_function(count)?)?;
    Ok(exports)
}

fn start(lua: &Lua, flow: LuaString) -> LuaResult<Option<i32>> {
    let any: LuaAnyUserData = lua.globals().raw_get(RT)?;
    let rt: RefMut<Wrapper<Runtime>> = any.borrow_mut()?;
    let any: LuaAnyUserData = lua.globals().raw_get(ST)?;
    let st: &Arc<RwLock<State>> = &**any.borrow_mut::<Wrapper<Arc<RwLock<State>>>>()?;
    let name = flow.to_string_lossy();
    rt.block_on(async {
        let st = &mut st.write().await;
        let id = st.start_session(&name).await.map(|(id, _)| id);
        Ok(id)
    })
}

fn terminate(lua: &Lua, session: i32) -> LuaResult<()> { Ok(()) }

fn count(lua: &Lua, session: i32) -> LuaResult<Option<usize>> {
    let any: LuaAnyUserData = lua.globals().raw_get(RT)?;
    let rt: RefMut<Wrapper<Runtime>> = any.borrow_mut()?;
    let any: LuaAnyUserData = lua.globals().raw_get(ST)?;
    let st: &Arc<RwLock<State>> = &**any.borrow_mut::<Wrapper<Arc<RwLock<State>>>>()?;
    rt.block_on(async {
        let st = st.read().await;
        if let Some(s) = st.session(session).await {
            Ok(Some(s.count()))
        } else {
            Ok(None)
        }
    })
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
