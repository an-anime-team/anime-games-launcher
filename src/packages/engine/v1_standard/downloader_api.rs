use std::ffi::OsStr;

use mlua::prelude::*;

use super::*;

pub struct DownloaderAPI<'lua> {
    lua: &'lua Lua,

    downloader_download: LuaFunctionBuilder<'lua>
}

impl<'lua> DownloaderAPI<'lua> {
    pub fn new(lua: &'lua Lua) -> Result<Self, EngineError> {
        Ok(Self {
            lua,

            downloader_download: Box::new(|lua: &'lua Lua, context: &Context| {
                let context = context.to_owned();

                lua.create_function(move |_, (url, options): (LuaString, Option<LuaTable>)| {
                    let mut output_file = None;
                    let mut continue_downloading = true;
                    let mut progress = None;

                    // Set downloading options if they're given.
                    if let Some(options) = options {
                        if let Ok(value) = options.get::<_, LuaString>("output_file") {
                            let value = resolve_path(value.to_string_lossy())?;

                            if !context.is_accessible(&value) {
                                return Err(LuaError::external("output file is inaccessible"));
                            }

                            output_file = Some(value);
                        }

                        continue_downloading = options.get::<_, bool>("continue_downloading")
                            .unwrap_or(true);

                        if let Ok(value) = options.get::<_, LuaFunction>("progress") {
                            progress = Some(value);
                        }
                    }

                    // Prepare downloader.
                    let mut downloader = Downloader::new(url.to_string_lossy())
                        .map_err(|err| LuaError::external(format!("failed to open downloader: {err}")))?
                        .with_continue_downloading(continue_downloading);

                    downloader = match output_file {
                        Some(output_file) => downloader.with_output_file(output_file),
                        None => {
                            let output_file = downloader.output_file()
                                .file_name()
                                .unwrap_or(OsStr::new("index.html"));

                            let output_file = context.module_folder.join(output_file);

                            downloader.with_output_file(output_file)
                        }
                    };

                    // Start downloading.
                    let (send, recv) = std::sync::mpsc::channel();

                    let context = RUNTIME.block_on(async move {
                        let context = downloader.download(move |curr, total, diff| {
                            let _ = send.send((curr, total, diff));
                        }).await;

                        context.map_err(|err| {
                            LuaError::external(format!("failed to start downloader: {err}"))
                        })
                    })?;

                    // Handle downloading progress events.
                    let mut finished = false;

                    while !context.is_finished() {
                        for (curr, total, diff) in recv.try_iter() {
                            finished = curr == total;

                            if let Some(callback) = &progress {
                                callback.call::<_, ()>((curr, total, diff))?;
                            }
                        }
                    }

                    context.wait().map_err(|err| {
                        LuaError::external(format!("failed to download file: {err:?}"))
                    })?;

                    Ok(finished)
                })
            })
        })
    }

    /// Create new lua table with API functions.
    pub fn create_env(&self, context: &Context) -> Result<LuaTable<'lua>, EngineError> {
        let env = self.lua.create_table_with_capacity(0, 1)?;

        env.set("download", (self.downloader_download)(self.lua, context)?)?;

        Ok(env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downloader_download() -> anyhow::Result<()> {
        let lua = Lua::new();
        let api = DownloaderAPI::new(&lua)?;

        let env = api.create_env(&Context {
            temp_folder: std::env::temp_dir(),
            module_folder: std::env::temp_dir(),
            persistent_folder: std::env::temp_dir(),
            ext_process_api: false
        })?;

        let path = std::env::temp_dir().join(".agl-v1-downloader-test-dxvk.tar.gz");

        let options = lua.create_table()?;

        options.set("output_file", path.to_string_lossy().to_string())?;
        options.set("continue_downloading", false)?;

        let result = env.call_function::<_, bool>("download", (
            "https://github.com/doitsujin/dxvk/releases/download/v2.4/dxvk-2.4.tar.gz",
            options
        ))?;

        assert!(result);
        assert_eq!(Hash::for_entry(path)?, Hash(13290421503141924848));

        Ok(())
    }
}
