#![feature(arc_new_cyclic)]

mod lnf;
mod wrapper;

use crate::{lnf::Lnf, wrapper::Wrapper};
use linearf::*;
use mlua::{prelude::*, serde::Deserializer as LuaDeserializer};
use serde::Deserialize;
use std::sync::Arc;
use tokio::runtime::Runtime;

const RT: &str = "_lienarf_rt";
const LINEARF: &str = "_linearf_linearf";

#[mlua::lua_module]
fn bridge(lua: &Lua) -> LuaResult<LuaTable> {
    initialize_log().map_err(LuaError::external)?;
    let rt = Runtime::new()?;
    let st = State::new_shared();
    let lnf = Lnf::new(st, rt.handle().clone());
    {
        lua.globals()
            .raw_set(RT, lua.create_userdata(Wrapper::new(rt))?)?;
        lua.set_named_registry_value(LINEARF, Wrapper::new(lnf))?;
    }
    let exports = lua.create_table()?;
    exports.set("format_error", lua.create_function(format_error)?)?;
    exports.set("run", lua.create_function(run)?)?;
    exports.set("tick", lua.create_function(tick)?)?;
    exports.set("resume", lua.create_function(resume)?)?;
    exports.set("flow_status", lua.create_function(flow_status)?)?;
    Ok(exports)
}

fn format_error(_lua: &Lua, (name, e): (LuaString, LuaError)) -> LuaResult<String> {
    log::error!("[{}] {:?}", name.to_string_lossy(), &e);
    Ok(format!("{:?}", e))
}

fn run<'a>(lua: &'a Lua, senario: LuaTable) -> LuaResult<LuaTable<'a>> {
    start_flow(lua, None, senario)
}

fn tick<'a>(lua: &'a Lua, (id, senario): (i32, LuaTable)) -> LuaResult<LuaTable<'a>> {
    start_flow(lua, Some(id), senario)
}

fn start_flow<'a>(lua: &'a Lua, id: Option<i32>, senario: LuaTable) -> LuaResult<LuaTable<'a>> {
    let senario = senario_deserializer(senario)?;
    let req = state::StartFlow {
        id: id.map(state::SessionId),
        senario
    };
    let lnf: Wrapper<Arc<Lnf>> = lua.named_registry_value(LINEARF)?;
    let (sid, fid) = lnf.runtime().block_on(async {
        let state = &mut lnf.state().write().await;
        state
            .start_flow(
                lnf.runtime().clone(),
                lnf.source(),
                lnf.matcher(),
                lnf.converter(),
                req
            )
            .map_err(|b| LuaError::ExternalError(Arc::from(b)))
    })?;
    {
        let t = lua.create_table()?;
        t.set("session", sid.0)?;
        t.set("flow", fid.0)?;
        Ok(t)
    }
}

fn senario_deserializer(senario: LuaTable) -> LuaResult<state::Senario<Vars, LuaDeserializer>> {
    let vars = Vars::deserialize(LuaDeserializer::new(mlua::Value::Table(
        senario.raw_get::<_, LuaTable>("linearf")?
    )))?;
    let source = LuaDeserializer::new(senario.raw_get::<_, mlua::Value>("source")?);
    let matcher = LuaDeserializer::new(senario.raw_get::<_, mlua::Value>("matcher")?);
    Ok(state::Senario {
        linearf: vars,
        source,
        matcher
    })
}

fn resume(lua: &Lua, id: i32) -> LuaResult<usize> {
    let id = state::SessionId(id);
    let lnf: Wrapper<Arc<Lnf>> = lua.named_registry_value(LINEARF)?;
    lnf.runtime().block_on(async {
        let state = &mut lnf.state().write().await;
        let id = state
            .resume(id)
            .map_err(|b| LuaError::ExternalError(Arc::from(b)))?;
        Ok(id.0)
    })
}

fn flow_status(lua: &Lua, (s, f): (i32, usize)) -> LuaResult<Option<LuaTable<'_>>> {
    let s = state::SessionId(s);
    let f = state::FlowId(f);
    let lnf: Wrapper<Arc<Lnf>> = lua.named_registry_value(LINEARF)?;
    lnf.runtime().block_on(async {
        let state = &mut lnf.state().read().await;
        if let Some(flow) = state.get_flow(s, f) {
            let (done, count) = flow.sorted_status().await;
            let t = lua.create_table()?;
            t.set("done", done)?;
            t.set("count", count)?;
            Ok(Some(t))
        } else {
            Ok(None)
        }
    })
}

// fn flow_status(lua: &Lua, (s, f): (i32, usize)) -> LuaResult<Option<LuaTable<'_>>> {

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
