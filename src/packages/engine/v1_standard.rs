use std::collections::HashMap;
use std::time::{UNIX_EPOCH, Duration};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::fs::File;

use mlua::prelude::*;

use super::EngineError;

#[derive(Debug)]
pub struct Standard<'lua> {
    lua: &'lua Lua,

    _file_handles: Arc<Mutex<HashMap<u64, File>>>,

    fs_exists: LuaFunction<'lua>,
    fs_metadata: LuaFunction<'lua>,
    fs_open: LuaFunction<'lua>
}

impl<'lua> Standard<'lua> {
    pub fn new(lua: &'lua Lua) -> Result<Self, EngineError> {
        let file_handles = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            lua,

            _file_handles: file_handles.clone(),

            fs_exists: lua.create_function(|_, path: String| {
                let path = PathBuf::from(path);

                Ok(path.exists())
            })?,

            fs_metadata: lua.create_function(|lua, path: String| {
                let path = PathBuf::from(path);

                let metadata = path.metadata()?;

                let result = lua.create_table()?;

                result.set("created_at", {
                    metadata.created()?
                        .duration_since(UNIX_EPOCH)
                        .as_ref()
                        .map(Duration::as_secs)
                        .unwrap_or_default()
                })?;

                result.set("modified_at", {
                    metadata.modified()?
                        .duration_since(UNIX_EPOCH)
                        .as_ref()
                        .map(Duration::as_secs)
                        .unwrap_or_default()
                })?;

                result.set("length", metadata.len())?;
                result.set("is_accessible", true)?;

                Ok(result)
            })?,

            fs_open: lua.create_function(move |_, (path, options): (String, Option<LuaTable>)| {
                let path = PathBuf::from(path);
                let handle = rand::random::<u64>();

                let mut read = true;
                let mut write = false;
                let mut create = false;
                let mut overwrite = false;
                let mut append = false;

                if let Some(options) = options {
                    read      = options.get::<_, bool>("read").unwrap_or(true);
                    write     = options.get::<_, bool>("write").unwrap_or_default();
                    create    = options.get::<_, bool>("create").unwrap_or_default();
                    overwrite = options.get::<_, bool>("overwrite").unwrap_or_default();
                    append    = options.get::<_, bool>("append").unwrap_or_default();
                }

                let file = File::options()
                    .read(read)
                    .write(write)
                    .create(create)
                    .truncate(overwrite)
                    .append(append)
                    .open(path)?;

                let mut handles = file_handles.lock()
                    .map_err(|err| LuaError::external(format!("Failed to create file handle: {err}")))?;

                handles.insert(handle, file);

                Ok(handle)
            })?
        })
    }

    /// Create new environment for the v1 modules standard.
    pub fn create_env(&self) -> Result<LuaTable<'lua>, EngineError> {
        let env = self.lua.create_table()?;
        let fs = self.lua.create_table()?;

        env.set("fs", fs.clone())?;

        fs.set("exists", self.fs_exists.clone())?;
        fs.set("metadata", self.fs_metadata.clone())?;
        fs.set("open", self.fs_open.clone())?;

        Ok(env)
    }
}
