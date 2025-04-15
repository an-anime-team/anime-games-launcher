use mlua::prelude::*;

use super::*;

#[derive(Debug, Clone)]
pub struct PipelineAction {
    lua: Lua,

    title: LocalizableString,
    description: Option<LocalizableString>,

    before: Option<LuaFunction>,
    perform: LuaFunction,
    after: Option<LuaFunction>
}

impl PipelineAction {
    pub fn from_lua(lua: Lua, table: &LuaTable) -> Result<Self, LuaError> {
        Ok(Self {
            lua,

            title: table.get::<LuaValue>("title")
                .map_err(AsLuaError::LuaError)
                .and_then(|title| LocalizableString::from_lua(&title))?,

            description: table.get::<LuaValue>("description")
                .map(|desc| -> Result<Option<LocalizableString>, LuaError> {
                    if desc.is_nil() || desc.is_null() {
                        Ok(None)
                    } else {
                        Ok(Some(LocalizableString::from_lua(&desc)?))
                    }
                })
                .unwrap_or(Ok(None))?,

            before: table.get::<LuaFunction>("before").ok(),
            perform: table.get("perform")?,
            after: table.get::<LuaFunction>("after").ok()
        })
    }

    #[inline]
    /// Get title of the action.
    pub fn title(&self) -> &LocalizableString {
        &self.title
    }

    #[inline]
    /// Get optional description of the action.
    pub fn description(&self) -> Option<&LocalizableString> {
        self.description.as_ref()
    }

    /// Call `before` hook if it's specified.
    ///
    /// If `true` is returned, then the action should be started.
    /// If `false`, then the action should be skipped.
    pub fn before(&self, progress: impl Fn(ProgressReport) -> bool + 'static) -> Result<Option<bool>, LuaError> {
        let Some(before) = &self.before else {
            return Ok(None);
        };

        let progress = self.lua.create_function(move |_, report: LuaValue| {
            Ok(progress(ProgressReport::from_lua(&report)?))
        })?;

        before.call::<bool>(progress).map(Some)
    }

    /// Perform the action.
    pub fn perform(&self, progress: impl Fn(ProgressReport) + 'static) -> Result<(), LuaError> {
        let progress = self.lua.create_function(move |_, report: LuaValue| {
            progress(ProgressReport::from_lua(&report)?);

            Ok(())
        })?;

        self.perform.call::<()>(progress)
    }

    /// Call `after` hook if it's specified.
    ///
    /// If `true` is returned, then the pipeline should continue execution.
    /// If `false`, then all the following actions should be skipped.
    pub fn after(&self, progress: impl Fn(ProgressReport) -> bool + 'static) -> Result<Option<bool>, LuaError> {
        let Some(after) = &self.after else {
            return Ok(None);
        };

        let progress = self.lua.create_function(move |_, report: LuaValue| {
            Ok(progress(ProgressReport::from_lua(&report)?))
        })?;

        after.call::<bool>(progress).map(Some)
    }
}
