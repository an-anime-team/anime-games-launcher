use mlua::prelude::*;

use crate::packages::prelude::*;

#[derive(Debug)]
pub struct Engine {
    store: PackagesStore,
    lock_file: LockFile,
    lua: Lua
}

impl Engine {
    pub fn create(store: PackagesStore, lock_file: LockFile) -> Self {
        Self {
            store,
            lock_file,
            lua: Lua::new()
        }
    }
}
