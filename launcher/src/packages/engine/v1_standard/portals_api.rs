use std::collections::HashMap;
use std::os::unix::ffi::OsStrExt;
use std::sync::{Arc, Mutex};
use std::fs::File;

use mlua::prelude::*;

use bufreaderwriter::rand::BufReaderWriterRand;
use unic_langid::LanguageIdentifier;

use crate::prelude::*;
use super::*;

use super::filesystem_api::IO_BUF_SIZE;

pub enum ToastOptions {
    Simple(LocalizableString),
    Activatable {
        message: LocalizableString,
        label: LocalizableString,
        callback: LuaFunction
    }
}

impl TryFrom<&LuaTable> for ToastOptions {
    type Error = AsLuaError;

    fn try_from(value: &LuaTable) -> Result<Self, Self::Error> {
        let message = value.get::<LuaValue>("message")
            .map_err(AsLuaError::from)
            .and_then(|message| LocalizableString::from_lua(&message))?;

        if let Ok(action) = value.get::<LuaTable>("action") {
            let label = action.get::<LuaValue>("label")
                .map_err(AsLuaError::from)
                .and_then(|label| LocalizableString::from_lua(&label))?;

            Ok(Self::Activatable {
                message,
                label,
                callback: action.get("callback")?
            })
        }

        else {
            Ok(Self::Simple(message))
        }
    }
}

pub struct NotificationOptions {
    pub title: LocalizableString,
    pub message: Option<LocalizableString>,
    pub icon: Option<String>
}

impl TryFrom<&LuaTable> for NotificationOptions {
    type Error = AsLuaError;

    fn try_from(value: &LuaTable) -> Result<Self, Self::Error> {
        Ok(Self {
            title: value.get::<LuaValue>("title")
                .map_err(AsLuaError::from)
                .and_then(|title| LocalizableString::from_lua(&title))?,

            message: value.get::<Option<LuaValue>>("message").ok()
                .and_then(|message| {
                    message.and_then(|message| LocalizableString::from_lua(&message).ok())
                }),

            icon: value.get::<LuaString>("icon").ok()
                .map(|icon| icon.to_string_lossy().to_string())
        })
    }
}

pub struct DialogOptions {
    pub title: LocalizableString,
    pub message: LocalizableString,
    pub buttons: Vec<DialogButton>
}

impl TryFrom<&LuaTable> for DialogOptions {
    type Error = AsLuaError;

    fn try_from(value: &LuaTable) -> Result<Self, Self::Error> {
        Ok(Self {
            title: value.get::<LuaValue>("title")
                .map_err(AsLuaError::from)
                .and_then(|title| LocalizableString::from_lua(&title))?,

            message: value.get::<LuaValue>("message")
                .map_err(AsLuaError::from)
                .and_then(|message| LocalizableString::from_lua(&message))?,

            buttons: value.get::<LuaTable>("buttons")
                .map(|raw_buttons| {
                    let mut buttons = Vec::with_capacity(raw_buttons.raw_len());

                    for button in raw_buttons.sequence_values::<LuaTable>() {
                        buttons.push(DialogButton::try_from(&button?)?);
                    }

                    Ok::<_, AsLuaError>(buttons)
                })??
        })
    }
}

pub struct DialogButton {
    pub name: String,
    pub label: LocalizableString,
    pub status: DialogButtonStatus
}

impl TryFrom<&LuaTable> for DialogButton {
    type Error = AsLuaError;

    fn try_from(value: &LuaTable) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.get("name")?,

            label: value.get::<LuaValue>("label")
                .map_err(AsLuaError::from)
                .and_then(|label| LocalizableString::from_lua(&label))?,

            status: value.get::<Option<LuaString>>("status")
                .and_then(|status| {
                    status.as_ref()
                        .map(DialogButtonStatus::try_from)
                        .unwrap_or(Ok(DialogButtonStatus::default()))
                })?
        })
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DialogButtonStatus {
    #[default]
    Normal,

    Suggested,
    Dangerous
}

impl TryFrom<&LuaString> for DialogButtonStatus {
    type Error = LuaError;

    fn try_from(value: &LuaString) -> Result<Self, Self::Error> {
        match value.as_bytes().as_ref() {
            b"normal" => Ok(Self::Normal),
            b"suggested" => Ok(Self::Suggested),
            b"dangerous" => Ok(Self::Dangerous),

            _ => Err(LuaError::FromLuaConversionError {
                from: "string",
                to: String::from("DialogButtonStatus"),
                message: None
            })
        }
    }
}

pub struct Options {
    pub show_toast: Box<dyn Fn(ToastOptions) + Send>,
    pub show_notification: Box<dyn Fn(NotificationOptions) + Send>,
    pub show_dialog: Box<dyn Fn(DialogOptions) -> Option<String> + Send>,

    pub file_handles: Arc<Mutex<HashMap<i32, BufReaderWriterRand<File>>>>
}

pub struct PortalsAPI {
    lua: Lua,

    portals_toast: LuaFunction,
    portals_notify: LuaFunction,
    portals_dialog: LuaFunction,
    portals_open_file: LuaFunction,
    portals_open_folder: LuaFunctionBuilder,
    portals_save_file: LuaFunction
}

impl PortalsAPI {
    pub fn new(lua: Lua, options: Options) -> Result<Self, PackagesEngineError> {
        Ok(Self {
            portals_toast: {
                lua.create_function(move |_, toast_options: LuaTable| {
                    (options.show_toast)(ToastOptions::try_from(&toast_options)?);

                    Ok(())
                })?
            },

            portals_notify: {
                lua.create_function(move |_, notify_options: LuaTable| {
                    (options.show_notification)(NotificationOptions::try_from(&notify_options)?);

                    Ok(())
                })?
            },

            portals_dialog: {
                lua.create_function(move |_, dialog_options: LuaTable| {
                    Ok((options.show_dialog)(DialogOptions::try_from(&dialog_options)?))
                })?
            },

            portals_open_file: {
                let file_handles = options.file_handles.clone();

                lua.create_function(move |lua, options: Option<LuaTable>| {
                    let mut dialog = rfd::FileDialog::new();

                    let mut multiple = false;

                    if let Some(options) = options {
                        if let Some(title) = options.get::<Option<LuaValue>>("title")? {
                            let title = LocalizableString::from_lua(&title)?;

                            let config = config::get();

                            let language = config.general.language.parse::<LanguageIdentifier>();

                            let title = match &language {
                                Ok(language) => title.translate(language),
                                Err(_) => title.default_translation()
                            };

                            dialog = dialog.set_title(title);
                        }

                        if let Some(directory) = options.get::<Option<LuaString>>("directory")? {
                            dialog = dialog.set_directory(PathBuf::from(directory.to_string_lossy()));
                        }

                        multiple = options.get::<bool>("multiple").unwrap_or(false);
                    }

                    fn open_file(path: impl AsRef<Path>) -> Result<BufReaderWriterRand<File>, LuaError> {
                        let file = File::options()
                            .read(true)
                            .write(true)
                            .open(path)
                            .map_err(LuaError::external)?;

                        Ok(BufReaderWriterRand::reader_with_capacity(IO_BUF_SIZE, file))
                    }

                    #[allow(clippy::collapsible_else_if)]
                    if multiple {
                        if let Some(files) = dialog.pick_files() {
                            let mut handles = file_handles.lock()
                                .map_err(|err| LuaError::external(format!("failed to register handle: {err}")))?;

                            let response = lua.create_table_with_capacity(files.len(), 0)?;

                            for file in files {
                                let mut handle = rand::random::<i32>();

                                while handles.contains_key(&handle) {
                                    handle = rand::random::<i32>();
                                }

                                handles.insert(handle, open_file(&file)?);

                                let file_details = lua.create_table_with_capacity(0, 2)?;

                                file_details.raw_set("path", file)?;
                                file_details.raw_set("handle", handle)?;

                                response.raw_push(file_details)?;
                            }

                            return Ok(LuaValue::Table(response));
                        }
                    }

                    else {
                        if let Some(file) = dialog.pick_file() {
                            let mut handles = file_handles.lock()
                                .map_err(|err| LuaError::external(format!("failed to register handle: {err}")))?;

                            let mut handle = rand::random::<i32>();

                            while handles.contains_key(&handle) {
                                handle = rand::random::<i32>();
                            }

                            handles.insert(handle, open_file(&file)?);

                            let response = lua.create_table_with_capacity(0, 2)?;

                            response.raw_set("path", file)?;
                            response.raw_set("handle", handle)?;

                            return Ok(LuaValue::Table(response));
                        }
                    }

                    Ok(LuaValue::Nil)
                })?
            },

            portals_open_folder: Box::new(move |lua, context| {
                let (resource_hash, local_validator) = (context.resource_hash, context.local_validator.clone());

                lua.create_function(move |lua, options: Option<LuaTable>| {
                    let mut dialog = rfd::FileDialog::new();

                    let mut multiple = false;

                    if let Some(options) = options {
                        if let Some(title) = options.get::<Option<LuaValue>>("title")? {
                            let title = LocalizableString::from_lua(&title)?;

                            let config = config::get();

                            let language = config.general.language.parse::<LanguageIdentifier>();

                            let title = match &language {
                                Ok(language) => title.translate(language),
                                Err(_) => title.default_translation()
                            };

                            dialog = dialog.set_title(title);
                        }

                        if let Some(directory) = options.get::<Option<LuaString>>("directory")? {
                            dialog = dialog.set_directory(PathBuf::from(directory.to_string_lossy()));
                        }

                        multiple = options.get::<bool>("multiple").unwrap_or(false);
                    }

                    #[allow(clippy::collapsible_else_if)]
                    if multiple {
                        if let Some(folders) = dialog.pick_folders() {
                            let result = lua.create_table_with_capacity(folders.len(), 0)?;

                            for folder in folders {
                                local_validator.allow_path(resource_hash, &folder)
                                    .map_err(LuaError::external)?;

                                result.raw_push(lua.create_string(folder.as_os_str().as_bytes())?)?;
                            }

                            return Ok(LuaValue::Table(result));
                        }
                    }

                    else {
                        if let Some(folder) = dialog.pick_folder() {
                            local_validator.allow_path(resource_hash, &folder)
                                .map_err(LuaError::external)?;

                            return Ok(LuaValue::String(lua.create_string(folder.as_os_str().as_bytes())?));
                        }
                    }

                    Ok(LuaValue::Nil)
                })
            }),

            portals_save_file: {
                let file_handles = options.file_handles.clone();

                lua.create_function(move |lua, options: Option<LuaTable>| {
                    let mut dialog = rfd::FileDialog::new()
                        .set_can_create_directories(true);

                    if let Some(options) = options {
                        if let Some(title) = options.get::<Option<LuaValue>>("title")? {
                            let title = LocalizableString::from_lua(&title)?;

                            let config = config::get();

                            let language = config.general.language.parse::<LanguageIdentifier>();

                            let title = match &language {
                                Ok(language) => title.translate(language),
                                Err(_) => title.default_translation()
                            };

                            dialog = dialog.set_title(title);
                        }

                        if let Some(directory) = options.get::<Option<LuaString>>("directory")? {
                            dialog = dialog.set_directory(PathBuf::from(directory.to_string_lossy()));
                        }

                        if let Some(file_name) = options.get::<Option<LuaString>>("file_name")? {
                            dialog = dialog.set_file_name(file_name.to_string_lossy());
                        }
                    }

                    if let Some(path) = dialog.save_file() {
                        let file = File::options()
                            .read(true)
                            .write(true)
                            .create(true)
                            .truncate(true)
                            .open(&path)
                            .map_err(LuaError::external)?;

                        let mut handles = file_handles.lock()
                            .map_err(|err| LuaError::external(format!("failed to register handle: {err}")))?;

                        let mut handle = rand::random::<i32>();

                        while handles.contains_key(&handle) {
                            handle = rand::random::<i32>();
                        }

                        handles.insert(handle, BufReaderWriterRand::reader_with_capacity(IO_BUF_SIZE, file));

                        let response = lua.create_table_with_capacity(0, 2)?;

                        response.raw_set("path", path)?;
                        response.raw_set("handle", handle)?;

                        return Ok(LuaValue::Table(response));
                    }

                    Ok(LuaValue::Nil)
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

        env.raw_set("toast", self.portals_toast.clone())?;
        env.raw_set("notify", self.portals_notify.clone())?;
        env.raw_set("dialog", self.portals_dialog.clone())?;
        env.raw_set("open_file", self.portals_open_file.clone())?;
        env.raw_set("open_folder", (self.portals_open_folder)(&self.lua, context)?)?;
        env.raw_set("save_file", self.portals_save_file.clone())?;

        Ok(env)
    }
}
