#![feature(arc_new_cyclic)]

mod lnf;
mod value;
mod wrapper;

use crate::{lnf::Lnf, wrapper::Wrapper};
use linearf::{item::MaybeUtf8, *};
use mlua::{prelude::*, serde::Deserializer as LuaDeserializer};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::runtime::Runtime;

const RT: &str = "_lienarf_rt";
const LINEARF: &str = "_linearf_linearf";

#[mlua::lua_module]
fn linearf_bridge(lua: &Lua) -> LuaResult<LuaTable> {
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
    exports.set("flow_items", lua.create_function(flow_items)?)?;
    exports.set("remove_session", lua.create_function(remove_session)?)?;
    exports.set("is_related_recipe", lua.create_function(is_related_recipe)?)?;
    Ok(exports)
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

fn flow_status(lua: &Lua, (s, f): (i32, usize)) -> LuaResult<Option<LuaTable<'_>>> {
    let s = state::SessionId(s);
    let f = state::FlowId(f);
    let lnf: Wrapper<Arc<Lnf>> = lua.named_registry_value(LINEARF)?;
    lnf.runtime().block_on(async {
        let state = &lnf.state().read().await;
        let flow = match state.get_flow(s, f) {
            Some(flow) => flow,
            None => return Ok(None)
        };
        let (done, count) = flow.sorted_status().await;
        let t = lua.create_table_with_capacity(0, 2)?;
        t.set("done", done)?;
        t.set("count", count)?;
        Ok(Some(t))
    })
}

fn flow_items(
    lua: &Lua,
    (s, f, start, end): (i32, usize, usize, usize)
) -> LuaResult<LuaValue<'_>> {
    let s = state::SessionId(s);
    let f = state::FlowId(f);
    let lnf: Wrapper<Arc<Lnf>> = lua.named_registry_value(LINEARF)?;
    let items = lnf.runtime().block_on(async {
        let state = &lnf.state().read().await;
        let flow = match state.get_flow(s, f) {
            Some(flow) => flow,
            None => {
                let msg = format!("flow {:?} {:?} not found", s, f);
                return Err(LuaError::external(msg));
            }
        };
        Ok(flow.sorted_items(start, end).await)
    })?;
    #[derive(Serialize)]
    struct I<'a> {
        id: u32,
        r#type: &'a str,
        value: LuaString<'a>,
        info: LuaValue<'a>,
        view: std::borrow::Cow<'a, str>
    }
    let xs = items
        .iter()
        .map(|i| {
            Ok(I {
                id: i.id,
                r#type: i.r#type,
                value: maybe_utf8_into_lua_string(lua, &i.value)?,
                info: value::convert_info(lua, &i.info)?,
                view: i.view()
            })
        })
        .collect::<LuaResult<Vec<_>>>()?;
    let v = lua.to_value_with(
        &xs,
        // serialize as nil
        mlua::SerializeOptions::new()
            .serialize_none_to_null(false)
            .serialize_unit_to_null(false)
    )?;
    Ok(v)
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

fn maybe_utf8_into_lua_string<'a>(lua: &'a Lua, s: &MaybeUtf8) -> LuaResult<LuaString<'a>> {
    use os_str_bytes::OsStrBytes;
    match s {
        MaybeUtf8::Utf8(s) => lua.create_string(s),
        MaybeUtf8::Bytes(b) => lua.create_string(b),
        MaybeUtf8::Os(s) => lua.create_string(&s.to_raw_bytes())
    }
}

fn is_related_recipe(lua: &Lua, e: LuaError) -> LuaResult<bool> { Ok(_is_related_recipe(&e)) }

fn _is_related_recipe(e: &LuaError) -> bool {
    use state::Error::*;
    use std::any::Any;
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
    match &e {
        SourceNotFound(_) | MatcherNotFound(_) | ConverterNotFound(_) => true,
        _ => false
    }
}
