use mlua::prelude::*;

use crate::prelude::*;
use super::*;

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

pub struct PortalsAPIOptions {
    pub show_toast: Box<dyn Fn(ToastOptions) + Send>,
    pub show_notification: Box<dyn Fn(NotificationOptions) + Send>,
    pub show_dialog: Box<dyn Fn(DialogOptions) -> Option<String> + Send>
}

pub struct PortalsAPI {
    lua: Lua,

    portals_toast: LuaFunction,
    portals_notify: LuaFunction,
    portals_dialog: LuaFunction
}

impl PortalsAPI {
    pub fn new(lua: Lua, options: PortalsAPIOptions) -> Result<Self, PackagesEngineError> {
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

            lua
        })
    }

    #[inline(always)]
    pub const fn lua(&self) -> &Lua {
        &self.lua
    }

    /// Create new lua table with API functions.
    pub fn create_env(&self, _context: &Context) -> Result<LuaTable, PackagesEngineError> {
        let env = self.lua.create_table_with_capacity(0, 3)?;

        env.raw_set("toast", self.portals_toast.clone())?;
        env.raw_set("notify", self.portals_notify.clone())?;
        env.raw_set("dialog", self.portals_dialog.clone())?;

        Ok(env)
    }
}
