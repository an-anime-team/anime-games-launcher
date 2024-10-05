use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use mlua::prelude::*;

use super::*;

enum Archive {
    Tar(TarArchive),
    Zip(ZipArchive),
    Sevenz(SevenzArchive)
}

pub struct ArchiveAPI<'lua> {
    lua: &'lua Lua,

    archive_open: LuaFunctionBuilder<'lua>,
    archive_entries: LuaFunction<'lua>,
    archive_extract: LuaFunctionBuilder<'lua>,
    archive_close: LuaFunction<'lua>
}

impl<'lua> ArchiveAPI<'lua> {
    pub fn new(lua: &'lua Lua) -> Result<Self, EngineError> {
        let archive_handles = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            lua,

            archive_open: {
                let archive_handles = archive_handles.clone();

                Box::new(move |lua: &'lua Lua, context: &Context| {
                    let context = context.to_owned();
                    let archive_handles = archive_handles.clone();

                    lua.create_function(move |_, (path, format): (LuaString, Option<LuaString>)| {
                        let path = resolve_path(path.to_string_lossy())?;

                        if !context.is_accessible(&path) {
                            return Err(LuaError::external("path is inaccessible"));
                        }

                        // Parse the archive format.
                        let format = match format {
                            Some(format) => {
                                match format.as_bytes() {
                                    b"tar" => ArchiveFormat::Tar,
                                    b"zip" => ArchiveFormat::Zip,
                                    b"7z"  => ArchiveFormat::Sevenz,

                                    _ => return Err(LuaError::external("unsupported archive format"))
                                }
                            }

                            None => ArchiveFormat::from_path(&path)
                                .ok_or_else(|| LuaError::external("unsupported archive format"))?
                        };

                        // Try to open the archive depending on its format.
                        let archive = match format {
                            ArchiveFormat::Tar => TarArchive::open(path)
                                .map_err(|err| LuaError::external(format!("failed to open tar archive: {err}")))
                                .map(Archive::Tar)?,

                            ArchiveFormat::Zip => ZipArchive::open(path)
                                .map_err(|err| LuaError::external(format!("failed to open zip archive: {err}")))
                                .map(Archive::Zip)?,

                            ArchiveFormat::Sevenz => SevenzArchive::open(path)
                                .map_err(|err| LuaError::external(format!("failed to open 7z archive: {err}")))
                                .map(Archive::Sevenz)?,
                        };

                        // Prepare new handle and store the open archive.
                        let mut handles = archive_handles.lock()
                            .map_err(|err| LuaError::external(format!("failed to register handle: {err}")))?;

                        let mut handle = rand::random::<u32>();

                        while handles.contains_key(&handle) {
                            handle = rand::random::<u32>();
                        }

                        handles.insert(handle, archive);

                        Ok(handle)
                    })
                })
            },

            archive_entries: {
                let archive_handles = archive_handles.clone();

                lua.create_function(move |lua, handle: u32| {
                    let handles = archive_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    // Get archive object using the given handle.
                    let Some(archive) = handles.get(&handle) else {
                        return Err(LuaError::external("invalid archive handle"));
                    };

                    // Get list of archive entries depending on its format.
                    let mut entries = match archive {
                        Archive::Tar(tar) => tar.get_entries()
                            .map_err(|err| LuaError::external(format!("failed to get tar archive entries: {err}")))?,

                        Archive::Zip(zip) => zip.get_entries()
                            .map_err(|err| LuaError::external(format!("failed to get zip archive entries: {err}")))?,

                        Archive::Sevenz(sevenz) => sevenz.get_entries()
                            .map_err(|err| LuaError::external(format!("failed to get 7z archive entries: {err}")))?,
                    };

                    // Prepare the lua output.
                    let entries_table = lua.create_table_with_capacity(entries.len(), 0)?;

                    for entry in entries.drain(..) {
                        let entry_table = lua.create_table()?;

                        entry_table.set("path", entry.path.to_string_lossy())?;
                        entry_table.set("size", entry.size)?;

                        entries_table.push(entry_table)?;
                    }

                    Ok(entries_table)
                })?
            },

            archive_extract: {
                let archive_handles = archive_handles.clone();

                Box::new(move |lua: &'lua Lua, context: &Context| {
                    let context = context.to_owned();
                    let archive_handles = archive_handles.clone();

                    lua.create_function(move |_, (handle, target, progress): (u32, LuaString, Option<LuaFunction>)| {
                        let target = resolve_path(target.to_string_lossy())?;

                        if !context.is_accessible(&target) {
                            return Err(LuaError::external("target path is inaccessible"));
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

                            match archive {
                                Archive::Tar(tar) => tar.extract(target, move |curr, total, diff| {
                                        let _ = send.send((curr, total, diff));
                                    })
                                    .map_err(|err| LuaError::external(format!("failed to start extracting tar archive: {err}")))?
                                    .wait()
                                    .map_err(|err| LuaError::external(format!("failed to extract tar archive: {err:?}")))?,

                                Archive::Zip(zip) => zip.extract(target, move |curr, total, diff| {
                                        let _ = send.send((curr, total, diff));
                                    })
                                    .map_err(|err| LuaError::external(format!("failed to start extracting zip archive: {err}")))?
                                    .wait()
                                    .map_err(|err| LuaError::external(format!("failed to extract zip archive: {err:?}")))?,

                                Archive::Sevenz(sevenz) => sevenz.extract(target, move |curr, total, diff| {
                                        let _ = send.send((curr, total, diff));
                                    })
                                    .map_err(|err| LuaError::external(format!("failed to start extracting 7z archive: {err}")))?
                                    .wait()
                                    .map_err(|err| LuaError::external(format!("failed to extract 7z archive: {err:?}")))?
                            };

                            Ok::<_, LuaError>(())
                        });

                        // Handle extraction progress events.
                        let mut finished = false;

                        while !handle.is_finished() {
                            for (curr, total, diff) in recv.try_iter() {
                                finished = curr == total;

                                if let Some(callback) = &progress {
                                    callback.call::<_, ()>((curr, total, diff))?;
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

                lua.create_function(move |_, handle: u32| {
                    archive_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?
                        .remove(&handle);

                    Ok(())
                })?
            }
        })
    }

    /// Create new lua table with API functions.
    pub fn create_env(&self, context: &Context) -> Result<LuaTable<'lua>, EngineError> {
        let env = self.lua.create_table_with_capacity(0, 4)?;

        env.set("open", (self.archive_open)(self.lua, context)?)?;
        env.set("entries", self.archive_entries.clone())?;
        env.set("extract", (self.archive_extract)(self.lua, context)?)?;
        env.set("close", self.archive_close.clone())?;

        Ok(env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn archive_entries() -> anyhow::Result<()> {
        let path = std::env::temp_dir().join(".agl-v1-archive-test-dxvk.tar.gz");

        if !path.exists() {
            Downloader::new("https://github.com/doitsujin/dxvk/releases/download/v2.4/dxvk-2.4.tar.gz").unwrap()
                .with_output_file(&path)
                .download(|_, _, _| {})
                .await.unwrap()
                .wait().unwrap();
        }

        let lua = Lua::new();
        let api = ArchiveAPI::new(&lua)?;

        let env = api.create_env(&Context {
            temp_folder: std::env::temp_dir(),
            module_folder: std::env::temp_dir(),
            persistent_folder: std::env::temp_dir(),
            ext_process_api: false
        })?;

        assert!(api.archive_entries.call::<_, LuaTable>(0).is_err());
        assert!(env.call_function::<_, LuaTable>("extract", 0).is_err());

        let handle = env.call_function::<_, u32>("open", path.to_string_lossy())?;
        let entries = api.archive_entries.call::<_, LuaTable>(handle)?;

        assert_eq!(entries.len()?, 13);

        let mut total_size = 0;
        let mut has_path = false;

        for entry in entries.sequence_values::<LuaTable>() {
            let entry = entry?;

            let path = entry.get::<_, String>("path")?;
            let size = entry.get::<_, u64>("size")?;

            total_size += size;

            if path == "dxvk-2.4/x64/d3d10core.dll" {
                has_path = true;
            }
        }

        assert_eq!(total_size, 25579660);
        assert!(has_path);

        api.archive_close.call::<_, ()>(handle)?;

        assert!(api.archive_entries.call::<_, LuaTable>(handle).is_err());
        assert!(env.call_function::<_, LuaTable>("extract", handle).is_err());

        Ok(())
    }

    #[tokio::test]
    async fn archive_extract() -> anyhow::Result<()> {
        let path = std::env::temp_dir().join(".agl-v1-archive-test");
        let dxvk_path = std::env::temp_dir().join(".agl-v1-archive-test-dxvk.tar.gz");

        if !dxvk_path.exists() {
            Downloader::new("https://github.com/doitsujin/dxvk/releases/download/v2.4/dxvk-2.4.tar.gz").unwrap()
                .with_output_file(&dxvk_path)
                .download(|_, _, _| {})
                .await.unwrap()
                .wait().unwrap();
        }

        let lua = Lua::new();
        let api = ArchiveAPI::new(&lua)?;

        let env = api.create_env(&Context {
            temp_folder: std::env::temp_dir(),
            module_folder: std::env::temp_dir(),
            persistent_folder: std::env::temp_dir(),
            ext_process_api: false
        })?;

        assert!(api.archive_entries.call::<_, LuaTable>(0).is_err());
        assert!(env.call_function::<_, LuaTable>("extract", 0).is_err());

        let handle = env.call_function::<_, u32>("open", dxvk_path.to_string_lossy())?;
        let result = env.call_function::<_, bool>("extract", (handle, path.to_string_lossy()))?;

        assert!(result);
        assert_eq!(Hash::for_entry(path)?, Hash(17827013605004440863));

        api.archive_close.call::<_, ()>(handle)?;

        assert!(api.archive_entries.call::<_, LuaTable>(handle).is_err());
        assert!(env.call_function::<_, LuaTable>("extract", handle).is_err());

        Ok(())
    }
}
