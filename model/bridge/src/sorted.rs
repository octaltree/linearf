use crate::{lnf::Lnf, wrapper::Wrapper, LINEARF};
use linearf::{
    item::{Item, MaybeUtf8},
    state,
    state::WithScore,
    Linearf, State
};
use mlua::{chunk, prelude::*};
use serde::Serialize;
use serde_json::{Map, Value};
use std::sync::Arc;
use tokio::sync::RwLockReadGuard;

pub fn lock_sorted<'a>(
    lua: &'a Lua,
    (s, f, handler): (i32, usize, LuaFunction<'a>)
) -> LuaResult<LuaValue<'a>> {
    let s = state::SessionId(s);
    let f = state::FlowId(f);
    let lnf: Wrapper<Arc<Lnf>> = lua.named_registry_value(LINEARF)?;
    let st = Arc::clone(lnf.state());
    lnf.runtime().block_on(async {
        let state: RwLockReadGuard<'a, State> = st.read().await;
        let flow = match state.get_flow(s, f) {
            Some(flow) => flow,
            None => {
                let msg = format!("flow {:?} {:?} not found", s, f);
                return Err(LuaError::external(msg));
            }
        };
        lua.load(chunk! {}).eval()
        // handler.call::<_, LuaValue<'a>>((LockSorted(flow.sorted().await),))
    })
}

pub struct LockSorted<'a>(pub(crate) RwLockReadGuard<'a, (bool, Vec<WithScore>)>);

impl<'a> LuaUserData for LockSorted<'a> {}

pub fn sorted_count(lua: &Lua, lock: LockSorted) -> LuaResult<usize> { Ok(lock.0 .1.len()) }

pub fn sorted_done(lua: &Lua, lock: LockSorted) -> LuaResult<bool> { Ok(lock.0 .0) }

/// Panic: if end > len
pub fn sorted_items<'a>(
    lua: &'a Lua,
    (lock, start, end): (LockSorted, usize, usize)
) -> LuaResult<LuaValue<'a>> {
    let len = lock.0 .1.len();
    convert_items(lua, &lock.0 .1[start..end])
}

pub fn sorted_item<'a>(
    lua: &'a Lua,
    (lock, id): (LockSorted, u32)
) -> LuaResult<Option<LuaValue<'a>>> {
    let i = match lock.0 .1.iter().find(|(i, _)| i.id == id) {
        Some((i, _)) => i,
        None => return Ok(None)
    };
    let i = convert_item(lua, i)?;
    lua.to_value_with(
        &i,
        mlua::SerializeOptions::new()
            .serialize_none_to_null(false)
            .serialize_unit_to_null(false)
    )
    .map(Some)
}

fn convert_items<'a>(lua: &'a Lua, items: &[WithScore]) -> LuaResult<LuaValue<'a>> {
    let xs = items
        .iter()
        .map(|(i, _)| convert_item(lua, &*i))
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

#[derive(Serialize)]
struct I<'a> {
    id: u32,
    r#type: &'a str,
    value: LuaString<'a>,
    info: LuaValue<'a>,
    view: std::borrow::Cow<'a, str>
}

fn convert_item<'a>(lua: &'a Lua, i: &'a Item) -> LuaResult<I<'a>> {
    Ok(I {
        id: i.id,
        r#type: i.r#type,
        value: maybe_utf8_into_lua_string(lua, &i.value)?,
        info: convert_info(lua, &i.info)?,
        view: i.view()
    })
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
