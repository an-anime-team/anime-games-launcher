use mlua::prelude::*;

use crate::prelude::*;

pub mod v1_standard;

pub use v1_standard::{
    GameEdition,
    GameVariant,
    GameLaunchStatus,
    GameLaunchInfo,
    InstallationStatus,
    InstallationDiff,
    ProgressReport,
    GameSettingsGroup,
    GameSettingsEntry,
    GameSettingsEntryReactivity,
    GameSettingsEntryFormat
};

#[derive(Debug, Clone)]
/// Unified wrapper around game integration standards.
pub enum GameEngine<'lua> {
    V1(v1_standard::GameIntegration<'lua>)
}

impl<'lua> GameEngine<'lua> {
    pub fn from_lua(lua: &'lua Lua, table: &LuaTable<'lua>) -> Result<Self, LuaError> {
        match table.get::<_, u32>("standard")? {
            1 => Ok(Self::V1(v1_standard::GameIntegration::from_lua(lua, table)?)),

            _ => Err(LuaError::external("unsupported game integration standard"))
        }
    }

    #[inline]
    /// Get list of available game editions.
    pub fn editions(&self, platform: TargetPlatform) -> Result<Vec<GameEdition>, LuaError> {
        match self {
            Self::V1(engine) => engine.editions(platform)
        }
    }

    #[inline]
    /// Get status of the game installation.
    pub fn game_status(&self, variant: &GameVariant) -> Result<InstallationStatus, LuaError> {
        match self {
            Self::V1(engine) => engine.game_status(variant)
        }
    }

    #[inline]
    /// Get installation diff.
    pub fn game_diff(&self, variant: &GameVariant) -> Result<Option<InstallationDiff>, LuaError> {
        match self {
            Self::V1(engine) => engine.game_diff(variant)
        }
    }

    #[inline]
    /// Get params used to launch the game.
    pub fn game_launch_info(&self, variant: &GameVariant) -> Result<GameLaunchInfo, AsLuaError> {
        match self {
            Self::V1(engine) => engine.game_launch_info(variant)
        }
    }

    #[inline]
    /// Get settings param from the game integration module.
    pub fn get_property(&self, name: impl AsRef<str>) -> Result<LuaValue, AsLuaError> {
        match self {
            Self::V1(engine) => engine.get_property(name)
        }
    }

    #[inline]
    /// Set settings param value.
    pub fn set_property(&self, name: impl AsRef<str>, value: LuaValue) -> Result<(), AsLuaError> {
        match self {
            Self::V1(engine) => engine.set_property(name, value)
        }
    }

    #[inline]
    /// Get game settings UI layout.
    pub fn get_settings_layout(&self, variant: impl AsRef<GameVariant>) -> Result<Vec<GameSettingsGroup>, AsLuaError> {
        match self {
            Self::V1(engine) => engine.get_settings_layout(variant)
        }
    }
}
