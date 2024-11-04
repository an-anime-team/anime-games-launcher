use mlua::prelude::*;

use crate::games::prelude::*;

use super::*;

#[derive(Debug, Clone)]
pub struct InstallationDiff<'lua> {
    title: LocalizableString,
    description: Option<LocalizableString>,
    pipeline: Vec<PipelineAction<'lua>>
}

impl<'lua> InstallationDiff<'lua> {
    pub fn from_lua(lua: &'lua Lua, table: &LuaTable<'lua>) -> Result<Self, LuaError> {
        Ok(Self {
            title: table.get::<_, LuaValue>("title")
                .and_then(|title| LocalizableString::try_from(&title))?,

            description: table.get::<_, LuaValue>("description")
                .map(|desc| {
                    if desc.is_nil() || desc.is_null() {
                        Ok(None)
                    } else {
                        LocalizableString::try_from(&desc).map(Some)
                    }
                })
                .unwrap_or(Ok(None))?,

            pipeline: table.get::<_, Vec<LuaTable>>("pipeline")
                .and_then(|pipeline| {
                    pipeline.iter()
                        .map(|action| PipelineAction::from_lua(lua, action))
                        .collect::<Result<Vec<_>, _>>()
                })?
        })
    }

    #[inline]
    /// Title of the diff.
    pub fn title(&self) -> &LocalizableString {
        &self.title
    }

    #[inline]
    /// Optional description of the diff.
    pub fn description(&self) -> Option<&LocalizableString> {
        self.description.as_ref()
    }

    #[inline]
    /// List of actions which will be executed to apply the diff.
    pub fn pipeline(&self) -> &[PipelineAction<'lua>] {
        &self.pipeline
    }
}
