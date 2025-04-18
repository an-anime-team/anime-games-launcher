use mlua::prelude::*;

use flume::Sender;

use crate::prelude::*;
use super::*;

#[derive(Debug, Clone)]
pub enum PortalMsg {
    Toast(LocalizableString),

    ActionToast {
        message: LocalizableString,
        action_label: LocalizableString,
        action_callback: LuaFunction
    }
}

pub struct PortalsAPI {
    lua: Lua,

    portals_toast: LuaFunction,
    portals_notify: LuaFunction,
    portals_dialog: LuaFunction
}

impl PortalsAPI {
    pub fn new(lua: Lua, sender: Sender<PortalMsg>) -> Result<Self, PackagesEngineError> {
        Ok(Self {
            portals_toast: {
                let sender = sender.clone();

                lua.create_function(move |_, options: LuaTable| {
                    let message = options.get::<LuaValue>("message")
                        .map_err(AsLuaError::from)
                        .and_then(|message| LocalizableString::from_lua(&message))?;

                    if let Ok(action) = options.get::<LuaTable>("action") {
                        let action_label = action.get::<LuaValue>("label")
                            .map_err(AsLuaError::from)
                            .and_then(|label| LocalizableString::from_lua(&label))?;

                        let _ = sender.send(PortalMsg::ActionToast {
                            message,
                            action_label,
                            action_callback: action.get("callback")?
                        });
                    }

                    else {
                        let _ = sender.send(PortalMsg::Toast(message));
                    }

                    Ok(())
                })?
            },

            portals_notify: {
                lua.create_function(move |_, options: LuaTable| {
                    Ok(())
                })?
            },

            portals_dialog: {
                lua.create_function(move |_, options: LuaTable| {
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
