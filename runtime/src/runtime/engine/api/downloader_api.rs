use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use wineyard_core::tasks;
use wineyard_core::network::downloader::{Downloader, DownloadOptions};

use mlua::prelude::*;

use super::*;

pub const DOWNLOADER_WAIT_UPDATE_INTERVAL: Duration = Duration::from_millis(50);

pub struct DownloaderAPI {
    lua: Lua,

    downloader_create: LuaFunction,
    downloader_download: LuaFunctionBuilder,
    downloader_progress: LuaFunction,
    downloader_wait: LuaFunction,
    downloader_abort: LuaFunction,
    downloader_close: LuaFunction
}

impl DownloaderAPI {
    pub fn new(lua: Lua) -> Result<Self, PackagesEngineError> {
        let downloader_handles = Arc::new(Mutex::new(HashMap::new()));
        let tasks_handles = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            downloader_create: {
                let downloader_handles = downloader_handles.clone();

                lua.create_function(move |_, _: ()| {
                    let downloader = Downloader::new();

                    let mut handles = downloader_handles.lock()
                        .map_err(|err| {
                            LuaError::external("failed to register downloader handle")
                                .context(err)
                        })?;

                    let mut handle = rand::random::<i32>();

                    while handles.contains_key(&handle) {
                        handle = rand::random::<i32>();
                    }

                    handles.insert(handle, downloader);

                    Ok(handle)
                })?
            },

            downloader_download: {
                let downloader_handles = downloader_handles.clone();
                let tasks_handles = tasks_handles.clone();

                Box::new(move |lua: &Lua, context: &Context| {
                    let context = context.to_owned();
                    let downloader_handles = downloader_handles.clone();
                    let tasks_handles = tasks_handles.clone();

                    lua.create_function(move |_, (handle, options): (i32, LuaTable)| {
                        let url = options.get::<LuaString>("url")?;
                        let output_file = options.get::<LuaString>("output_file")?;

                        let mut output_file = resolve_path(output_file.to_string_lossy())?;

                        if output_file.is_relative() {
                            output_file = context.module_folder.join(output_file);
                        }

                        if !context.is_accessible(&output_file) {
                            return Err(LuaError::external("path is inaccessible"));
                        }

                        if let Some(parent) = output_file.parent() {
                            if !parent.is_dir() {
                                std::fs::create_dir_all(parent)?;
                            }
                        }

                        let mut download_options = DownloadOptions {
                            continue_download: true,
                            on_update: None,
                            on_finish: None
                        };

                        if let Ok(value) = options.get("continue_download") {
                            download_options.continue_download = value;
                        }

                        // If we make a rust callback in downloader options -
                        // there will be a deadlock because the lua engine is
                        // blocked by the `downloader.wait` function.
                        let on_update = options.get::<LuaFunction>("on_update").ok();
                        let on_finish = options.get::<LuaFunction>("on_finish").ok();

                        let downloader_handles = downloader_handles.lock()
                            .map_err(|err| {
                                LuaError::external("failed to register downloader handle")
                                    .context(err)
                            })?;

                        let Some(downloader) = downloader_handles.get(&handle) else {
                            return Err(LuaError::external("invalid downloader handle"));
                        };

                        let mut tasks_handles = tasks_handles.lock()
                            .map_err(|err| {
                                LuaError::external("failed to register downloader task handle")
                                    .context(err)
                            })?;

                        let task = downloader.download_with_options(
                            url.to_string_lossy(),
                            output_file,
                            download_options
                        );

                        let mut handle = rand::random::<i32>();

                        while tasks_handles.contains_key(&handle) {
                            handle = rand::random::<i32>();
                        }

                        tasks_handles.insert(handle, (task, on_update, on_finish));

                        Ok(handle)
                    })
                })
            },

            downloader_progress: {
                let tasks_handles = tasks_handles.clone();

                lua.create_function(move |lua, handle: i32| {
                    let handles = tasks_handles.lock()
                        .map_err(|err| {
                            LuaError::external("failed to read downloader handle")
                                .context(err)
                        })?;

                    let Some((task, on_update, _)) = handles.get(&handle) else {
                        return Err(LuaError::external("invalid download task handle"));
                    };

                    let progress = lua.create_table_with_capacity(0, 4)?;

                    let current = task.current();
                    let total = task.total();

                    progress.raw_set("current", current)?;
                    progress.raw_set("total", total)?;
                    progress.raw_set("fraction", task.fraction())?;
                    progress.raw_set("finished", task.is_finished())?;

                    if let Some(on_update) = on_update {
                        on_update.call::<()>((current, total))?;
                    }

                    Ok(progress)
                })?
            },

            downloader_wait: {
                let tasks_handles = tasks_handles.clone();

                lua.create_function(move |_, handle: i32| {
                    let mut handles = tasks_handles.lock()
                        .map_err(|err| {
                            LuaError::external("failed to read downloader handle")
                                .context(err)
                        })?;

                    let Some((task, on_update, on_finish)) = handles.remove(&handle) else {
                        return Err(LuaError::external("invalid download task handle"));
                    };

                    while !task.is_finished() {
                        if let Some(on_update) = &on_update {
                            let current = task.current();
                            let total = task.total();

                            on_update.call::<()>((current, total))?;
                        }

                        std::thread::sleep(DOWNLOADER_WAIT_UPDATE_INTERVAL);
                    }

                    let result = tasks::block_on(task.wait())
                        .map_err(LuaError::external)?;

                    if let Some(on_finish) = on_finish {
                        on_finish.call::<()>(result)?;
                    }

                    Ok(result)
                })?
            },

            downloader_abort: {
                let tasks_handles = tasks_handles.clone();

                lua.create_function(move |_, handle: i32| {
                    let mut handles = tasks_handles.lock()
                        .map_err(|err| {
                            LuaError::external("failed to read downloader handle")
                                .context(err)
                        })?;

                    if let Some((task, _, _)) = handles.remove(&handle) {
                        task.abort();
                    }

                    Ok(())
                })?
            },

            downloader_close: {
                let downloader_handles = downloader_handles.clone();

                lua.create_function(move |_, handle: i32| {
                    downloader_handles.lock()
                        .map_err(|err| {
                            LuaError::external("failed to read downloader handle")
                                .context(err)
                        })?
                        .remove(&handle);

                    Ok(())
                })?
            },

            lua
        })
    }

    #[inline(always)]
    pub const fn lua(&self) -> &Lua {
        &self.lua
    }

    /// Create new lua table with API functions.
    pub fn create_env(&self, context: &Context) -> Result<LuaTable, PackagesEngineError> {
        let env = self.lua.create_table_with_capacity(0, 6)?;

        env.raw_set("create", self.downloader_create.clone())?;
        env.raw_set("download", (self.downloader_download)(&self.lua, context)?)?;
        env.raw_set("progress", self.downloader_progress.clone())?;
        env.raw_set("wait", self.downloader_wait.clone())?;
        env.raw_set("abort", self.downloader_abort.clone())?;
        env.raw_set("close", self.downloader_close.clone())?;

        Ok(env)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn downloader_download() -> anyhow::Result<()> {
//         let lua = Lua::new();
//         let api = DownloaderAPI::new(lua.clone())?;

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

//         let path = std::env::temp_dir().join(".agl-v1-downloader-test-dxvk.tar.gz");

//         let downloader = env.call_function::<i64>("create", ())?;

//         let options = lua.create_table()?;

//         options.set("url", "https://github.com/doitsujin/dxvk/releases/download/v2.6.1/dxvk-2.6.1.tar.gz")?;
//         options.set("output_file", path.to_string_lossy().to_string())?;
//         options.set("continue_downloading", false)?;

//         let task = env.call_function::<i64>("download", (downloader, options))?;

//         let mut progress = env.call_function::<LuaTable>("progress", task)?;

//         while progress.get::<u64>("total")? == 0 {
//             progress = env.call_function::<LuaTable>("progress", task)?;
//         }

//         assert!(!progress.get::<bool>("finished")?);

//         let total = env.call_function::<u64>("wait", task)?;

//         assert_eq!(total, progress.get::<u64>("total")?);
//         assert_eq!(Hash::for_entry(path)?, Hash(12012134683777074236));

//         Ok(())
//     }
// }
