#![feature(arc_new_cyclic)]

mod lnf;
mod sorted;
mod wrapper;

use crate::{lnf::Lnf, wrapper::Wrapper};
use linearf::*;
use mlua::{prelude::*, serde::Deserializer as LuaDeserializer};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc
};
use tokio::runtime::Runtime;

const RT: &str = "_lienarf_rt";
const LINEARF: &str = "_linearf_linearf";

#[macros::lua_module]
fn linearf_bridge(lua: &Lua) -> LuaResult<LuaTable> {
    initialize().map_err(LuaError::external)?;
    let rt = Runtime::new()?;
    let st = State::new_shared();
    let lnf = Lnf::new(st, rt.handle().clone());
    {
        lua.globals()
            .raw_set(RT, lua.create_userdata(Wrapper::new(rt))?)?;
        lua.set_named_registry_value(LINEARF, Wrapper::new(lnf))?;
    }
    let exports = lua.create_table()?;
    exports.set("run", lua.create_function(run)?)?;
    exports.set("tick", lua.create_function(tick)?)?;
    exports.set("resume", lua.create_function(resume)?)?;
    exports.set("flow_status", lua.create_function(sorted::flow_status)?)?;
    exports.set("flow_items", lua.create_function(sorted::flow_items)?)?;
    exports.set("flow_view", lua.create_function(sorted::flow_view)?)?;
    exports.set("pid", lua.create_function(sorted::pid)?)?;
    exports.set("remove_session", lua.create_function(remove_session)?)?;
    exports.set("inspect_error", lua.create_function(inspect_error)?)?;
    exports.set(
        "remove_all_sessions_later",
        lua.create_function(remove_all_sessions_later)?
    )?;
    exports.set(
        "remove_all_sessions",
        lua.create_function(remove_all_sessions)?
    )?;
    exports.set("clean_dir", lua.create_function(clean_dir)?)?;
    Ok(exports)
}

pub(crate) fn dir() -> PathBuf { std::env::temp_dir().join("vim_linearf") }

fn initialize() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let dir = dir();
    fs::create_dir_all(&dir)?;
    let log = dir.join("vim_linearf.log");
    initialize_log(&log)?;
    log::info!("initialize");
    Ok(())
}

fn initialize_log(file: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use log::LevelFilter;
    use log4rs::{
        append::file::FileAppender,
        config::{Appender, Config, Root}
    };
    let logfile = FileAppender::builder().build(file)?;
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(
            Root::builder()
                .appender("logfile")
                .build(LevelFilter::Trace)
        )?;
    log4rs::init_config(config)?;
    Ok(())
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
            .await
            .map_err(LuaError::external)
    })?;
    {
        let t = lua.create_table_with_capacity(0, 2)?;
        t.set("session", sid.0)?;
        t.set("flow", fid.0)?;
        Ok(t)
    }
}

fn senario_deserializer(senario: LuaTable) -> LuaResult<state::Senario<Vars, LuaDeserializer>> {
    let vars = Vars::deserialize(LuaDeserializer::new(LuaValue::Table(
        senario.raw_get::<_, LuaTable>("linearf")?
    )))?;
    let source = LuaDeserializer::new(senario.raw_get::<_, LuaValue>("source")?);
    let matcher = LuaDeserializer::new(senario.raw_get::<_, LuaValue>("matcher")?);
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
        let id = state.resume(id).map_err(LuaError::external)?;
        Ok(id.0)
    })
}

fn remove_session(lua: &Lua, id: i32) -> LuaResult<()> {
    let id = state::SessionId(id);
    let lnf: Wrapper<Arc<Lnf>> = lua.named_registry_value(LINEARF)?;
    lnf.runtime().block_on(async {
        let state = &mut lnf.state().write().await;
        state.remove_session(id);
    });
    Ok(())
}

fn inspect_error<'a>(lua: &'a Lua, (name, e): (LuaString, LuaError)) -> LuaResult<LuaTable<'a>> {
    log::error!("[{}] {:?}", name.to_string_lossy(), &e);
    let related = _is_related_recipe(&e);
    let msg = format!("{:?}", e);
    {
        let t = lua.create_table_with_capacity(0, 2)?;
        t.set("message", msg)?;
        t.set("is_related_recipe", related)?;
        Ok(t)
    }
}

fn _is_related_recipe(e: &LuaError) -> bool {
    use state::Error::*;
    let e = match e {
        LuaError::ExternalError(e) => e,
        LuaError::CallbackError { cause, .. } => return _is_related_recipe(&*cause),
        _ => {
            return false;
        }
    };
    let e = match e.downcast_ref::<state::Error>() {
        Some(e) => e,
        None => return false
    };
    matches!(
        e,
        SourceNotFound(_) | MatcherNotFound(_) | ConverterNotFound(_)
    )
}

fn remove_all_sessions_later(lua: &Lua, (): ()) -> LuaResult<()> {
    let lnf: Wrapper<Arc<Lnf>> = lua.named_registry_value(LINEARF)?;
    let l = Arc::clone(&lnf);
    lnf.runtime().spawn(async move {
        let state = &mut l.state().write().await;
        state.remove_all_sesions();
    });
    Ok(())
}

fn remove_all_sessions(lua: &Lua, (): ()) -> LuaResult<()> {
    let lnf: Wrapper<Arc<Lnf>> = lua.named_registry_value(LINEARF)?;
    lnf.runtime().block_on(async {
        let state = &mut lnf.state().write().await;
        state.remove_all_sesions();
    });
    Ok(())
}

fn clean_dir(lua: &Lua, (): ()) -> LuaResult<()> {
    fs::remove_dir_all(dir()).map_err(LuaError::external)?;
    Ok(())
}
