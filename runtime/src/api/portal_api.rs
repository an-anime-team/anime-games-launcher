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
use std::fs::File;

use mlua::prelude::*;

use bufreaderwriter::rand::BufReaderWriterRand;

use agl_locale::LocalizableString;

use super::*;
use super::filesystem_api::IO_BUF_SIZE;

#[derive(Debug, Clone, PartialEq)]
pub enum ToastOptions {
    Simple(LocalizableString),
    Activatable {
        message: LocalizableString,
        label: LocalizableString,
        callback: LuaFunction
    }
}

impl ToastOptions {
    pub fn to_lua(&self, lua: &Lua) -> Result<LuaTable, LuaError> {
        match self {
            Self::Simple(message) => {
                let options = lua.create_table_with_capacity(0, 1)?;

                options.raw_set("message", message.to_lua(lua)?)?;

                Ok(options)
            }

            Self::Activatable { message, label, callback } => {
                let options = lua.create_table_with_capacity(0, 3)?;

                options.raw_set("message", message.to_lua(lua)?)?;
                options.raw_set("label", label.to_lua(lua)?)?;
                options.raw_set("callback", callback)?;

                Ok(options)
            }
        }
    }

    pub fn from_lua(value: &LuaTable) -> Result<Self, LuaError> {
        let message = value.get::<LuaValue>("message")
            .and_then(|message| LocalizableString::from_lua(&message))?;

        if let Ok(action) = value.get::<LuaTable>("action") {
            let label = action.get::<LuaValue>("label")
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationOptions {
    pub title: LocalizableString,
    pub message: Option<LocalizableString>,
    pub icon: Option<String>
}

impl NotificationOptions {
    pub fn to_lua(&self, lua: &Lua) -> Result<LuaTable, LuaError> {
        let options = lua.create_table_with_capacity(0, 3)?;

        options.raw_set("title", self.title.to_lua(lua)?)?;

        if let Some(message) = &self.message {
            options.raw_set("message", message.to_lua(lua)?)?;
        }

        if let Some(icon) = &self.icon {
            options.raw_set("icon", icon.as_str())?;
        }

        Ok(options)
    }

    pub fn from_lua(value: &LuaTable) -> Result<Self, LuaError> {
        Ok(Self {
            title: value.get::<LuaValue>("title")
                .and_then(|title| LocalizableString::from_lua(&title))?,

            message: value.get::<Option<LuaValue>>("message").ok()
                .and_then(|message| {
                    message.and_then(|message| {
                        LocalizableString::from_lua(&message).ok()
                    })
                }),

            icon: value.get::<LuaString>("icon").ok()
                .map(|icon| icon.to_string_lossy().to_string())
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogOptions {
    pub title: LocalizableString,
    pub message: LocalizableString,
    pub buttons: Vec<DialogButton>
}

impl DialogOptions {
    pub fn to_lua(&self, lua: &Lua) -> Result<LuaTable, LuaError> {
        let buttons = lua.create_table_with_capacity(self.buttons.len(), 0)?;

        for button in &self.buttons {
            buttons.raw_push(button.to_lua(lua)?)?;
        }

        let options = lua.create_table_with_capacity(0, 3)?;

        options.raw_set("title", self.title.to_lua(lua)?)?;
        options.raw_set("message", self.message.to_lua(lua)?)?;
        options.raw_set("buttons", buttons)?;

        Ok(options)
    }

    pub fn from_lua(value: &LuaTable) -> Result<Self, LuaError> {
        Ok(Self {
            title: value.get::<LuaValue>("title")
                .and_then(|title| LocalizableString::from_lua(&title))?,

            message: value.get::<LuaValue>("message")
                .and_then(|message| LocalizableString::from_lua(&message))?,

            buttons: value.get::<LuaTable>("buttons")
                .map(|raw_buttons| {
                    let mut buttons = Vec::with_capacity(raw_buttons.raw_len());

                    for button in raw_buttons.sequence_values::<LuaTable>() {
                        buttons.push(DialogButton::from_lua(&button?)?);
                    }

                    Ok::<_, LuaError>(buttons)
                })??
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogButton {
    pub name: String,
    pub label: LocalizableString,
    pub status: DialogButtonStatus
}

impl DialogButton {
    pub fn to_lua(&self, lua: &Lua) -> Result<LuaTable, LuaError> {
        let result = lua.create_table_with_capacity(0, 3)?;

        result.raw_set("name", self.name.as_str())?;
        result.raw_set("label", self.label.to_lua(lua)?)?;
        result.raw_set("status", self.status.to_string())?;

        Ok(result)
    }

    pub fn from_lua(value: &LuaTable) -> Result<Self, LuaError> {
        Ok(Self {
            name: value.get("name")?,

            label: value.get::<LuaValue>("label")
                .and_then(|label| LocalizableString::from_lua(&label))?,

            status: value.get::<Option<String>>("status")
                .and_then(|status| {
                    let Some(status) = status else {
                        return Ok(DialogButtonStatus::default());
                    };

                    DialogButtonStatus::from_str(&status)
                        .map_err(|_| LuaError::external("unsupported dialog button status"))
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

impl std::fmt::Display for DialogButtonStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal    => f.write_str("normal"),
            Self::Suggested => f.write_str("suggested"),
            Self::Dangerous => f.write_str("dangerous")
        }
    }
}

impl FromStr for DialogButtonStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "normal" | "default"   => Ok(Self::Normal),
            "suggested"            => Ok(Self::Suggested),
            "dangerous" | "danger" => Ok(Self::Dangerous),

            _ => Err(())
        }
    }
}

pub struct PortalApiOptions {
    /// Callback used to display a toast message.
    pub show_toast: Box<dyn Fn(ToastOptions) + Send>,

    /// Callback used to display a system notification.
    pub show_notification: Box<dyn Fn(NotificationOptions) + Send>,

    /// Callback used to display a dialog.
    pub show_dialog: Box<dyn Fn(DialogOptions) -> Option<String> + Send>,

    /// Callback used to translate localizable string.
    pub translate: fn(LocalizableString) -> String,

    /// Table of filesystem API file handles.
    pub file_handles: Arc<Mutex<HashMap<i32, BufReaderWriterRand<File>>>>
}

pub struct PortalApi {
    lua: Lua,

    portal_toast: LuaFunction,
    portal_notify: LuaFunction,
    portal_dialog: LuaFunction,
    portal_open_file: LuaFunction,
    // portal_open_folder: LuaFunctionBuilder,
    portal_save_file: LuaFunction
}

impl PortalApi {
    pub fn new(lua: Lua, options: PortalApiOptions) -> Result<Self, LuaError> {
        Ok(Self {
            portal_toast: {
                lua.create_function(move |_, toast_options: LuaTable| {
                    (options.show_toast)(ToastOptions::from_lua(&toast_options)?);

                    Ok(())
                })?
            },

            portal_notify: {
                lua.create_function(move |_, notify_options: LuaTable| {
                    (options.show_notification)(NotificationOptions::from_lua(&notify_options)?);

                    Ok(())
                })?
            },

            portal_dialog: {
                lua.create_function(move |_, dialog_options: LuaTable| {
                    Ok((options.show_dialog)(DialogOptions::from_lua(&dialog_options)?))
                })?
            },

            portal_open_file: {
                let file_handles = options.file_handles.clone();

                lua.create_function(move |lua, open_file_options: Option<LuaTable>| {
                    let mut dialog = rfd::FileDialog::new();

                    let mut multiple = false;

                    if let Some(open_file_options) = open_file_options {
                        if let Some(title) = open_file_options.get::<Option<LuaValue>>("title")? {
                            let title = LocalizableString::from_lua(&title)?;

                            dialog = dialog.set_title((options.translate)(title));
                        }

                        if let Some(directory) = open_file_options.get::<Option<String>>("directory")? {
                            dialog = dialog.set_directory(PathBuf::from(directory));
                        }

                        multiple = open_file_options.get::<bool>("multiple")
                            .unwrap_or(false);
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

            // TODO: find some good way to temporary add write access to the
            //       open folder and return back this method.

            // portal_open_folder: Box::new(move |lua, context| {
            //     let (resource_hash, local_validator) = (context.resource_hash, context.local_validator.clone());

            //     lua.create_function(move |lua, options: Option<LuaTable>| {
            //         let mut dialog = rfd::FileDialog::new();

            //         let mut multiple = false;

            //         if let Some(options) = options {
            //             if let Some(title) = options.get::<Option<LuaValue>>("title")? {
            //                 let title = LocalizableString::from_lua(&title)?;

            //                 let config = config::get();

            //                 let language = config.general.language.parse::<LanguageIdentifier>();

            //                 let title = match &language {
            //                     Ok(language) => title.translate(language),
            //                     Err(_) => title.default_translation()
            //                 };

            //                 dialog = dialog.set_title(title);
            //             }

            //             if let Some(directory) = options.get::<Option<LuaString>>("directory")? {
            //                 dialog = dialog.set_directory(PathBuf::from(directory.to_string_lossy()));
            //             }

            //             multiple = options.get::<bool>("multiple").unwrap_or(false);
            //         }

            //         #[allow(clippy::collapsible_else_if)]
            //         if multiple {
            //             if let Some(folders) = dialog.pick_folders() {
            //                 let result = lua.create_table_with_capacity(folders.len(), 0)?;

            //                 for folder in folders {
            //                     local_validator.allow_path(resource_hash, &folder)
            //                         .map_err(LuaError::external)?;

            //                     result.raw_push(lua.create_string(folder.as_os_str().as_bytes())?)?;
            //                 }

            //                 return Ok(LuaValue::Table(result));
            //             }
            //         }

            //         else {
            //             if let Some(folder) = dialog.pick_folder() {
            //                 local_validator.allow_path(resource_hash, &folder)
            //                     .map_err(LuaError::external)?;

            //                 return Ok(LuaValue::String(lua.create_string(folder.as_os_str().as_bytes())?));
            //             }
            //         }

            //         Ok(LuaValue::Nil)
            //     })
            // }),

            portal_save_file: {
                let file_handles = options.file_handles.clone();

                lua.create_function(move |lua, safe_file_options: Option<LuaTable>| {
                    let mut dialog = rfd::FileDialog::new()
                        .set_can_create_directories(true);

                    if let Some(safe_file_options) = safe_file_options {
                        if let Some(title) = safe_file_options.get::<Option<LuaValue>>("title")? {
                            let title = LocalizableString::from_lua(&title)?;

                            dialog = dialog.set_title((options.translate)(title));
                        }

                        if let Some(directory) = safe_file_options.get::<Option<String>>("directory")? {
                            dialog = dialog.set_directory(PathBuf::from(directory));
                        }

                        if let Some(file_name) = safe_file_options.get::<Option<String>>("file_name")? {
                            dialog = dialog.set_file_name(file_name);
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

    /// Create new lua table with API functions.
    pub fn create_env(&self) -> Result<LuaTable, LuaError> {
        let env = self.lua.create_table_with_capacity(0, 5)?;

        env.raw_set("toast", self.portal_toast.clone())?;
        env.raw_set("notify", self.portal_notify.clone())?;
        env.raw_set("dialog", self.portal_dialog.clone())?;
        env.raw_set("open_file", self.portal_open_file.clone())?;
        // env.raw_set("open_folder", (self.portal_open_folder)(&self.lua, context)?)?;
        env.raw_set("save_file", self.portal_save_file.clone())?;

        Ok(env)
    }
}
