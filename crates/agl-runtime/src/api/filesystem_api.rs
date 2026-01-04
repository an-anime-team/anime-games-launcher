// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-runtime
// Copyright (C) 2025 - 2026  Nikita Podvirnyi <krypt0nn@vk.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use std::time::{Duration, UNIX_EPOCH};

use mlua::prelude::*;

use bufreaderwriter::rand::BufReaderWriterRand;

use agl_core::tasks::fs;

use super::bytes::Bytes;
use super::task_api::{Promise, PromiseValue, TaskOutput, task_output};
use super::*;

pub const IO_READ_CHUNK_LEN: usize = 4096; // 4 KiB file reads
pub const IO_BUFFER_SIZE: usize = 4 * 1024 * 1024 * 1024; // 4 MiB read/write in-RAM cache

pub struct FilesystemApi {
    lua: Lua,

    fs_exists: LuaFunctionBuilder,
    fs_metadata: LuaFunctionBuilder,
    fs_copy: LuaFunctionBuilder,
    fs_move: LuaFunctionBuilder,
    fs_remove: LuaFunctionBuilder,
    fs_open: LuaFunctionBuilder,
    fs_seek: LuaFunction,
    fs_seek_rel: LuaFunction,
    fs_truncate: LuaFunction,
    fs_read: LuaFunction,
    fs_write: LuaFunction,
    fs_flush: LuaFunction,
    fs_close: LuaFunction,

    fs_create_file: LuaFunctionBuilder,
    fs_read_file: LuaFunctionBuilder,
    fs_write_file: LuaFunctionBuilder,
    fs_remove_file: LuaFunctionBuilder,
    fs_create_dir: LuaFunctionBuilder,
    fs_read_dir: LuaFunctionBuilder,
    fs_remove_dir: LuaFunctionBuilder
}

impl FilesystemApi {
    pub fn new(lua: Lua) -> Result<Self, LuaError> {
        let file_handles = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            fs_exists: Box::new(|lua: &Lua, context: &Context| {
                let context = context.to_owned();

                lua.create_function(move |_, mut path: PathBuf| -> Result<bool, LuaError> {
                    if path.is_relative() {
                        path = context.module_folder.join(path);
                    }

                    path = normalize_path(path, true)
                        .map_err(|err| {
                            LuaError::external(format!("failed to normalize path: {err}"))
                        })?;

                    if !path.exists() {
                        return Ok(false);
                    }

                    context.can_read_path(&path)
                        .map_err(LuaError::external)
                })
            }),

            fs_metadata: Box::new(|lua: &Lua, context: &Context| {
                let context = context.to_owned();

                lua.create_function(move |lua, mut path: PathBuf| {
                    if path.is_relative() {
                        path = context.module_folder.join(path);
                    }

                    path = normalize_path(path, false)
                        .map_err(|err| {
                            LuaError::external(format!("failed to normalize path: {err}"))
                        })?;

                    let metadata = path.metadata()?;

                    let result = lua.create_table_with_capacity(0, 5)?;

                    result.raw_set("created_at", {
                        metadata.created()?
                            .duration_since(UNIX_EPOCH)
                            .as_ref()
                            .map(Duration::as_secs)
                            .unwrap_or_default()
                    })?;

                    result.raw_set("modified_at", {
                        metadata.modified()?
                            .duration_since(UNIX_EPOCH)
                            .as_ref()
                            .map(Duration::as_secs)
                            .unwrap_or_default()
                    })?;

                    result.raw_set("length", metadata.len())?;

                    let permissions = lua.create_table_with_capacity(0, 2)?;

                    permissions.raw_set("read", context.can_read_path(&path)?)?;
                    permissions.raw_set("write", context.can_write_path(&path)?)?;

                    result.raw_set("permissions", permissions)?;

                    result.raw_set("type", {
                        if metadata.is_symlink() {
                            "symlink"
                        } else if metadata.is_dir() {
                            "directory"
                        } else {
                            "file"
                        }
                    })?;

                    Ok(result)
                })
            }),

            fs_copy: Box::new(|lua: &Lua, context: &Context| {
                let context = context.to_owned();

                lua.create_function(move |lua: &Lua, (mut source, mut target): (PathBuf, PathBuf)| {
                    if source.is_relative() {
                        source = context.module_folder.join(source);
                    }

                    if target.is_relative() {
                        target = context.module_folder.join(target);
                    }

                    source = normalize_path(source, true)
                        .map_err(|err| {
                            LuaError::external(format!("failed to normalize source path: {err}"))
                        })?;

                    target = normalize_path(target, true)
                        .map_err(|err| {
                            LuaError::external(format!("failed to normalize target path: {err}"))
                        })?;

                    // Throw an error if source path doesn't exists or inaccessible.
                    if !source.exists() {
                        return Err(LuaError::external("source path doesn't exists"));
                    }

                    if !context.can_read_path(&source)? {
                        return Err(LuaError::external("no source path read permissions"));
                    }

                    // Throw an error if target path already exists or inaccessible.
                    if target.exists() {
                        return Err(LuaError::external("target path already exists"));
                    }

                    if !context.can_write_path(&target)? {
                        return Err(LuaError::external("no target path write permissions"));
                    }

                    async fn try_copy(source: &Path, target: &Path) -> std::io::Result<()> {
                        if source.is_file() {
                            fs::copy(source, target).await?;
                        }

                        else if source.is_dir() {
                            fs::create_dir_all(target).await?;

                            for entry in source.read_dir()? {
                                let entry = entry?;

                                Box::pin(try_copy(
                                    &entry.path(),
                                    &target.join(entry.file_name())
                                )).await?;
                            }
                        }

                        else if source.is_symlink() {
                            // FIXME: only works on unix systems while we target
                            //        to support all the OSes.

                            #[allow(clippy::collapsible_if)]
                            if let Some(source_filename) = source.file_name() {
                                std::os::unix::fs::symlink(
                                    source.read_link()?,
                                    target.join(source_filename)
                                )?;
                            }
                        }

                        Ok(())
                    }

                    let value = PromiseValue::from_future(async move {
                        try_copy(&source, &target).await?;

                        Ok(task_output(Ok(LuaValue::Nil)))
                    });

                    Promise::new(value)
                        .into_lua(lua)
                })
            }),

            fs_move: Box::new(|lua: &Lua, context: &Context| {
                let context = context.to_owned();

                lua.create_function(move |lua: &Lua, (mut source, mut target): (PathBuf, PathBuf)| {
                    if source.is_relative() {
                        source = context.module_folder.join(source);
                    }

                    if target.is_relative() {
                        target = context.module_folder.join(target);
                    }

                    source = normalize_path(source, false)
                        .map_err(|err| {
                            LuaError::external(format!("failed to normalize source path: {err}"))
                        })?;

                    target = normalize_path(target, true)
                        .map_err(|err| {
                            LuaError::external(format!("failed to normalize target path: {err}"))
                        })?;

                    // Throw an error if source path doesn't exists or inaccessible.
                    if !source.exists() {
                        return Err(LuaError::external("source path doesn't exists"));
                    }

                    if !context.can_write_path(&source)? {
                        return Err(LuaError::external("no source path write permissions"));
                    }

                    // Throw an error if target path already exists or inaccessible.
                    if target.exists() {
                        return Err(LuaError::external("target path already exists"));
                    }

                    if !context.can_write_path(&target)? {
                        return Err(LuaError::external("no target path write permissions"));
                    }

                    async fn try_move(source: &Path, target: &Path) -> std::io::Result<()> {
                        if source.is_file() {
                            // Try to rename the file (mv) or copy it and then
                            // remove the source if mv has failed (different
                            // mounts).
                            if fs::rename(source, target).await.is_err() {
                                fs::copy(source, target).await?;
                                fs::remove_file(source).await?;
                            }
                        }

                        else if source.is_dir() {
                            // Try to rename the folder (mv) or create a target
                            // folder and move all the files there.
                            if fs::rename(source, target).await.is_err() {
                                fs::create_dir_all(target).await?;

                                for entry in source.read_dir()? {
                                    let entry = entry?;

                                    Box::pin(try_move(
                                        &entry.path(),
                                        &target.join(entry.file_name())
                                    )).await?;
                                }

                                fs::remove_dir_all(source).await?;
                            }
                        }

                        else if source.is_symlink() {
                            if let Some(source_filename) = source.file_name() {
                                std::os::unix::fs::symlink(
                                    source.read_link()?,
                                    target.join(source_filename)
                                )?;
                            }

                            fs::remove_file(source).await?;
                        }

                        Ok(())
                    }

                    let value = PromiseValue::from_future(async move {
                        try_move(&source, &target).await?;

                        Ok(task_output(Ok(LuaValue::Nil)))
                    });

                    Promise::new(value)
                        .into_lua(lua)
                })
            }),

            fs_remove: Box::new(|lua: &Lua, context: &Context| {
                let context = context.to_owned();

                lua.create_function(move |lua: &Lua, mut path: PathBuf| {
                    if path.is_relative() {
                        path = context.module_folder.join(path);
                    }

                    path = normalize_path(path, false)
                        .map_err(|err| {
                            LuaError::external(format!("failed to normalize path: {err}"))
                        })?;

                    if !context.can_write_path(&path)? {
                        return Err(LuaError::external("no path write permissions"));
                    }

                    let value = PromiseValue::from_future(async move {
                        if path.is_dir() {
                            fs::remove_dir_all(path).await?;
                        } else {
                            fs::remove_file(path).await?;
                        }

                        Ok(task_output(Ok(LuaValue::Nil)))
                    });

                    Promise::new(value)
                        .into_lua(lua)
                })
            }),

            fs_open: {
                let file_handles = file_handles.clone();

                Box::new(move |lua: &Lua, context: &Context| {
                    let context = context.to_owned();
                    let file_handles = file_handles.clone();

                    lua.create_function(move |_, (mut path, options): (PathBuf, Option<LuaTable>)| {
                        if path.is_relative() {
                            path = context.module_folder.join(path);
                        }

                        path = normalize_path(path, true)
                            .map_err(|err| {
                                LuaError::external(format!("failed to normalize path: {err}"))
                            })?;

                        if let Some(parent) = path.parent() && !parent.is_dir() {
                            if !context.can_write_path(parent)? {
                                return Err(LuaError::external("no path write permissions"));
                            }

                            std::fs::create_dir_all(parent)?;
                        }

                        let mut read = true;
                        let mut write = false;
                        let mut create = false;
                        let mut overwrite = false;
                        let mut append = false;

                        if let Some(options) = options {
                            read      = options.get::<bool>("read").unwrap_or(true);
                            write     = options.get::<bool>("write").unwrap_or_default();
                            create    = options.get::<bool>("create").unwrap_or_default();
                            overwrite = options.get::<bool>("overwrite").unwrap_or_default();
                            append    = options.get::<bool>("append").unwrap_or_default();
                        }

                        if read && !context.can_read_path(&path)? {
                            return Err(LuaError::external("no path read permissions"));
                        }

                        if (write || create || overwrite || append) && !context.can_write_path(&path)? {
                            return Err(LuaError::external("no path write permissions"));
                        }

                        let file = File::options()
                            .read(read)
                            .write(write)
                            .create(create)
                            .truncate(overwrite)
                            .append(append)
                            .open(path)?;

                        let mut handles = file_handles.lock()
                            .map_err(|err| {
                                LuaError::external(format!("failed to register handle: {err}"))
                            })?;

                        let mut handle = rand::random::<i32>();

                        while handles.contains_key(&handle) {
                            handle = rand::random::<i32>();
                        }

                        let buf = BufReaderWriterRand::reader_with_capacity(IO_BUFFER_SIZE, file);

                        handles.insert(handle, buf);

                        Ok(handle)
                    })
                })
            },

            fs_seek: {
                let file_handles = file_handles.clone();

                lua.create_function(move |_, (handle, position): (i32, i64)| {
                    let mut handles = file_handles.lock()
                        .map_err(|err| {
                            LuaError::external(format!("failed to read handle: {err}"))
                        })?;

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

            fs_seek_rel: {
                let file_handles = file_handles.clone();

                lua.create_function(move |_, (handle, offset): (i32, i64)| {
                    let mut handles = file_handles.lock()
                        .map_err(|err| {
                            LuaError::external(format!("failed to read handle: {err}"))
                        })?;

                    let Some(file) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid file handle"));
                    };

                    file.seek(SeekFrom::Current(offset))?;

                    Ok(())
                })?
            },

            fs_truncate: {
                let file_handles = file_handles.clone();

                lua.create_function(move |_, (handle, length): (i32, u64)| {
                    let mut handles = file_handles.lock()
                        .map_err(|err| {
                            LuaError::external(format!("failed to read handle: {err}"))
                        })?;

                    let Some(file) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid file handle"));
                    };

                    file.get_mut().set_len(length)?;

                    Ok(())
                })?
            },

            fs_read: {
                let file_handles = file_handles.clone();

                lua.create_function(move |lua: &Lua, (handle, position, length): (i32, Option<i64>, Option<u64>)| {
                    let mut handles = file_handles.lock()
                        .map_err(|err| {
                            LuaError::external(format!("failed to read handle: {err}"))
                        })?;

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

                        Bytes::from(buf)
                            .into_lua(lua)
                    }

                    // Or just read a chunk of data.
                    else {
                        let mut buf = [0; IO_READ_CHUNK_LEN];

                        let len = file.read(&mut buf)?;

                        if len == 0 {
                            return Ok(LuaValue::Nil);
                        }

                        Bytes::from(buf[..len].to_vec())
                            .into_lua(lua)
                    }
                })?
            },

            fs_write: {
                let file_handles = file_handles.clone();

                lua.create_function(move |_, (handle, content, position): (i32, Bytes, Option<i64>)| {
                    let mut handles = file_handles.lock()
                        .map_err(|err| {
                            LuaError::external(format!("failed to read handle: {err}"))
                        })?;

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
                    file.write_all(content.as_slice())?;

                    Ok(())
                })?
            },

            fs_flush: {
                let file_handles = file_handles.clone();

                lua.create_function(move |_, handle: i32| {
                    let mut handles = file_handles.lock()
                        .map_err(|err| {
                            LuaError::external(format!("failed to read handle: {err}"))
                        })?;

                    // Flush the file if the handle is valid.
                    if let Some(file) = handles.get_mut(&handle) {
                        file.flush()?;
                    }

                    Ok(())
                })?
            },

            fs_close: {
                let file_handles = file_handles.clone();

                lua.create_function(move |_, handle: i32| {
                    let mut handles = file_handles.lock()
                        .map_err(|err| {
                            LuaError::external(format!("failed to read handle: {err}"))
                        })?;

                    // Flush the file if the handle is valid.
                    if let Some(file) = handles.get_mut(&handle) {
                        file.flush()?;
                    }

                    // Remove the file handle.
                    handles.remove(&handle);

                    Ok(())
                })?
            },

            fs_create_file: Box::new(|lua: &Lua, context: &Context| {
                let context = context.to_owned();

                lua.create_function(move |lua: &Lua, mut path: PathBuf| {
                    if path.is_relative() {
                        path = context.module_folder.join(path);
                    }

                    path = normalize_path(path, true)
                        .map_err(|err| {
                            LuaError::external(format!("failed to normalize path: {err}"))
                        })?;

                    if !context.can_write_path(&path)? {
                        return Err(LuaError::external("no path write permissions"));
                    }

                    let value = PromiseValue::from_future(async move {
                        if let Some(parent) = path.parent() && !parent.is_dir() {
                            fs::create_dir_all(parent).await?;
                        }

                        fs::write(path, []).await?;

                        Ok(task_output(Ok(LuaValue::Nil)))
                    });

                    Promise::new(value)
                        .into_lua(lua)
                })
            }),

            fs_read_file: Box::new(|lua: &Lua, context: &Context| {
                let context = context.to_owned();

                lua.create_function(move |lua: &Lua, mut path: PathBuf| {
                    if path.is_relative() {
                        path = context.module_folder.join(path);
                    }

                    path = normalize_path(path, true)
                        .map_err(|err| {
                            LuaError::external(format!("failed to normalize path: {err}"))
                        })?;

                    if !context.can_read_path(&path)? {
                        return Err(LuaError::external("no path read permissions"));
                    }

                    let value = PromiseValue::from_future(async move {
                        let content = Bytes::from(fs::read(path).await?);

                        Ok(Box::new(move |lua: &Lua| {
                            content.into_lua(lua)
                        }) as TaskOutput)
                    });

                    Promise::new(value)
                        .into_lua(lua)
                })
            }),

            fs_write_file: Box::new(|lua: &Lua, context: &Context| {
                let context = context.to_owned();

                lua.create_function(move |lua: &Lua, (mut path, content): (PathBuf, Bytes)| {
                    if path.is_relative() {
                        path = context.module_folder.join(path);
                    }

                    path = normalize_path(path, true)
                        .map_err(|err| {
                            LuaError::external(format!("failed to normalize path: {err}"))
                        })?;

                    if !context.can_write_path(&path)? {
                        return Err(LuaError::external("no path write permissions"));
                    }

                    let value = PromiseValue::from_future(async move {
                        if let Some(parent) = path.parent() && !parent.is_dir() {
                            fs::create_dir_all(parent).await?;
                        }

                        fs::write(path, content.as_slice()).await?;

                        Ok(task_output(Ok(LuaValue::Nil)))
                    });

                    Promise::new(value)
                        .into_lua(lua)
                })
            }),

            fs_remove_file: Box::new(|lua: &Lua, context: &Context| {
                let context = context.to_owned();

                lua.create_function(move |lua: &Lua, mut path: PathBuf| {
                    if path.is_relative() {
                        path = context.module_folder.join(path);
                    }

                    path = normalize_path(path, false)
                        .map_err(|err| {
                            LuaError::external(format!("failed to normalize path: {err}"))
                        })?;

                    if !context.can_write_path(&path)? {
                        return Err(LuaError::external("no path write permissions"));
                    }

                    let value = PromiseValue::from_future(async move {
                        fs::remove_file(path).await?;

                        Ok(task_output(Ok(LuaValue::Nil)))
                    });

                    Promise::new(value)
                        .into_lua(lua)
                })
            }),

            fs_create_dir: Box::new(|lua: &Lua, context: &Context| {
                let context = context.to_owned();

                lua.create_function(move |lua: &Lua, mut path: PathBuf| {
                    if path.is_relative() {
                        path = context.module_folder.join(path);
                    }

                    path = normalize_path(path, true)
                        .map_err(|err| {
                            LuaError::external(format!("failed to normalize path: {err}"))
                        })?;

                    if !context.can_write_path(&path)? {
                        return Err(LuaError::external("no path write permissions"));
                    }

                    let value = PromiseValue::from_future(async move {
                        fs::create_dir_all(path).await?;

                        Ok(task_output(Ok(LuaValue::Nil)))
                    });

                    Promise::new(value)
                        .into_lua(lua)
                })
            }),

            fs_read_dir: Box::new(|lua: &Lua, context: &Context| {
                let context = context.to_owned();

                lua.create_function(move |lua: &Lua, mut path: PathBuf| {
                    if path.is_relative() {
                        path = context.module_folder.join(path);
                    }

                    path = normalize_path(path, true)
                        .map_err(|err| {
                            LuaError::external(format!("failed to normalize path: {err}"))
                        })?;

                    if !context.can_read_path(&path)? {
                        return Err(LuaError::external("no path read permissions"));
                    }

                    let value = PromiseValue::from_blocking(move || {
                        let entries = path.read_dir()?
                            .map(|entry| {
                                entry.map(|entry| (entry.file_name(), entry.path()))
                            })
                            .collect::<Result<Box<[_]>, _>>()?;

                        Ok(Box::new(move |lua: &Lua| {
                            let result = lua.create_table_with_capacity(entries.len(), 0)?;

                            for (name, path) in entries {
                                let entry_table = lua.create_table_with_capacity(0, 3)?;

                                entry_table.raw_set("name", name.to_string_lossy().to_string())?;
                                entry_table.raw_set("path", path.to_string_lossy().to_string())?;

                                entry_table.raw_set("type", {
                                    if path.is_symlink() {
                                        "symlink"
                                    } else if path.is_dir() {
                                        "directory"
                                    } else {
                                        "file"
                                    }
                                })?;

                                result.raw_push(entry_table)?;
                            }

                            Ok(LuaValue::Table(result))
                        }) as TaskOutput)
                    });

                    Promise::new(value)
                        .into_lua(lua)
                })
            }),

            fs_remove_dir: Box::new(|lua: &Lua, context: &Context| {
                let context = context.to_owned();

                lua.create_function(move |lua: &Lua, mut path: PathBuf| {
                    if path.is_relative() {
                        path = context.module_folder.join(path);
                    }

                    path = normalize_path(path, true)
                        .map_err(|err| {
                            LuaError::external(format!("failed to normalize path: {err}"))
                        })?;

                    if !context.can_write_path(&path)? {
                        return Err(LuaError::external("no path write permissions"));
                    }

                    let value = PromiseValue::from_future(async move {
                        fs::remove_dir_all(path).await?;

                        Ok(task_output(Ok(LuaValue::Nil)))
                    });

                    Promise::new(value)
                        .into_lua(lua)
                })
            }),

            lua
        })
    }

    /// Create new lua table with API functions.
    pub fn create_env(&self, context: &Context) -> Result<LuaTable, LuaError> {
        let env = self.lua.create_table_with_capacity(0, 20)?;

        env.raw_set("exists", (self.fs_exists)(&self.lua, context)?)?;
        env.raw_set("metadata", (self.fs_metadata)(&self.lua, context)?)?;
        env.raw_set("copy", (self.fs_copy)(&self.lua, context)?)?;
        env.raw_set("move", (self.fs_move)(&self.lua, context)?)?;
        env.raw_set("remove", (self.fs_remove)(&self.lua, context)?)?;
        env.raw_set("open", (self.fs_open)(&self.lua, context)?)?;
        env.raw_set("seek", &self.fs_seek)?;
        env.raw_set("seek_rel", &self.fs_seek_rel)?;
        env.raw_set("truncate", &self.fs_truncate)?;
        env.raw_set("read", &self.fs_read)?;
        env.raw_set("write", &self.fs_write)?;
        env.raw_set("flush", &self.fs_flush)?;
        env.raw_set("close", &self.fs_close)?;

        env.raw_set("create_file", (self.fs_create_file)(&self.lua, context)?)?;
        env.raw_set("read_file", (self.fs_read_file)(&self.lua, context)?)?;
        env.raw_set("write_file", (self.fs_write_file)(&self.lua, context)?)?;
        env.raw_set("remove_file", (self.fs_remove_file)(&self.lua, context)?)?;
        env.raw_set("create_dir", (self.fs_create_dir)(&self.lua, context)?)?;
        env.raw_set("read_dir", (self.fs_read_dir)(&self.lua, context)?)?;
        env.raw_set("remove_dir", (self.fs_remove_dir)(&self.lua, context)?)?;

        Ok(env)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn fs_file_handle() -> anyhow::Result<()> {
//         let path = std::env::temp_dir().join(".agl-v1-file-handle-test");

//         if path.exists() {
//             std::fs::remove_file(&path)?;
//         }

//         let path = path.to_string_lossy().to_string();

//         let lua = Lua::new();
//         let api = FilesystemAPI::new(lua.clone())?;

//         let env = api.create_env(&Context {
//             resource_hash: Hash::rand(),
//             temp_folder: std::env::temp_dir(),
//             module_folder: std::env::temp_dir(),
//             persistent_folder: std::env::temp_dir(),
//             input_resources: vec![],
//             ext_process_api: false,
//             ext_allowed_paths: vec![],
//             local_validator: LocalValidator::open(std::env::temp_dir().join("local_validator.json"))?
//         })?;

//         assert!(!env.call_function::<bool>("exists", path.clone())?);
//         assert!(env.call_function::<i32>("open", path.clone()).is_err());

//         let options = lua.create_table()?;

//         options.set("read", true)?;
//         options.set("write", true)?;
//         options.set("create", true)?;

//         let handle = env.call_function::<i32>("open", (path.clone(), options))?;

//         assert_eq!(api.fs_read.call::<Vec<u8>>(handle)?.len(), 0);

//         api.fs_write.call::<()>((handle, b"Hello, ".to_vec()))?;
//         api.fs_write.call::<()>((handle, b"World!".to_vec()))?;
//         api.fs_flush.call::<()>(handle)?;

//         api.fs_seek_rel.call::<()>((handle, -13))?;

//         assert_eq!(api.fs_read.call::<Vec<u8>>(handle)?, b"Hello, World!");

//         api.fs_seek.call::<()>((handle, 0))?;
//         api.fs_write.call::<()>((handle, b"Amogus".to_vec()))?;
//         api.fs_flush.call::<()>(handle)?;

//         api.fs_seek.call::<()>((handle, 0))?;

//         assert_eq!(api.fs_read.call::<Vec<u8>>(handle)?, b"Amogus World!");

//         api.fs_seek.call::<()>((handle, -6))?;
//         api.fs_write.call::<()>((handle, b"Amogus".to_vec()))?;
//         api.fs_flush.call::<()>(handle)?;

//         api.fs_seek.call::<()>((handle, 0))?;

//         assert_eq!(api.fs_read.call::<Vec<u8>>(handle)?, b"Amogus Amogus");

//         api.fs_seek.call::<()>((handle, 0))?;
//         api.fs_write.call::<()>((handle, b"Sugoma".to_vec()))?;

//         assert_eq!(api.fs_read.call::<Vec<u8>>(handle)?, b" Amogus");

//         api.fs_flush.call::<()>(handle)?;
//         api.fs_seek.call::<()>((handle, 0))?;

//         assert_eq!(api.fs_read.call::<Vec<u8>>(handle)?, b"Sugoma Amogus");
//         assert_eq!(api.fs_read.call::<Vec<u8>>((handle, 3, 7))?, b"oma Amo");
//         assert_eq!(api.fs_read.call::<Vec<u8>>(handle)?, b"gus");
//         assert_eq!(api.fs_read.call::<Vec<u8>>((handle, -6))?, b"Amogus");

//         api.fs_write.call::<()>((handle, b"Mogusa".to_vec(), 0))?;
//         api.fs_write.call::<()>((handle, b"Susoma".to_vec(), 7))?;

//         assert_eq!(api.fs_read.call::<Vec<u8>>((handle, 0))?, b"Mogusa Susoma");

//         api.fs_truncate.call::<()>((handle, 6))?;

//         assert_eq!(api.fs_read.call::<Vec<u8>>((handle, 0))?, b"Mogusa");

//         api.fs_truncate.call::<()>((handle, 10))?;

//         assert_eq!(api.fs_read.call::<Vec<u8>>((handle, 0))?, b"Mogusa\0\0\0\0");

//         api.fs_close.call::<()>(handle)?;

//         assert!(api.fs_read.call::<Vec<u8>>(handle).is_err());

//         Ok(())
//     }

//     #[test]
//     fn fs_file_actions() -> anyhow::Result<()> {
//         let path = std::env::temp_dir().join(".agl-v1-file-actions-test");

//         if path.exists() {
//             std::fs::remove_file(&path)?;
//         }

//         let path = path.to_string_lossy().to_string();

//         let api = FilesystemAPI::new(Lua::new())?;

//         let env = api.create_env(&Context {
//             resource_hash: Hash::rand(),
//             temp_folder: std::env::temp_dir(),
//             module_folder: std::env::temp_dir(),
//             persistent_folder: std::env::temp_dir(),
//             input_resources: vec![],
//             ext_process_api: false,
//             ext_allowed_paths: vec![],
//             local_validator: LocalValidator::open(std::env::temp_dir().join("local_validator.json"))?
//         })?;

//         assert!(!env.call_function::<bool>("exists", path.clone())?);

//         env.call_function::<()>("write_file", (path.clone(), vec![1, 2, 3]))?;

//         assert!(env.call_function::<bool>("exists", path.clone())?);

//         let metadata = env.call_function::<LuaTable>("metadata", path.clone())?;

//         assert_eq!(metadata.get::<u64>("length")?, 3);
//         assert_eq!(metadata.get::<String>("type")?, "file");
//         assert!(metadata.get::<bool>("is_accessible")?);

//         assert_eq!(env.call_function::<Vec<u8>>("read_file", path.clone())?, &[1, 2, 3]);

//         assert!(env.call_function::<()>("copy", (format!("{path}123"), format!("{path}456"))).is_err());
//         assert!(env.call_function::<()>("copy", (path.clone(), path.clone())).is_err());

//         env.call_function::<()>("copy", (path.clone(), format!("{path}_copy")))?;

//         assert!(env.call_function::<bool>("exists", format!("{path}_copy"))?);

//         env.call_function::<()>("remove_file", path.clone())?;

//         assert!(!env.call_function::<bool>("exists", path.clone())?);

//         env.call_function::<()>("move", (format!("{path}_copy"), path.clone()))?;

//         assert!(!env.call_function::<bool>("exists", format!("{path}_copy"))?);
//         assert!(env.call_function::<bool>("exists", path.clone())?);

//         env.call_function::<()>("remove", path.clone())?;

//         assert!(!env.call_function::<bool>("exists", path.clone())?);

//         Ok(())
//     }

//     #[tokio::test]
//     async fn fs_folder_actions() -> anyhow::Result<()> {
//         let path = std::env::temp_dir().join(".agl-v1-folder-actions-test");
//         let dxvk_path = std::env::temp_dir().join(".agl-v1-folder-actions-test-dxvk.tar.gz");

//         if path.exists() {
//             std::fs::remove_dir_all(&path)?;
//         }

//         let path = path.to_string_lossy().to_string();

//         let api = FilesystemAPI::new(Lua::new())?;

//         let env = api.create_env(&Context {
//             resource_hash: Hash::rand(),
//             temp_folder: std::env::temp_dir(),
//             module_folder: std::env::temp_dir(),
//             persistent_folder: std::env::temp_dir(),
//             input_resources: vec![],
//             ext_process_api: false,
//             ext_allowed_paths: vec![],
//             local_validator: LocalValidator::open(std::env::temp_dir().join("local_validator.json"))?
//         })?;

//         assert!(!env.call_function::<bool>("exists", path.clone())?);

//         env.call_function::<()>("create_dir", path.clone())?;

//         assert!(env.call_function::<bool>("exists", path.clone())?);

//         let metadata = env.call_function::<LuaTable>("metadata", path.clone())?;

//         assert_eq!(metadata.get::<String>("type")?, "folder");
//         assert!(metadata.get::<bool>("is_accessible")?);

//         if !dxvk_path.exists() {
//             let downloader = Downloader::new()?;

//             let task = downloader.download(
//                 "https://github.com/doitsujin/dxvk/releases/download/v2.6.1/dxvk-2.6.1.tar.gz",
//                 &dxvk_path,
//                 DownloadOptions::default()
//             );

//             task.wait().await?;
//         }

//         filesystem_api::archive_extract(dxvk_path, &path, |_, _, _| {})?;

//         let path = format!("{path}/dxvk-2.6.1");

//         assert_eq!(Hash::for_entry(&path)?, Hash(10603016547360459180));

//         let entries = env.call_function::<LuaTable>("read_dir", path.clone())?;

//         assert_eq!(entries.len()?, 2);

//         for _ in 0..2 {
//             let entry = entries.pop::<LuaTable>()?;

//             assert!(["x32", "x64"].contains(&entry.get::<String>("name")?.as_str()));
//             assert!(std::fs::exists(&entry.get::<String>("path")?)?);
//             assert_eq!(entry.get::<String>("type")?, "folder");
//         }

//         assert!(!env.call_function::<bool>("exists", format!("{path}_copy"))?);

//         env.call_function::<()>("copy", (path.clone(), format!("{path}_copy")))?;

//         assert!(env.call_function::<bool>("exists", format!("{path}_copy"))?);

//         assert!(env.call_function::<()>("remove_file", path.clone()).is_err());

//         env.call_function::<()>("remove_dir", path.clone())?;

//         assert!(!env.call_function::<bool>("exists", path.clone())?);

//         env.call_function::<()>("move", (format!("{path}_copy"), path.clone()))?;

//         assert!(!env.call_function::<bool>("exists", format!("{path}_copy"))?);
//         assert!(env.call_function::<bool>("exists", path.clone())?);

//         assert_eq!(Hash::for_entry(&path)?, Hash(10603016547360459180));

//         Ok(())
//     }

//     #[test]
//     fn fs_sandbox() -> anyhow::Result<()> {
//         let path_a = std::env::temp_dir().join(".agl-v1-fs-sandbox-test-a"); // file
//         let path_b = std::env::temp_dir().join(".agl-v1-fs-sandbox-test-b"); // folder
//         let path_c = std::env::temp_dir().join(".agl-v1-fs-sandbox-test-c"); // temp

//         if path_a.exists() {
//             std::fs::remove_file(&path_a)?;
//         }

//         if path_b.exists() {
//             std::fs::remove_dir_all(&path_b)?;
//         }

//         if path_c.exists() {
//             let _ = std::fs::remove_file(&path_c)
//                 .or_else(|_| std::fs::remove_dir_all(&path_c));
//         }

//         let api = FilesystemAPI::new(Lua::new())?;

//         let env = api.create_env(&Context {
//             resource_hash: Hash::rand(),
//             temp_folder: path_a.clone(),
//             module_folder: path_b.clone(),
//             persistent_folder: path_c.clone(),
//             input_resources: vec![],
//             ext_process_api: false,
//             ext_allowed_paths: vec![],
//             local_validator: LocalValidator::open(std::env::temp_dir().join("local_validator.json"))?
//         })?;

//         let path_a = path_a.to_string_lossy().to_string();
//         let path_b = path_b.to_string_lossy().to_string();
//         let path_c = path_c.to_string_lossy().to_string();

//         let inaccessible_path = std::env::temp_dir()
//             .join(".agl-v1-fs-sandbox-test-inaccessible")
//             .to_string_lossy()
//             .to_string();

//         assert!(!env.call_function::<bool>("exists", path_a.clone())?);
//         assert!(!env.call_function::<bool>("exists", path_b.clone())?);
//         assert!(!env.call_function::<bool>("exists", path_c.clone())?);
//         assert!(!env.call_function::<bool>("exists", inaccessible_path.clone())?);

//         env.call_function::<()>("create_file", path_a.clone())?;

//         assert!(env.call_function::<bool>("exists", path_a.clone())?);

//         let metadata = env.call_function::<LuaTable>("metadata", path_a.clone())?;

//         assert_eq!(metadata.get::<u64>("length")?, 0);
//         assert_eq!(metadata.get::<String>("type")?, "file");
//         assert!(metadata.get::<bool>("is_accessible")?);

//         env.call_function::<()>("write_file", (path_a.clone(), vec![1, 2, 3]))?;

//         assert_eq!(env.call_function::<Vec<u8>>("read_file", path_a.clone())?, &[1, 2, 3]);

//         assert!(env.call_function::<()>("copy", (path_a.clone(), inaccessible_path.clone())).is_err());
//         assert!(env.call_function::<()>("move", (path_a.clone(), inaccessible_path.clone())).is_err());

//         env.call_function::<()>("move", (path_a.clone(), path_c.clone()))?;

//         assert!(!env.call_function::<bool>("exists", path_a.clone())?);
//         assert!(env.call_function::<bool>("exists", path_c.clone())?);

//         let handle = env.call_function::<i32>("open", path_c.clone())?;

//         assert_eq!(env.call_function::<Vec<u8>>("read", handle)?, &[1, 2, 3]);

//         env.call_function::<()>("close", handle)?;
//         env.call_function::<()>("remove", path_c.clone())?;

//         assert!(!env.call_function::<bool>("exists", path_c.clone())?);

//         env.call_function::<()>("create_dir", path_b.clone())?;

//         assert!(env.call_function::<bool>("exists", path_b.clone())?);

//         let metadata = env.call_function::<LuaTable>("metadata", path_b.clone())?;

//         assert_eq!(metadata.get::<String>("type")?, "folder");
//         assert!(metadata.get::<bool>("is_accessible")?);

//         assert!(env.call_function::<()>("copy", (path_b.clone(), inaccessible_path.clone())).is_err());
//         assert!(env.call_function::<()>("move", (path_b.clone(), inaccessible_path.clone())).is_err());

//         env.call_function::<()>("move", (path_b.clone(), path_c.clone()))?;

//         assert!(!env.call_function::<bool>("exists", path_b.clone())?);
//         assert!(env.call_function::<bool>("exists", path_c.clone())?);

//         env.call_function::<()>("create_file", format!("{path_c}/test"))?;

//         assert_eq!(env.call_function::<LuaTable>("read_dir", path_c.clone())?.raw_len(), 1);

//         env.call_function::<()>("remove", path_c.clone())?;

//         assert!(!env.call_function::<bool>("exists", path_c.clone())?);

//         assert!(env.call_function::<()>("create_file", inaccessible_path.clone()).is_err());
//         assert!(env.call_function::<()>("create_dir", inaccessible_path.clone()).is_err());
//         assert!(env.call_function::<Vec<u8>>("read_file", inaccessible_path.clone()).is_err());
//         assert!(env.call_function::<LuaTable>("read_dir", inaccessible_path.clone()).is_err());
//         assert!(env.call_function::<()>("write_file", (inaccessible_path.clone(), vec![1, 2, 3])).is_err());
//         assert!(env.call_function::<i32>("open", inaccessible_path.clone()).is_err());

//         Ok(())
//     }
// }
