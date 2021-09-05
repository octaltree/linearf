use linearf::{Flow, State};
use mlua::prelude::*;
use std::{cell::RefMut, process::Command, rc::Rc, sync::Arc};
use tokio::{
    runtime::{Handle, Runtime},
    sync::RwLock
};

const RT: &str = "_lienarf_rt";
const ST: &str = "_linearf_state";

#[mlua::lua_module]
fn bridge(lua: &Lua) -> LuaResult<LuaTable> {
    initialize_log().unwrap();
    let rt = Runtime::new()?;
    let st = State::new_shared();
    lua.globals()
        .raw_set(RT, lua.create_userdata(Wrapper::new(rt))?)?;
    lua.set_named_registry_value(ST, Wrapper::new(st))?;
    let exports = lua.create_table()?;
    exports.set("error", lua.create_function(error)?)?;
    exports.set("run", lua.create_function(run)?)?;
    exports.set("terminate", lua.create_function(terminate)?)?;
    exports.set("count", lua.create_function(count)?)?;
    exports.set("change_query", lua.create_function(change_query)?)?;
    Ok(exports)
}

fn error(lua: &Lua, (name, e): (LuaString, LuaError)) -> LuaResult<String> {
    log::error!("[{}] {:?}", name.to_string_lossy(), e);
    Ok(format!("{:?}", e))
}

fn run(lua: &Lua, (selected, args): (LuaString, LuaString)) -> LuaResult<i32> {
    let any: LuaAnyUserData = lua.globals().raw_get(RT)?;
    let rt: RefMut<Wrapper<Runtime>> = any.borrow_mut()?;
    let st = lua.named_registry_value::<_, Wrapper<Arc<RwLock<State>>>>(ST)?;
    rt.block_on(async {
        let handle = rt.handle().clone();
        let st = &mut st.write().await;
        let flow = build_flow(st, args, selected).ok_or(LuaError::external("not found"))?;
        let (id, _) = st.start_session(handle, flow).await;
        Ok(id)
    })
}

fn build_flow(st: &State, args: LuaString, selected: LuaString) -> Option<Arc<Flow>> {
    // TODO
    Some(Arc::new(Flow {}))
}

fn terminate(lua: &Lua, session: i32) -> LuaResult<()> { Ok(()) }

fn count(lua: &Lua, session: i32) -> LuaResult<Option<usize>> {
    let any: LuaAnyUserData = lua.globals().raw_get(RT)?;
    let rt: RefMut<Wrapper<Runtime>> = any.borrow_mut()?;
    let any: LuaAnyUserData = lua.globals().raw_get(ST)?;
    let st = &**any.borrow_mut::<Wrapper<Arc<RwLock<State>>>>()?;
    rt.block_on(async {
        if let Some(l) = st.read().await.session(session).await {
            let s = l.read().await;
            Ok(Some(s.count()))
        } else {
            Ok(None)
        }
    })
}

fn change_query(lua: &Lua, (session, query): (i32, LuaString)) -> LuaResult<()> {
    let q = query.to_string_lossy();
    let any: LuaAnyUserData = lua.globals().raw_get(RT)?;
    let rt: RefMut<Wrapper<Runtime>> = any.borrow_mut()?;
    let any: LuaAnyUserData = lua.globals().raw_get(ST)?;
    let st = &**any.borrow_mut::<Wrapper<Arc<RwLock<State>>>>()?;
    rt.block_on(async move {
        if let Some(l) = st.read().await.session(session).await {
            let s = &mut l.write().await;
            s.change_query(q);
        }
        Ok(())
    })
}

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
                .build(LevelFilter::Trace)
        )?;
    log4rs::init_config(config)?;
    log::info!("initialize");
    Ok(())
}
