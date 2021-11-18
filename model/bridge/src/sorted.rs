use crate::{lnf::Lnf, wrapper::Wrapper, LINEARF};
use linearf::{
    item::{Item, MaybeUtf8},
    state,
    state::WithScore,
    Linearf
};
use mlua::prelude::*;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::{collections::HashSet, sync::Arc};

pub fn flow_status(lua: &Lua, (s, f): (i32, usize)) -> LuaResult<LuaTable<'_>> {
    let s = state::SessionId(s);
    let f = state::FlowId(f);
    let lnf: Wrapper<Arc<Lnf>> = lua.named_registry_value(LINEARF)?;
    lnf.runtime().block_on(async {
        let state = lnf.state().read().await;
        let flow = state.try_get_flow(s, f).map_err(LuaError::external)?;
        let sorted = flow
            .sorted()
            .await
            .ok_or(state::Error::FlowDisposed(s, f))
            .map_err(LuaError::external)?;
        let (done, hit_count, source_count) =
            (sorted.done, sorted.items.len(), sorted.source_count);
        std::mem::drop(sorted);
        std::mem::drop(state);
        let t = lua.create_table_with_capacity(0, 2)?;
        t.set("done", done)?;
        t.set("count", hit_count)?;
        t.set("source_count", source_count)?;
        Ok(t)
    })
}

pub fn flow_items<'a>(
    lua: &'a Lua,
    (s, f, ranges, fields): (i32, usize, LuaValue, LuaValue)
) -> LuaResult<LuaTable<'a>> {
    let s = state::SessionId(s);
    let f = state::FlowId(f);
    let ranges: Vec<(usize, usize)> = lua.from_value(ranges)?;
    let fields: Fields = lua.from_value(fields)?;
    let lnf: Wrapper<Arc<Lnf>> = lua.named_registry_value(LINEARF)?;
    lnf.runtime().block_on(async {
        let state = lnf.state().read().await;
        let flow = state.try_get_flow(s, f).map_err(LuaError::external)?;
        let sorted = flow
            .sorted()
            .await
            .ok_or(state::Error::FlowDisposed(s, f))
            .map_err(LuaError::external)?;
        let it = ranges
            .into_iter()
            .map(|(s, e)| &sorted.items[s..std::cmp::min(e, sorted.items.len())]);
        {
            let t = convert(lua, fields, it)?;
            t.set("done", sorted.done)?;
            t.set("count", sorted.items.len())?;
            t.set("source_count", sorted.source_count)?;
            Ok(t)
        }
    })
}

pub fn flow_id_items<'a>(
    lua: &'a Lua,
    (s, f, ids, fields): (i32, usize, LuaValue, LuaValue)
) -> LuaResult<LuaTable<'a>> {
    let s = state::SessionId(s);
    let f = state::FlowId(f);
    let ids: Vec<u32> = lua.from_value(ids)?;
    let ids: HashSet<u32> = ids.into_iter().collect();
    let fields: Fields = lua.from_value(fields)?;
    let lnf: Wrapper<Arc<Lnf>> = lua.named_registry_value(LINEARF)?;
    lnf.runtime().block_on(async {
        let state = lnf.state().read().await;
        let flow = state.try_get_flow(s, f).map_err(LuaError::external)?;
        let sorted = flow
            .sorted()
            .await
            .ok_or(state::Error::FlowDisposed(s, f))
            .map_err(LuaError::external)?;
        let it = sorted.id_items(&ids);
        let items: Vec<_> = it.collect();
        let t = convert_items(lua, fields, &items)?;
        Ok(t)
    })
}

#[derive(Deserialize, Clone, Copy)]
struct Fields {
    #[serde(default)]
    id: bool,
    #[serde(default)]
    value: bool,
    #[serde(default)]
    info: bool,
    #[serde(default)]
    view: bool
}

fn convert_items<'a>(lua: &'a Lua, fields: Fields, xs: &[WithScore]) -> LuaResult<LuaTable<'a>> {
    let t = lua.create_table_with_capacity(xs.len().try_into().unwrap(), 0)?;
    for (i, (item, _)) in xs.iter().enumerate() {
        t.raw_insert(
            (i + 1).try_into().unwrap(),
            convert_item(lua, fields, item)?
        )?;
    }
    Ok(t)
    // lua stack overflow 7999slots
    // lua.create_sequence_from(
    //    xs.iter()
    //        .map(|(i, _)| convert_item(lua, fields, i))
    //        .collect::<Result<Vec<_>, _>>()?
    //)
}

fn convert<'a, 'b>(
    lua: &'a Lua,
    fields: Fields,
    it: impl Iterator<Item = &'b [WithScore]>
) -> LuaResult<LuaTable<'a>> {
    lua.create_sequence_from(
        it.map(|xs| -> LuaResult<_> { convert_items(lua, fields, xs) })
            .collect::<Result<Vec<_>, _>>()?
    )
}

fn convert_item<'a>(lua: &'a Lua, fields: Fields, i: &Item) -> LuaResult<LuaTable<'a>> {
    let ret = lua.create_table_with_capacity(0, 5)?;
    if fields.id {
        ret.set("id", i.id)?;
    }
    if fields.value {
        ret.set("value", maybe_utf8_into_lua_string(lua, &i.value)?)?;
    }
    if fields.info {
        ret.set("info", convert_info(lua, &i.info)?)?;
    }
    if fields.view {
        ret.set("view", i.view())?;
    }
    Ok(ret)
}

fn maybe_utf8_into_lua_string<'a>(lua: &'a Lua, s: &MaybeUtf8) -> LuaResult<LuaString<'a>> {
    use os_str_bytes::OsStrBytes;
    match s {
        MaybeUtf8::Utf8(s) => lua.create_string(s),
        MaybeUtf8::Bytes(b) => lua.create_string(b),
        MaybeUtf8::Os(s) => lua.create_string(&s.to_raw_bytes())
    }
}

fn convert_info<'a>(lua: &'a Lua, value: &Option<Map<String, Value>>) -> LuaResult<LuaValue<'a>> {
    if let Some(m) = value {
        convert_map(lua, m)
    } else {
        Ok(LuaValue::Nil)
    }
}

fn convert_value<'a>(lua: &'a Lua, v: &Value) -> LuaResult<LuaValue<'a>> {
    Ok(match v {
        Value::Null => LuaValue::Nil,
        &Value::Bool(x) => LuaValue::Boolean(x),
        Value::Number(x) => {
            if let Some(i) = x.as_i64() {
                LuaValue::Integer(i)
            } else if let Some(f) = x.as_f64() {
                LuaValue::Number(f)
            } else {
                return Err(LuaError::external(format!("Failed to convert {:?}", x)));
            }
        }
        Value::String(x) => LuaValue::String(lua.create_string(x)?),
        Value::Array(xs) => {
            let vs = xs
                .iter()
                .map(|x| convert_value(lua, x))
                .collect::<LuaResult<Vec<_>>>()?;
            LuaValue::Table(lua.create_sequence_from(vs)?)
        }
        Value::Object(x) => convert_map(lua, x)?
    })
}

fn convert_map<'a>(lua: &'a Lua, m: &Map<String, Value>) -> LuaResult<LuaValue<'a>> {
    let kvs = m
        .iter()
        .map(|(k, v)| Ok((lua.create_string(k)?, convert_value(lua, v)?)))
        .collect::<LuaResult<Vec<_>>>()?;
    Ok(LuaValue::Table(lua.create_table_from(kvs)?))
}
