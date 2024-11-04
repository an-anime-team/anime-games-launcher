use mlua::prelude::*;

use crate::games::prelude::*;

#[derive(Debug, Clone)]
pub struct ProgressReport<'lua> {
    /// Optional title of the current action.
    pub title: Option<LocalizableString>,

    /// Optional description of the current action.
    pub description: Option<LocalizableString>,

    pub progress_current: u64,
    pub progress_total: u64,

    progress_format: Option<LuaFunction<'lua>>
}

impl<'lua> ProgressReport<'lua> {
    #[inline]
    /// Return `current / total` fraction.
    pub fn fraction(&self) -> f64 {
        self.progress_current as f64 / self.progress_total as f64
    }

    /// Return formatted progress string if the formatter callback is specified.
    pub fn format(&self) -> Result<Option<LocalizableString>, LuaError> {
        let Some(format) = &self.progress_format else {
            return Ok(None);
        };

        let str = format.call::<_, LuaValue>(())?;

        LocalizableString::try_from(&str).map(Some)
    }
}

impl<'lua> TryFrom<&LuaTable<'lua>> for ProgressReport<'lua> {
    type Error = LuaError;

    fn try_from(value: &LuaTable<'lua>) -> Result<Self, Self::Error> {
        let progress = value.get::<_, LuaTable>("progress")?;

        Ok(Self {
            title: value.get::<_, LuaValue>("title")
                .map(|title| {
                    if title.is_nil() || title.is_null() {
                        Ok(None)
                    } else {
                        LocalizableString::try_from(&title).map(Some)
                    }
                })
                .unwrap_or(Ok(None))?,

            description: value.get::<_, LuaValue>("description")
                .map(|desc| {
                    if desc.is_nil() || desc.is_null() {
                        Ok(None)
                    } else {
                        LocalizableString::try_from(&desc).map(Some)
                    }
                })
                .unwrap_or(Ok(None))?,

            progress_current: progress.get("current")?,
            progress_total: progress.get("total")?,

            progress_format: progress.get::<_, LuaFunction>("format").ok()
        })
    }
}
