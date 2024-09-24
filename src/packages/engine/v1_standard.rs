use std::collections::HashMap;
use std::time::{UNIX_EPOCH, Duration};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};

use mlua::prelude::*;
use bufreaderwriter::rand::BufReaderWriterRand;

use super::EngineError;

const READ_CHUNK_LEN: usize = 8192;

pub struct Standard<'lua> {
    lua: &'lua Lua,

    _file_handles: Arc<Mutex<HashMap<u64, BufReaderWriterRand<File>>>>,

    fs_exists: LuaFunction<'lua>,
    fs_metadata: LuaFunction<'lua>,
    fs_open: LuaFunction<'lua>,
    fs_seek: LuaFunction<'lua>,
    fs_read: LuaFunction<'lua>,
    fs_write: LuaFunction<'lua>,
    fs_flush: LuaFunction<'lua>,
    fs_close: LuaFunction<'lua>,

    fs_read_file: LuaFunction<'lua>,
    fs_write_file: LuaFunction<'lua>,
    fs_remove_file: LuaFunction<'lua>
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

            fs_open: {
                let file_handles = file_handles.clone();

                lua.create_function(move |_, (path, options): (String, Option<LuaTable>)| {
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
                        .map_err(|err| LuaError::external(format!("failed to register handle: {err}")))?;
    
                    handles.insert(handle, BufReaderWriterRand::new_reader(file));
    
                    Ok(handle)
                })?
            },

            fs_seek: {
                let file_handles = file_handles.clone();

                lua.create_function(move |_, (handle, position): (u64, i64)| {
                    let mut handles = file_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(file) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid file handle"));
                    };

                    // Seek the file.
                    if position >= 0 {
                        file.seek(SeekFrom::Start(position as u64))?;
                    }

                    else {
                        file.seek(SeekFrom::End(position))?;
                    }

                    Ok(())
                })?
            },

            fs_read: {
                let file_handles = file_handles.clone();

                lua.create_function(move |_, (handle, position, length): (u64, Option<i64>, Option<u64>)| {
                    let mut handles = file_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(file) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid file handle"));
                    };

                    // Seek the file if position is given.
                    if let Some(position) = position {
                        if position >= 0 {
                            file.seek(SeekFrom::Start(position as u64))?;
                        }

                        else {
                            file.seek(SeekFrom::End(position))?;
                        }
                    }

                    // Read exact amount of bytes.
                    if let Some(length) = length {
                        let mut buf = vec![0; length as usize];

                        file.read_exact(&mut buf)?;

                        Ok(buf)
                    }

                    // Or just read a chunk of data.
                    else {
                        let mut buf = [0; READ_CHUNK_LEN];

                        let len = file.read(&mut buf)?;

                        Ok(buf[..len].to_vec())
                    }
                })?
            },

            fs_write: {
                let file_handles = file_handles.clone();

                lua.create_function(move |_, (handle, content, position): (u64, Vec<u8>, Option<i64>)| {
                    let mut handles = file_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(file) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid file handle"));
                    };

                    // Seek the file if position is given.
                    if let Some(position) = position {
                        if position >= 0 {
                            file.seek(SeekFrom::Start(position as u64))?;
                        }

                        else {
                            file.seek(SeekFrom::End(position))?;
                        }
                    }

                    // Write the content to the file.
                    file.write_all(&content)?;

                    Ok(())
                })?
            },

            fs_flush: {
                let file_handles = file_handles.clone();

                lua.create_function(move |_, handle: u64| {
                    let mut handles = file_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    // Flush the file if the handle is valid.
                    if let Some(file) = handles.get_mut(&handle) {
                        file.flush()?;
                    }

                    Ok(())
                })?
            },

            fs_close: {
                let file_handles = file_handles.clone();

                lua.create_function(move |_, handle: u64| {
                    let mut handles = file_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    // Flush the file if the handle is valid.
                    if let Some(file) = handles.get_mut(&handle) {
                        file.flush()?;
                    }

                    // Remove the file handle.
                    handles.remove(&handle);

                    Ok(())
                })?
            },

            fs_read_file: lua.create_function(|_, path: String| {
                let path = PathBuf::from(path);

                Ok(std::fs::read(path)?)
            })?,

            fs_write_file: lua.create_function(|_, (path, content): (String, Vec<u8>)| {
                let path = PathBuf::from(path);

                std::fs::write(path, &content)?;

                Ok(())
            })?,

            fs_remove_file: lua.create_function(|_, path: String| {
                let path = PathBuf::from(path);

                std::fs::remove_file(path)?;

                Ok(())
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
        fs.set("seek", self.fs_seek.clone())?;
        fs.set("read", self.fs_read.clone())?;
        fs.set("write", self.fs_write.clone())?;
        fs.set("flush", self.fs_flush.clone())?;
        fs.set("close", self.fs_close.clone())?;

        fs.set("read_file", self.fs_read_file.clone())?;
        fs.set("write_file", self.fs_write_file.clone())?;
        fs.set("remove_file", self.fs_remove_file.clone())?;

        Ok(env)
    }
}
