#![feature(arc_new_cyclic)]

mod lnf;
mod wrapper;

use crate::{lnf::Lnf, wrapper::Wrapper};
use linearf::{matcher::MatcherRegistry, source::SourceRegistry, Senario, Shared, State, Vars};
use mlua::{prelude::*, serde::Deserializer as LuaDeserializer};
use serde::Deserialize;
use std::{cell::RefMut, sync::Arc};
use tokio::runtime::Runtime;

const RT: &str = "_lienarf_rt";
const ST: &str = "_linearf_st";
const SOURCE: &str = "_linearf_source";
const MATCHER: &str = "_linearf_matcher";

#[mlua::lua_module]
fn bridge(lua: &Lua) -> LuaResult<LuaTable> {
    initialize_log().map_err(LuaError::external)?;
    let rt = Runtime::new()?;
    let st = State::new_shared();
    let source = todo!();
    let matcher = todo!();
    let lnf = Lnf::new(st.clone(), rt.handle().clone());
    {
        lua.globals()
            .raw_set(RT, lua.create_userdata(Wrapper::new(rt))?)?;
        lua.set_named_registry_value(ST, Wrapper::new(st))?;
        lua.set_named_registry_value(SOURCE, Wrapper::new(source))?;
        lua.set_named_registry_value(MATCHER, Wrapper::new(matcher))?;
    }
    let exports = lua.create_table()?;
    exports.set("format_error", lua.create_function(format_error)?)?;
    exports.set("run", lua.create_function(run)?)?;
    exports.set("tick", lua.create_function(tick)?)?;
    exports.set("resume", lua.create_function(resume)?)?;
    Ok(exports)
}

fn format_error(_lua: &Lua, (name, e): (LuaString, LuaError)) -> LuaResult<String> {
    log::error!("[{}] {:?}", name.to_string_lossy(), &e);
    Ok(format!("{:?}", e))
}

fn run<'a>(lua: &'a Lua, senario: LuaTable) -> LuaResult<LuaTable<'a>> {
    let senario = senario_deserializer(senario)?;
    let any: LuaAnyUserData = lua.globals().raw_get(RT)?;
    let rt: RefMut<Wrapper<Runtime>> = any.borrow_mut()?;
    let st: Wrapper<Shared<State>> = lua.named_registry_value(ST)?;
    let source: Wrapper<Arc<registry::Source>> = lua.named_registry_value(SOURCE)?;
    let matcher: Wrapper<Arc<registry::Matcher>> = lua.named_registry_value(MATCHER)?;
    let (sid, fid) = rt.block_on(async {
        let handle = rt.handle().clone();
        let state = &mut st.write().await;
        let (sid, fid) = state
            .start_session(handle, (*source).clone(), (*matcher).clone(), senario)
            .await
            .map_err(|b| LuaError::ExternalError(Arc::from(b)))?;
        Ok::<_, LuaError>((sid.0, fid.0))
    })?;
    {
        let t = lua.create_table()?;
        t.set("session", sid)?;
        t.set("flow", fid)?;
        Ok(t)
    }
}

fn tick<'a>(lua: &'a Lua, (id, senario): (i32, LuaTable)) -> LuaResult<LuaTable<'a>> {
    let id = linearf::SessionId(id);
    let senario = senario_deserializer(senario)?;
    let any: LuaAnyUserData = lua.globals().raw_get(RT)?;
    let rt: RefMut<Wrapper<Runtime>> = any.borrow_mut()?;
    let st: Wrapper<Shared<State>> = lua.named_registry_value(ST)?;
    let source: Wrapper<Arc<registry::Source>> = lua.named_registry_value(SOURCE)?;
    let matcher: Wrapper<Arc<registry::Matcher>> = lua.named_registry_value(MATCHER)?;
    let (sid, fid) = rt.block_on(async {
        let handle = rt.handle().clone();
        let state = &mut st.write().await;
        let (sid, fid) = state
            .tick(handle, (*source).clone(), (*matcher).clone(), id, senario)
            .await
            .map_err(|b| LuaError::ExternalError(Arc::from(b)))?;
        Ok::<_, LuaError>((sid.0, fid.0))
    })?;
    {
        let t = lua.create_table()?;
        t.set("session", sid)?;
        t.set("flow", fid)?;
        Ok(t)
    }
}

fn resume(lua: &Lua, id: i32) -> LuaResult<i32> {
    let id = linearf::SessionId(id);
    let any: LuaAnyUserData = lua.globals().raw_get(RT)?;
    let rt: RefMut<Wrapper<Runtime>> = any.borrow_mut()?;
    let st: Wrapper<Shared<State>> = lua.named_registry_value(ST)?;
    rt.block_on(async {
        let state = &mut st.write().await;
        let id = state
            .resume(id)
            .await
            .map_err(|b| LuaError::ExternalError(Arc::from(b)))?;
        Ok(id.0)
    })
}

fn senario_deserializer(senario: LuaTable) -> LuaResult<Senario<Vars, LuaDeserializer>> {
    let vars = linearf::Vars::deserialize(LuaDeserializer::new(mlua::Value::Table(
        senario.raw_get::<_, LuaTable>("linearf")?
    )))?;
    let source = LuaDeserializer::new(senario.raw_get::<_, mlua::Value>("source")?);
    let matcher = LuaDeserializer::new(senario.raw_get::<_, mlua::Value>("matcher")?);
    Ok(Senario {
        linearf: vars,
        source,
        matcher
    })
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
