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

use std::str::FromStr;

use mlua::prelude::*;

use agl_locale::string::LocalizableString;

use super::*;

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

#[derive(Debug, Clone, PartialEq)]
pub struct DialogOptions {
    pub title: LocalizableString,
    pub message: LocalizableString,
    pub buttons: Vec<DialogButton>,
    pub can_close: bool
}

impl DialogOptions {
    pub fn to_lua(&self, lua: &Lua) -> Result<LuaTable, LuaError> {
        let buttons = lua.create_table_with_capacity(self.buttons.len(), 0)?;

        for button in &self.buttons {
            buttons.raw_push(button.to_lua(lua)?)?;
        }

        let options = lua.create_table_with_capacity(0, 4)?;

        options.raw_set("title", self.title.to_lua(lua)?)?;
        options.raw_set("message", self.message.to_lua(lua)?)?;
        options.raw_set("can_close", self.can_close)?;

        if !buttons.is_empty() {
            options.raw_set("buttons", buttons)?;
        }

        Ok(options)
    }

    pub fn from_lua(value: &LuaTable) -> Result<Self, LuaError> {
        Ok(Self {
            title: value.get::<LuaValue>("title")
                .and_then(|title| LocalizableString::from_lua(&title))?,

            message: value.get::<LuaValue>("message")
                .and_then(|message| LocalizableString::from_lua(&message))?,

            buttons: value.get::<Option<LuaTable>>("buttons")?
                .map(|raw_buttons| {
                    let mut buttons = Vec::with_capacity(raw_buttons.raw_len());

                    for button in raw_buttons.sequence_values::<LuaTable>() {
                        buttons.push(DialogButton::from_lua(&button?)?);
                    }

                    Ok::<_, LuaError>(buttons)
                })
                .unwrap_or_else(|| Ok(vec![]))?,

            can_close: value.get::<Option<bool>>("can_close")?
                .unwrap_or(true)
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DialogButton {
    pub label: LocalizableString,
    pub status: DialogButtonStatus,
    pub callback: LuaFunction
}

impl DialogButton {
    pub fn to_lua(&self, lua: &Lua) -> Result<LuaTable, LuaError> {
        let result = lua.create_table_with_capacity(0, 3)?;

        result.raw_set("label", self.label.to_lua(lua)?)?;
        result.raw_set("status", self.status.to_string())?;
        result.raw_set("callback", &self.callback)?;

        Ok(result)
    }

    pub fn from_lua(value: &LuaTable) -> Result<Self, LuaError> {
        Ok(Self {
            label: value.get::<LuaValue>("label")
                .and_then(|label| LocalizableString::from_lua(&label))?,

            status: value.get::<Option<String>>("status")
                .and_then(|status| {
                    let Some(status) = status else {
                        return Ok(DialogButtonStatus::default());
                    };

                    DialogButtonStatus::from_str(&status)
                        .map_err(|_| LuaError::external("unsupported dialog button status"))
                })?,

            callback: value.get("callback")?
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
    pub show_dialog: Box<dyn Fn(DialogOptions) + Send>,

    /// Callback used to translate localizable string.
    pub translate: fn(LocalizableString) -> String
}

pub struct PortalApi {
    lua: Lua,

    portal_toast: LuaFunction,
    portal_notify: LuaFunction,
    portal_dialog: LuaFunction,
    portal_open_file: LuaFunctionBuilder,
    portal_open_folder: LuaFunctionBuilder,
    portal_save_file: LuaFunctionBuilder
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
                    (options.show_dialog)(DialogOptions::from_lua(&dialog_options)?);

                    Ok(())
                })?
            },

            portal_open_file: Box::new(move |lua, context| {
                let module_scope = context.scope.clone();

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

                    #[allow(clippy::collapsible_else_if)]
                    if multiple {
                        if let Some(paths) = dialog.pick_files() {
                            let Ok(mut scope) = module_scope.write() else {
                                return Err(LuaError::external("failed to lock module scope"));
                            };

                            let result = lua.create_table_with_capacity(paths.len(), 0)?;

                            for path in paths {
                                scope.sandbox_read_paths.push(path.clone());

                                let path = path.to_string_lossy();

                                result.raw_push(
                                    lua.create_string(path.as_bytes())?
                                )?;
                            }

                            return Ok(LuaValue::Table(result));
                        }
                    }

                    else {
                        if let Some(path) = dialog.pick_file() {
                            let Ok(mut scope) = module_scope.write() else {
                                return Err(LuaError::external("failed to lock module scope"));
                            };

                            scope.sandbox_read_paths.push(path.clone());

                            let path = path.to_string_lossy();

                            return Ok(LuaValue::String(
                                lua.create_string(path.as_bytes())?
                            ));
                        }
                    }

                    Ok(LuaValue::Nil)
                })
            }),

            portal_open_folder: Box::new(move |lua, context| {
                let module_scope = context.scope.clone();

                lua.create_function(move |lua, open_folder_options: Option<LuaTable>| {
                    let mut dialog = rfd::FileDialog::new();

                    let mut multiple = false;

                    if let Some(open_folder_options) = open_folder_options {
                        if let Some(title) = open_folder_options.get::<Option<LuaValue>>("title")? {
                            let title = LocalizableString::from_lua(&title)?;

                            dialog = dialog.set_title((options.translate)(title));
                        }

                        if let Some(directory) = open_folder_options.get::<Option<String>>("directory")? {
                            dialog = dialog.set_directory(PathBuf::from(directory));
                        }

                        multiple = open_folder_options.get::<bool>("multiple")
                            .unwrap_or(false);
                    }

                    #[allow(clippy::collapsible_else_if)]
                    if multiple {
                        if let Some(paths) = dialog.pick_folders() {
                            let Ok(mut scope) = module_scope.write() else {
                                return Err(LuaError::external("failed to lock module scope"));
                            };

                            let result = lua.create_table_with_capacity(paths.len(), 0)?;

                            for path in paths {
                                scope.sandbox_write_paths.push(path.clone());

                                let path = path.to_string_lossy();

                                result.raw_push(
                                    lua.create_string(path.as_bytes())?
                                )?;
                            }

                            return Ok(LuaValue::Table(result));
                        }
                    }

                    else {
                        if let Some(path) = dialog.pick_folder() {
                            let Ok(mut scope) = module_scope.write() else {
                                return Err(LuaError::external("failed to lock module scope"));
                            };

                            scope.sandbox_write_paths.push(path.clone());

                            let path = path.to_string_lossy();

                            return Ok(LuaValue::String(
                                lua.create_string(path.as_bytes())?
                            ));
                        }
                    }

                    Ok(LuaValue::Nil)
                })
            }),

            portal_save_file: Box::new(move |lua, context| {
                let module_scope = context.scope.clone();

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
                        let Ok(mut scope) = module_scope.write() else {
                            return Err(LuaError::external("failed to lock module scope"));
                        };

                        scope.sandbox_write_paths.push(path.clone());

                        let path = path.to_string_lossy();

                        return Ok(LuaValue::String(
                            lua.create_string(path.as_bytes())?
                        ));
                    }

                    Ok(LuaValue::Nil)
                })
            }),

            lua
        })
    }

    /// Create new lua table with API functions.
    pub fn create_env(&self, context: &Context) -> Result<LuaTable, LuaError> {
        let env = self.lua.create_table_with_capacity(0, 6)?;

        env.raw_set("toast", &self.portal_toast)?;
        env.raw_set("notify", &self.portal_notify)?;
        env.raw_set("dialog", &self.portal_dialog)?;
        env.raw_set("open_file", (self.portal_open_file)(&self.lua, context)?)?;
        env.raw_set("open_folder", (self.portal_open_folder)(&self.lua, context)?)?;
        env.raw_set("save_file", (self.portal_save_file)(&self.lua, context)?)?;

        Ok(env)
    }
}
