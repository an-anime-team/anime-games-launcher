use mlua::prelude::*;

use crate::packages::prelude::*;

#[derive(Debug)]
pub struct Engine {
    store: PackagesStore,
    lock_file: LockFileManifest,
    lua: Lua
}

impl Engine {
    pub fn create(store: PackagesStore, lock_file: LockFileManifest) -> Self {
        let engine = Lua::new();

        // for package in lock_file.

        // engine.globals().set("v1_packages_load", engine.create_function(|_, name| {

        // }));

        Self {
            store,
            lock_file,
            lua: Lua::new()
        }
    }
}
