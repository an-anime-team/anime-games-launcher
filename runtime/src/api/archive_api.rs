// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-runtime
// Copyright (C) 2025  Nikita Podvirnyi <krypt0nn@vk.com>
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
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use agl_core::archives::{Archive, ArchiveFormat};

use mlua::prelude::*;

use super::*;

pub struct ArchiveApi {
    lua: Lua,

    archive_open: LuaFunctionBuilder,
    archive_entries: LuaFunction,
    archive_extract: LuaFunctionBuilder,
    archive_close: LuaFunction
}

impl ArchiveApi {
    pub fn new(lua: Lua) -> Result<Self, LuaError> {
        let archive_handles = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            archive_open: {
                let archive_handles = archive_handles.clone();

                Box::new(move |lua: &Lua, context: &Context| {
                    let context = context.to_owned();
                    let archive_handles = archive_handles.clone();

                    lua.create_function(move |_, (mut path, format): (PathBuf, Option<LuaString>)| {
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

                        // Parse the archive format.
                        let format = match format {
                            Some(format) => ArchiveFormat::from_str(&format.to_string_lossy())
                                .map_err(LuaError::external)?,

                            None => ArchiveFormat::from_filename(path.to_string_lossy())
                                .ok_or_else(|| LuaError::external("unsupported archive format"))?
                        };

                        // Try to open the archive depending on its format.
                        let archive = Archive::open_with_format(&path, format)
                            .ok_or_else(|| LuaError::external("failed to open archive"))?;

                        // Prepare new handle and store the open archive.
                        let mut handles = archive_handles.lock()
                            .map_err(|err| LuaError::external(format!("failed to register handle: {err}")))?;

                        let mut handle = rand::random::<i32>();

                        while handles.contains_key(&handle) {
                            handle = rand::random::<i32>();
                        }

                        handles.insert(handle, archive);

                        Ok(handle)
                    })
                })
            },

            archive_entries: {
                let archive_handles = archive_handles.clone();

                lua.create_function(move |lua, handle: i32| {
                    let handles = archive_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    // Get archive object using the given handle.
                    let Some(archive) = handles.get(&handle) else {
                        return Err(LuaError::external("invalid archive handle"));
                    };

                    // Get list of archive entries depending on its format.
                    let mut entries = archive.get_entries()
                        .map_err(|err| LuaError::external(format!("failed to get archive entries: {err}")))?;

                    // Prepare the lua output.
                    let entries_table = lua.create_table_with_capacity(entries.len(), 0)?;

                    for entry in entries.drain(..) {
                        let entry_table = lua.create_table_with_capacity(0, 2)?;

                        entry_table.raw_set("path", entry.path.to_string_lossy())?;
                        entry_table.raw_set("size", entry.size)?;

                        entries_table.raw_push(entry_table)?;
                    }

                    Ok(entries_table)
                })?
            },

            archive_extract: {
                let archive_handles = archive_handles.clone();

                Box::new(move |lua: &Lua, context: &Context| {
                    let context = context.to_owned();
                    let archive_handles = archive_handles.clone();

                    lua.create_function(move |_, (handle, mut target, progress): (i32, PathBuf, Option<LuaFunction>)| {
                        if target.is_relative() {
                            target = context.module_folder.join(target);
                        }

                        target = normalize_path(target, true)
                            .map_err(|err| {
                                LuaError::external(format!("failed to normalize path: {err}"))
                            })?;

                        if !context.can_write_path(&target)? {
                            return Err(LuaError::external("no target path write permissions"));
                        }

                        // Start extracting the archive in a background thread depending on its format.
                        let (send, recv) = std::sync::mpsc::channel();

                        let archive_handles = archive_handles.clone();

                        let handle = std::thread::spawn(move || {
                            let handles = archive_handles.lock()
                                .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                            // Get archive object using the given handle.
                            let Some(archive) = handles.get(&handle) else {
                                return Err(LuaError::external("invalid archive handle"));
                            };

                            archive
                                .extract_with_progress(target, move |curr, total, diff| {
                                    let _ = send.send((curr, total, diff));
                                })
                                .map_err(|err| LuaError::external(format!("failed to start extracting archive: {err}")))?
                                .wait()
                                .map_err(|err| LuaError::external(format!("failed to extract archive: {err:?}")))?;

                            Ok::<_, LuaError>(())
                        });

                        // Handle extraction progress events.
                        let mut finished = false;

                        while !handle.is_finished() {
                            for (curr, total, diff) in recv.try_iter() {
                                finished = curr >= total;

                                if let Some(callback) = &progress {
                                    callback.call::<()>((curr, total, diff))?;
                                }
                            }
                        }

                        handle.join().map_err(|err| {
                            LuaError::external(format!("failed to extract archive: {err:?}"))
                        })??;

                        Ok(finished)
                    })
                })
            },

            archive_close: {
                let archive_handles = archive_handles.clone();

                lua.create_function(move |_, handle: i32| {
                    archive_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?
                        .remove(&handle);

                    Ok(())
                })?
            },

            lua
        })
    }

    /// Create new lua table with API functions.
    pub fn create_env(&self, context: &Context) -> Result<LuaTable, LuaError> {
        let env = self.lua.create_table_with_capacity(0, 4)?;

        env.raw_set("open", (self.archive_open)(&self.lua, context)?)?;
        env.raw_set("entries", self.archive_entries.clone())?;
        env.raw_set("extract", (self.archive_extract)(&self.lua, context)?)?;
        env.raw_set("close", self.archive_close.clone())?;

        Ok(env)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     async fn get_archive() -> Result<PathBuf, DownloaderError> {
//         let path = std::env::temp_dir().join(".agl-v1-archive-test-dxvk.tar.gz");

//         if !path.exists() {
//             let downloader = Downloader::new()?;

//             let task = downloader.download(
//                 "https://github.com/doitsujin/dxvk/releases/download/v2.6.1/dxvk-2.6.1.tar.gz",
//                 &path,
//                 DownloadOptions::default()
//             );

//             task.wait().await?;
//         }

//         Ok(path)
//     }

//     #[tokio::test]
//     async fn archive_entries() -> anyhow::Result<()> {
//         let path = get_archive().await?;

//         let api = ArchiveAPI::new(Lua::new())?;

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

//         assert!(api.archive_entries.call::<LuaTable>(0).is_err());
//         assert!(env.call_function::<LuaTable>("extract", 0).is_err());

//         let handle = env.call_function::<i32>("open", path.to_string_lossy())?;
//         let entries = api.archive_entries.call::<LuaTable>(handle)?;

//         assert_eq!(entries.len()?, 13);

//         let mut total_size = 0;
//         let mut has_path = false;

//         for entry in entries.sequence_values::<LuaTable>() {
//             let entry = entry?;

//             let path = entry.get::<String>("path")?;
//             let size = entry.get::<u64>("size")?;

//             total_size += size;

//             if path == "dxvk-2.6.1/x64/d3d11.dll" {
//                 has_path = true;
//             }
//         }

//         assert_eq!(total_size, 28119180);
//         assert!(has_path);

//         api.archive_close.call::<()>(handle)?;

//         assert!(api.archive_entries.call::<LuaTable>(handle).is_err());
//         assert!(env.call_function::<LuaTable>("extract", handle).is_err());

//         Ok(())
//     }

//     #[tokio::test]
//     async fn archive_extract() -> anyhow::Result<()> {
//         let dxvk_file_path = get_archive().await?;
//         let dxvk_folder_path = std::env::temp_dir().join(".agl-v1-archive-test");

//         if dxvk_folder_path.exists() {
//             std::fs::remove_dir_all(&dxvk_folder_path)?;
//         }

//         let api = ArchiveAPI::new(Lua::new())?;

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

//         assert!(api.archive_entries.call::<LuaTable>(0).is_err());
//         assert!(env.call_function::<LuaTable>("extract", 0).is_err());

//         let handle = env.call_function::<i32>("open", dxvk_file_path.to_string_lossy())?;
//         let result = env.call_function::<bool>("extract", (handle, dxvk_folder_path.to_string_lossy()))?;

//         assert!(result);
//         assert_eq!(Hash::for_entry(&dxvk_folder_path)?, Hash(1628850133365029209));

//         api.archive_close.call::<()>(handle)?;

//         assert!(api.archive_entries.call::<LuaTable>(handle).is_err());
//         assert!(env.call_function::<LuaTable>("extract", handle).is_err());

//         std::fs::remove_dir_all(dxvk_folder_path)?;

//         Ok(())
//     }
// }
