use std::str::FromStr;

use mlua::prelude::*;

use crate::games::prelude::*;

use super::*;

#[derive(Debug, Clone)]
pub struct GameComponent<'lua> {
    lua: &'lua Lua,

    name: String,
    title: LocalizableString,
    description: Option<LocalizableString>,

    required: Option<LuaValue<'lua>>,
    priority: Option<LuaValue<'lua>>,

    get_status: LuaFunction<'lua>,
    get_diff: LuaFunction<'lua>
}

impl<'lua> GameComponent<'lua> {
    pub fn from_lua(lua: &'lua Lua, table: &LuaTable<'lua>) -> Result<Self, LuaError> {
        Ok(Self {
            lua,

            name: table.get::<_, LuaString>("name")?
                .to_string_lossy()
                .to_string(),

            title: table.get::<_, LuaValue>("title")
                .and_then(|title| LocalizableString::try_from(&title))?,

            description: table.get::<_, LuaValue>("description")
                .map(|desc| LocalizableString::try_from(&desc).map(Some))
                .unwrap_or(Ok(None))?,

            required: table.get::<_, LuaValue>("required").ok(),
            priority: table.get::<_, LuaValue>("priority").ok(),

            get_status: table.get("get_status")?,
            get_diff: table.get("get_diff")?
        })
    }

    #[inline]
    /// Get unique name of the component.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    #[inline]
    /// Get title of the component.
    pub fn title(&self) -> &LocalizableString {
        &self.title
    }

    #[inline]
    /// Get description of the component.
    pub fn description(&self) -> Option<&LocalizableString> {
        self.description.as_ref()
    }

    /// Try to check if the component is required.
    /// 
    /// If set, then component will be forcely installed.
    pub fn required(&self) -> Result<Option<bool>, LuaError> {
        let Some(required) = &self.required else {
            return Ok(None);
        };

        if let Some(required) = required.as_boolean() {
            Ok(Some(required))
        } else if let Some(required) = required.as_function() {
            required.call(()).map(Some)
        } else {
            Err(LuaError::external("invalid game component 'required' field value"))
        }
    }

    /// Try to get priority of the component.
    /// 
    /// When specified, components with greater value
    /// are installed (updated) first.
    pub fn priority(&self) -> Result<Option<u32>, LuaError> {
        let Some(priority) = &self.priority else {
            return Ok(None);
        };

        if let Some(priority) = priority.as_u32() {
            Ok(Some(priority))
        } else if let Some(priority) = priority.as_function() {
            priority.call(()).map(Some)
        } else {
            Err(LuaError::external("invalid game component 'priority' field value"))
        }
    }

    /// Get status of the component installation.
    pub fn get_status(&self) -> Result<InstallationStatus, LuaError> {
        self.get_status.call::<_, LuaString>(())
            .and_then(|status| InstallationStatus::from_str(&status.to_string_lossy()))
    }

    /// Get installation diff of the component.
    pub fn get_diff(&self) -> Result<InstallationDiff, LuaError> {
        self.get_diff.call::<_, LuaTable>(())
            .and_then(|diff| InstallationDiff::from_lua(self.lua, &diff))
    }
}
