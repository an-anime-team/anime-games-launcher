use mlua::prelude::*;

use super::*;

#[derive(Debug, Clone)]
pub struct InstallationDiff {
    title: LocalizableString,
    description: Option<LocalizableString>,
    pipeline: Vec<PipelineAction>
}

impl InstallationDiff {
    pub fn from_lua(lua: Lua, table: &LuaTable) -> Result<Self, LuaError> {
        Ok(Self {
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

            pipeline: table.get::<Vec<LuaTable>>("pipeline")
                .and_then(|pipeline| {
                    pipeline.iter()
                        .map(|action| PipelineAction::from_lua(lua.clone(), action))
                        .collect::<Result<Vec<_>, _>>()
                })?
        })
    }

    /// Title of the diff.
    #[inline]
    pub fn title(&self) -> &LocalizableString {
        &self.title
    }

    /// Optional description of the diff.
    #[inline]
    pub fn description(&self) -> Option<&LocalizableString> {
        self.description.as_ref()
    }

    /// List of actions which will be executed to apply the diff.
    #[inline]
    pub fn pipeline(&self) -> &[PipelineAction] {
        &self.pipeline
    }
}
