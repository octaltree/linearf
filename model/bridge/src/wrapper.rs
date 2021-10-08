use mlua::prelude::LuaUserData;

#[derive(Clone)]
pub struct Wrapper<T>(T);

impl<T> LuaUserData for Wrapper<T> {}

impl<T> std::ops::Deref for Wrapper<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<T> Wrapper<T> {
    pub fn new(inner: T) -> Self { Self(inner) }
}
