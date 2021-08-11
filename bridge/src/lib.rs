use mlua::prelude::*;
use std::process::Command;

#[mlua::lua_module]
fn bridge(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("spawn", lua.create_function(spawn)?)?;
    Ok(exports)
}

fn spawn(_: &Lua, _: ()) -> LuaResult<()> {
    let mut rt = tokio::runtime::Runtime::new()?;
    Command::new("touch").arg("/tmp/bar").status().unwrap();
    // TODO
    rt.spawn(async {
        Command::new("touch").arg("/tmp/foo").status().unwrap();
    });
    Ok(())
}

#[derive(Clone, Copy)]
struct MyUserData(i32);

impl LuaUserData for MyUserData {}
