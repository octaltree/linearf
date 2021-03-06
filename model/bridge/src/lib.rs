mod lnf;
mod sorted;
mod wrapper;

use crate::{lnf::Lnf, wrapper::Wrapper};
use linearf::*;
use mlua::{
    prelude::*,
    serde::{Deserializer as LuaDeserializer, Serializer}
};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc
};
use tokio::runtime::Runtime;

const RT: &str = "_linearf_rt";
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
    exports.set("flow_id_items", lua.create_function(sorted::flow_id_items)?)?;
    exports.set("remove_session", lua.create_function(remove_session)?)?;
    exports.set("session_ids", lua.create_function(session_ids)?)?;
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
    exports.set("dispatch_action", lua.create_function(dispatch_action)?)?;
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

fn run<'a>(lua: &'a Lua, scenario: LuaTable) -> LuaResult<LuaTable<'a>> {
    start_flow(lua, None, scenario)
}

fn tick<'a>(lua: &'a Lua, (id, scenario): (i32, LuaTable)) -> LuaResult<LuaTable<'a>> {
    start_flow(lua, Some(id), scenario)
}

fn start_flow<'a>(lua: &'a Lua, id: Option<i32>, scenario: LuaTable) -> LuaResult<LuaTable<'a>> {
    let scenario = scenario_deserializer(scenario)?;
    let req = state::StartFlow {
        id: id.map(state::SessionId),
        scenario
    };
    log::debug!("{:?}", &req.scenario.linearf.query);
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

fn scenario_deserializer(scenario: LuaTable) -> LuaResult<state::Scenario<Vars, LuaDeserializer>> {
    let vars = Vars::deserialize(LuaDeserializer::new_with_options(
        LuaValue::Table(scenario.raw_get::<_, LuaTable>("linearf")?),
        LuaDeserializeOptions::new().deny_unsupported_types(false)
    ))?;
    let source = LuaDeserializer::new(scenario.raw_get::<_, LuaValue>("source")?);
    let matcher = LuaDeserializer::new(scenario.raw_get::<_, LuaValue>("matcher")?);
    Ok(state::Scenario {
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

fn session_ids(lua: &Lua, newer: bool) -> LuaResult<LuaTable> {
    let lnf: Wrapper<Arc<Lnf>> = lua.named_registry_value(LINEARF)?;
    lnf.runtime().block_on(async {
        let state = lnf.state().read().await;
        let it = state.sessions();
        if newer {
            lua.create_sequence_from(it.rev().map(|(i, _)| i.0))
        } else {
            lua.create_sequence_from(it.map(|(i, _)| i.0))
        }
    })
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

fn clean_dir(_lua: &Lua, (): ()) -> LuaResult<()> {
    fs::remove_dir_all(dir()).map_err(LuaError::external)?;
    Ok(())
}

fn dispatch_action<'a>(
    lua: &'a Lua,
    (name, params): (LuaString, LuaValue)
) -> LuaResult<LuaValue<'a>> {
    let name = name.to_string_lossy();
    let d = LuaDeserializer::new(params);
    let lnf: Wrapper<Arc<Lnf>> = lua.named_registry_value(LINEARF)?;
    let registry = lnf.action();
    let p: Arc<_> = registry.parse(&name, d).ok_or_else(|| {
        LuaError::external(ActionError::ActionNotFound(SmartString::from(&*name)))
    })??;
    let r: Arc<_> = registry.run(&name, &p);
    registry
        .serialize(&name, &r, Serializer::new(lua))
        .ok_or_else(|| LuaError::external(ActionError::ActionNotFound(SmartString::from(&*name))))?
}

#[derive(Debug, thiserror::Error)]
pub enum ActionError {
    #[error("Action {0:?} is not found")]
    ActionNotFound(SmartString)
}
