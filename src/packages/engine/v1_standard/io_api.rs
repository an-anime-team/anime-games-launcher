use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use std::time::{UNIX_EPOCH, Duration};

use mlua::prelude::*;

use bufreaderwriter::rand::BufReaderWriterRand;

use super::*;

const IO_READ_CHUNK_LEN: usize = 8192;

pub struct IOAPI<'lua> {
    lua: &'lua Lua,

    fs_exists: LuaFunction<'lua>,
    fs_metadata: LuaFunction<'lua>,
    fs_copy: LuaFunction<'lua>,
    fs_move: LuaFunction<'lua>,
    fs_remove: LuaFunction<'lua>,
    fs_open: LuaFunction<'lua>,
    fs_seek: LuaFunction<'lua>,
    fs_read: LuaFunction<'lua>,
    fs_write: LuaFunction<'lua>,
    fs_flush: LuaFunction<'lua>,
    fs_close: LuaFunction<'lua>,

    fs_read_file: LuaFunction<'lua>,
    fs_write_file: LuaFunction<'lua>,
    fs_remove_file: LuaFunction<'lua>,
    fs_create_dir: LuaFunction<'lua>,
    fs_read_dir: LuaFunction<'lua>,
    fs_remove_dir: LuaFunction<'lua>
}

impl<'lua> IOAPI<'lua> {
    pub fn new(lua: &'lua Lua) -> Result<Self, EngineError> {
        let file_handles = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            lua,

            fs_exists: lua.create_function(|_, path: LuaString| {
                let path = resolve_path(path.to_string_lossy())?;

                Ok(path.exists())
            })?,

            fs_metadata: lua.create_function(|lua, path: LuaString| {
                let path = resolve_path(path.to_string_lossy())?;

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
                        .unwrap_or_default() as u32
                })?;

                result.set("length", metadata.len() as u32)?;
                result.set("is_accessible", true)?; // TODO

                result.set("type", {
                    if metadata.is_symlink() {
                        "symlink"
                    } else if metadata.is_dir() {
                        "folder"
                    } else {
                        "file"
                    }
                })?;

                Ok(result)
            })?,

            fs_copy: lua.create_function(|_, (source, target): (LuaString, LuaString)| {
                let source = resolve_path(source.to_string_lossy())?;
                let target = resolve_path(target.to_string_lossy())?;

                // Throw an error if source path doesn't exists.
                if !source.exists() {
                    return Err(LuaError::external("source path doesn't exists"));
                }

                // Throw an error if target path already exists.
                if target.exists() {
                    return Err(LuaError::external("target path already exists"));
                }

                fn try_copy(source: &Path, target: &Path) -> std::io::Result<()> {
                    if source.is_file() {
                        std::fs::copy(source, target)?;
                    }

                    else if source.is_dir() {
                        std::fs::create_dir_all(target)?;

                        for entry in source.read_dir()? {
                            let entry = entry?;

                            try_copy(&entry.path(), &target.join(entry.file_name()))?;
                        }
                    }

                    else if source.is_symlink() {
                        if let Some(source_filename) = source.file_name() {
                            std::os::unix::fs::symlink(
                                source.read_link()?,
                                target.join(source_filename)
                            )?;
                        }
                    }

                    Ok(())
                }

                try_copy(&source, &target)?;

                Ok(())
            })?,

            fs_move: lua.create_function(|_, (source, target): (LuaString, LuaString)| {
                let source = resolve_path(source.to_string_lossy())?;
                let target = resolve_path(target.to_string_lossy())?;

                // Throw an error if source path doesn't exists.
                if !source.exists() {
                    return Err(LuaError::external("source path doesn't exists"));
                }

                // Throw an error if target path already exists.
                if target.exists() {
                    return Err(LuaError::external("target path already exists"));
                }

                fn try_move(source: &Path, target: &Path) -> std::io::Result<()> {
                    if source.is_file() {
                        // Try to rename the file (mv) or copy
                        // it and then remove the source if mv
                        // has failed (different mounts).
                        if std::fs::rename(source, target).is_err() {
                            std::fs::copy(source, target)?;
                            std::fs::remove_file(source)?;
                        }
                    }

                    else if source.is_dir() {
                        // Try to rename the folder (mv) or create
                        // a target folder and move all the files there.
                        if std::fs::rename(source, target).is_err() {
                            std::fs::create_dir_all(target)?;

                            for entry in source.read_dir()? {
                                let entry = entry?;

                                try_move(&entry.path(), &target.join(entry.file_name()))?;
                            }

                            std::fs::remove_dir_all(source)?;
                        }
                    }

                    else if source.is_symlink() {
                        if let Some(source_filename) = source.file_name() {
                            std::os::unix::fs::symlink(
                                source.read_link()?,
                                target.join(source_filename)
                            )?;
                        }

                        std::fs::remove_file(source)?;
                    }

                    Ok(())
                }

                try_move(&source, &target)?;

                Ok(())
            })?,

            fs_remove: lua.create_function(|_, path: LuaString| {
                let path = resolve_path(path.to_string_lossy())?;

                // Symlinks are resolved so we don't need to check for them.
                if path.is_file() {
                    std::fs::remove_file(path)?;
                }

                else if path.is_dir() {
                    std::fs::remove_dir_all(path)?;
                }

                Ok(())
            })?,

            fs_open: {
                let file_handles = file_handles.clone();

                lua.create_function(move |_, (path, options): (LuaString, Option<LuaTable>)| {
                    let path = resolve_path(path.to_string_lossy())?;

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

                    let mut handle = rand::random::<u32>();

                    while handles.contains_key(&handle) {
                        handle = rand::random::<u32>();
                    }

                    handles.insert(handle, BufReaderWriterRand::new_reader(file));

                    Ok(handle)
                })?
            },

            fs_seek: {
                let file_handles = file_handles.clone();

                lua.create_function(move |_, (handle, position): (u32, i32)| {
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
                        file.seek(SeekFrom::End(position as i64))?;
                    }

                    Ok(())
                })?
            },

            fs_read: {
                let file_handles = file_handles.clone();

                lua.create_function(move |_, (handle, position, length): (u32, Option<i32>, Option<u32>)| {
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
                            file.seek(SeekFrom::End(position as i64))?;
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
                        let mut buf = [0; IO_READ_CHUNK_LEN];

                        let len = file.read(&mut buf)?;

                        Ok(buf[..len].to_vec())
                    }
                })?
            },

            fs_write: {
                let file_handles = file_handles.clone();

                lua.create_function(move |_, (handle, content, position): (u32, Vec<u8>, Option<i32>)| {
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
                            file.seek(SeekFrom::End(position as i64))?;
                        }
                    }

                    // Write the content to the file.
                    file.write_all(&content)?;

                    Ok(())
                })?
            },

            fs_flush: {
                let file_handles = file_handles.clone();

                lua.create_function(move |_, handle: u32| {
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

                lua.create_function(move |_, handle: u32| {
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

            fs_read_file: lua.create_function(|_, path: LuaString| {
                let path = resolve_path(path.to_string_lossy())?;

                Ok(std::fs::read(path)?)
            })?,

            fs_write_file: lua.create_function(|_, (path, content): (LuaString, Vec<u8>)| {
                let path = resolve_path(path.to_string_lossy())?;

                std::fs::write(path, &content)?;

                Ok(())
            })?,

            fs_remove_file: lua.create_function(|_, path: LuaString| {
                let path = resolve_path(path.to_string_lossy())?;

                std::fs::remove_file(path)?;

                Ok(())
            })?,

            fs_create_dir: lua.create_function(|_, path: LuaString| {
                let path = resolve_path(path.to_string_lossy())?;

                std::fs::create_dir_all(path)?;

                Ok(())
            })?,

            fs_read_dir: lua.create_function(|lua, path: LuaString| {
                let path = resolve_path(path.to_string_lossy())?;

                let entries = lua.create_table()?;

                for entry in path.read_dir()? {
                    let entry = entry?;
                    let entry_table = lua.create_table()?;

                    entry_table.set("name", entry.file_name().to_string_lossy().to_string())?;
                    entry_table.set("path", entry.path().to_string_lossy().to_string())?;

                    entry_table.set("type", {
                        if entry.path().is_symlink() {
                            "symlink"
                        } else if entry.path().is_dir() {
                            "folder"
                        } else {
                            "file"
                        }
                    })?;

                    entries.push(entry_table)?;
                }

                Ok(entries)
            })?,

            fs_remove_dir: lua.create_function(|_, path: LuaString| {
                let path = resolve_path(path.to_string_lossy())?;

                std::fs::remove_dir_all(path)?;

                Ok(())
            })?
        })
    }

    /// Create new lua table with API functions.
    pub fn create_env(&self) -> Result<LuaTable<'lua>, EngineError> {
        let env = self.lua.create_table_with_capacity(0, 17)?;

        env.set("exists", self.fs_exists.clone())?;
        env.set("metadata", self.fs_metadata.clone())?;
        env.set("copy", self.fs_copy.clone())?;
        env.set("move", self.fs_move.clone())?;
        env.set("remove", self.fs_remove.clone())?;
        env.set("open", self.fs_open.clone())?;
        env.set("seek", self.fs_seek.clone())?;
        env.set("read", self.fs_read.clone())?;
        env.set("write", self.fs_write.clone())?;
        env.set("flush", self.fs_flush.clone())?;
        env.set("close", self.fs_close.clone())?;

        env.set("read_file", self.fs_read_file.clone())?;
        env.set("write_file", self.fs_write_file.clone())?;
        env.set("remove_file", self.fs_remove_file.clone())?;
        env.set("create_dir", self.fs_create_dir.clone())?;
        env.set("read_dir", self.fs_read_dir.clone())?;
        env.set("remove_dir", self.fs_remove_dir.clone())?;

        Ok(env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fs_file_handle() -> anyhow::Result<()> {
        let path = std::env::temp_dir().join(".agl-v1-file-handle-test");

        if path.exists() {
            std::fs::remove_file(&path)?;
        }

        let path = path.to_string_lossy().to_string();

        let lua = Lua::new();
        let api = IOAPI::new(&lua)?;

        assert!(!api.fs_exists.call::<_, bool>(path.clone())?);
        assert!(api.fs_open.call::<_, u64>(path.clone()).is_err());

        let options = lua.create_table()?;

        options.set("read", true)?;
        options.set("write", true)?;
        options.set("create", true)?;

        let handle = api.fs_open.call::<_, u64>((path.clone(), options))?;

        assert_eq!(api.fs_read.call::<_, Vec<u8>>(handle)?.len(), 0);

        api.fs_write.call::<_, ()>((handle, b"Hello, ".to_vec()))?;
        api.fs_write.call::<_, ()>((handle, b"World!".to_vec()))?;
        api.fs_flush.call::<_, ()>(handle)?;

        api.fs_seek.call::<_, ()>((handle, 0))?;

        assert_eq!(api.fs_read.call::<_, Vec<u8>>(handle)?, b"Hello, World!");

        api.fs_seek.call::<_, ()>((handle, 0))?;
        api.fs_write.call::<_, ()>((handle, b"Amogus".to_vec()))?;
        api.fs_flush.call::<_, ()>(handle)?;

        api.fs_seek.call::<_, ()>((handle, 0))?;

        assert_eq!(api.fs_read.call::<_, Vec<u8>>(handle)?, b"Amogus World!");

        api.fs_seek.call::<_, ()>((handle, -6))?;
        api.fs_write.call::<_, ()>((handle, b"Amogus".to_vec()))?;
        api.fs_flush.call::<_, ()>(handle)?;

        api.fs_seek.call::<_, ()>((handle, 0))?;

        assert_eq!(api.fs_read.call::<_, Vec<u8>>(handle)?, b"Amogus Amogus");

        api.fs_seek.call::<_, ()>((handle, 0))?;
        api.fs_write.call::<_, ()>((handle, b"Sugoma".to_vec()))?;

        assert_eq!(api.fs_read.call::<_, Vec<u8>>(handle)?, b" Amogus");

        api.fs_flush.call::<_, ()>(handle)?;
        api.fs_seek.call::<_, ()>((handle, 0))?;

        assert_eq!(api.fs_read.call::<_, Vec<u8>>(handle)?, b"Sugoma Amogus");
        assert_eq!(api.fs_read.call::<_, Vec<u8>>((handle, 3, 7))?, b"oma Amo");
        assert_eq!(api.fs_read.call::<_, Vec<u8>>(handle)?, b"gus");
        assert_eq!(api.fs_read.call::<_, Vec<u8>>((handle, -6))?, b"Amogus");

        api.fs_write.call::<_, ()>((handle, b"Mogusa".to_vec(), 0))?;
        api.fs_write.call::<_, ()>((handle, b"Susoma".to_vec(), 7))?;

        assert_eq!(api.fs_read.call::<_, Vec<u8>>((handle, 0))?, b"Mogusa Susoma");

        api.fs_close.call::<_, ()>(handle)?;

        assert!(api.fs_read.call::<_, Vec<u8>>(handle).is_err());

        Ok(())
    }

    #[test]
    fn fs_file_actions() -> anyhow::Result<()> {
        let path = std::env::temp_dir().join(".agl-v1-file-actions-test");

        if path.exists() {
            std::fs::remove_file(&path)?;
        }

        let path = path.to_string_lossy().to_string();

        let lua = Lua::new();
        let api = IOAPI::new(&lua)?;

        assert!(!api.fs_exists.call::<_, bool>(path.clone())?);

        api.fs_write_file.call::<_, ()>((path.clone(), vec![1, 2, 3]))?;

        assert!(api.fs_exists.call::<_, bool>(path.clone())?);

        let metadata = api.fs_metadata.call::<_, LuaTable>(path.clone())?;

        assert_eq!(metadata.get::<_, u32>("length")?, 3);
        assert_eq!(metadata.get::<_, String>("type")?, "file");
        assert!(metadata.get::<_, bool>("is_accessible")?);

        assert_eq!(api.fs_read_file.call::<_, Vec<u8>>(path.clone())?, &[1, 2, 3]);

        assert!(api.fs_copy.call::<_, ()>((format!("{path}123"), format!("{path}456"))).is_err());
        assert!(api.fs_copy.call::<_, ()>((path.clone(), path.clone())).is_err());

        api.fs_copy.call::<_, ()>((path.clone(), format!("{path}_copy")))?;

        assert!(api.fs_exists.call::<_, bool>(format!("{path}_copy"))?);

        api.fs_remove_file.call::<_, ()>(path.clone())?;

        assert!(!api.fs_exists.call::<_, bool>(path.clone())?);

        api.fs_move.call::<_, ()>((format!("{path}_copy"), path.clone()))?;

        assert!(!api.fs_exists.call::<_, bool>(format!("{path}_copy"))?);
        assert!(api.fs_exists.call::<_, bool>(path.clone())?);

        api.fs_remove.call::<_, ()>(path.clone())?;

        assert!(!api.fs_exists.call::<_, bool>(path.clone())?);

        Ok(())
    }

    #[tokio::test]
    async fn fs_folder_actions() -> anyhow::Result<()> {
        let path = std::env::temp_dir().join(".agl-v1-folder-actions-test");
        let dxvk_path = std::env::temp_dir().join(".agl-v1-folder-actions-test-dxvk.tar.gz");

        if path.exists() {
            std::fs::remove_dir_all(&path)?;
        }

        let path = path.to_string_lossy().to_string();

        let lua = Lua::new();
        let api = IOAPI::new(&lua)?;

        assert!(!api.fs_exists.call::<_, bool>(path.clone())?);

        api.fs_create_dir.call::<_, ()>(path.clone())?;

        assert!(api.fs_exists.call::<_, bool>(path.clone())?);

        let metadata = api.fs_metadata.call::<_, LuaTable>(path.clone())?;

        assert_eq!(metadata.get::<_, String>("type")?, "folder");
        assert!(metadata.get::<_, bool>("is_accessible")?);

        if !dxvk_path.exists() {
            Downloader::new("https://github.com/doitsujin/dxvk/releases/download/v2.4/dxvk-2.4.tar.gz")
                .map_err(|err| anyhow::anyhow!(err.to_string()))?
                .with_output_file(&dxvk_path)
                .download(|_, _, _| {})
                .await
                .map_err(|err| anyhow::anyhow!(err.to_string()))?
                .wait()
                .map_err(|err| anyhow::anyhow!(err.to_string()))?;
        }

        super::archive_extract(dxvk_path, &path, |_, _, _| {})?;

        let path = format!("{path}/dxvk-2.4");

        assert_eq!(Hash::for_entry(&path)?, Hash(15040088835594252178));

        let entries = api.fs_read_dir.call::<_, LuaTable>(path.clone())?;

        assert_eq!(entries.len()?, 2);

        for _ in 0..2 {
            let entry = entries.pop::<LuaTable>()?;

            assert!(["x32", "x64"].contains(&entry.get::<_, String>("name")?.as_str()));
            assert!(std::fs::exists(&entry.get::<_, String>("path")?)?);
            assert_eq!(entry.get::<_, String>("type")?, "folder");
        }

        assert!(!api.fs_exists.call::<_, bool>(format!("{path}_copy"))?);

        api.fs_copy.call::<_, ()>((path.clone(), format!("{path}_copy")))?;

        assert!(api.fs_exists.call::<_, bool>(format!("{path}_copy"))?);

        assert!(api.fs_remove_file.call::<_, ()>(path.clone()).is_err());

        api.fs_remove_dir.call::<_, ()>(path.clone())?;

        assert!(!api.fs_exists.call::<_, bool>(path.clone())?);

        api.fs_move.call::<_, ()>((format!("{path}_copy"), path.clone()))?;

        assert!(!api.fs_exists.call::<_, bool>(format!("{path}_copy"))?);
        assert!(api.fs_exists.call::<_, bool>(path.clone())?);

        assert_eq!(Hash::for_entry(&path)?, Hash(15040088835594252178));

        Ok(())
    }
}
