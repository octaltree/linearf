use mlua::prelude::*;
use std::process::Command;

#[mlua::lua_module]
fn bridge(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    // exports.set("sum", lua.create_function(sum)?)?;
    // exports.set("used_memory", lua.create_function(used_memory)?)?;
    // exports.set("check_userdata", lua.create_function(check_userdata)?)?;
    exports.set("spawn", lua.create_function(spawn)?)?;
    Ok(exports)
}

fn sum(_: &Lua, (a, b): (i64, i64)) -> LuaResult<i64> { Ok(a + b) }

fn used_memory(lua: &Lua, _: ()) -> LuaResult<usize> { Ok(lua.used_memory()) }

fn check_userdata(_: &Lua, ud: MyUserData) -> LuaResult<i32> { Ok(ud.0) }

fn spawn(_: &Lua, _: ()) -> LuaResult<()> {
    let mut rt = tokio::runtime::Runtime::new()?;
    Command::new("touch").arg("/tmp/bar").status().unwrap();
    rt.spawn(async {
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        Command::new("touch").arg("/tmp/foo").status().unwrap();
    });
    Ok(())
}

#[derive(Clone, Copy)]
struct MyUserData(i32);

impl LuaUserData for MyUserData {}

//#[mlua::lua_module]
// fn rust_module_second(lua: &Lua) -> LuaResult<LuaTable> {
//    let exports = lua.create_table()?;
//    exports.set("userdata", lua.create_userdata(MyUserData(123))?)?;
//    Ok(exports)
//}

#[mlua::lua_module]
fn rust_module_error(_: &Lua) -> LuaResult<LuaTable> { Err("custom module error".to_lua_err()) }
