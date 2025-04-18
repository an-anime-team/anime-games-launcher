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

pub struct NotificationOptions {
    pub title: LocalizableString,
    pub message: Option<LocalizableString>,
    pub icon: Option<String>
}

pub struct PortalsAPIOptions {
    pub show_toast: Box<dyn Fn(ToastOptions) + Send>,
    pub show_notification: Box<dyn Fn(NotificationOptions) + Send>
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
                    let message = toast_options.get::<LuaValue>("message")
                        .map_err(AsLuaError::from)
                        .and_then(|message| LocalizableString::from_lua(&message))?;

                    if let Ok(action) = toast_options.get::<LuaTable>("action") {
                        let label = action.get::<LuaValue>("label")
                            .map_err(AsLuaError::from)
                            .and_then(|label| LocalizableString::from_lua(&label))?;

                        (options.show_toast)(ToastOptions::Activatable {
                            message,
                            label,
                            callback: action.get("callback")?
                        });
                    }

                    else {
                        (options.show_toast)(ToastOptions::Simple(message));
                    }

                    Ok(())
                })?
            },

            portals_notify: {
                lua.create_function(move |_, toast_options: LuaTable| {
                    (options.show_notification)(NotificationOptions {
                        title: toast_options.get::<LuaValue>("title")
                            .map_err(AsLuaError::from)
                            .and_then(|title| LocalizableString::from_lua(&title))?,

                        message: toast_options.get::<LuaValue>("message")
                            .map_err(AsLuaError::from)
                            .and_then(|message| LocalizableString::from_lua(&message))
                            .ok(),

                        icon: toast_options.get::<LuaString>("icon").ok()
                            .map(|icon| icon.to_string_lossy().to_string())
                    });

                    Ok(())
                })?
            },

            portals_dialog: {
                lua.create_function(move |_, toast_options: LuaTable| {
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
    pub fn create_env(&self, _context: &Context) -> Result<LuaTable, PackagesEngineError> {
        let env = self.lua.create_table_with_capacity(0, 3)?;

        env.raw_set("toast", self.portals_toast.clone())?;
        env.raw_set("notify", self.portals_notify.clone())?;
        env.raw_set("dialog", self.portals_dialog.clone())?;

        Ok(env)
    }
}
