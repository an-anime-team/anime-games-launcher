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
pub enum GameEngine {
    V1(v1_standard::GameIntegration)
}

impl GameEngine {
    pub fn from_lua(lua: Lua, table: &LuaTable) -> Result<Self, LuaError> {
        match table.get::<u32>("standard")? {
            1 => Ok(Self::V1(v1_standard::GameIntegration::from_lua(lua, table)?)),

            _ => Err(LuaError::external("unsupported game integration standard"))
        }
    }

    /// Get list of available game editions.
    #[inline]
    pub fn editions(&self, platform: TargetPlatform) -> Result<Vec<GameEdition>, LuaError> {
        match self {
            Self::V1(engine) => engine.editions(platform)
        }
    }

    /// Get status of the game installation.
    #[inline]
    pub fn game_status(&self, variant: &GameVariant) -> Result<InstallationStatus, LuaError> {
        match self {
            Self::V1(engine) => engine.game_status(variant)
        }
    }

    /// Get installation diff.
    #[inline]
    pub fn game_diff(&self, variant: &GameVariant) -> Result<Option<InstallationDiff>, LuaError> {
        match self {
            Self::V1(engine) => engine.game_diff(variant)
        }
    }

    /// Get params used to launch the game.
    #[inline]
    pub fn game_launch_info(&self, variant: &GameVariant) -> Result<GameLaunchInfo, AsLuaError> {
        match self {
            Self::V1(engine) => engine.game_launch_info(variant)
        }
    }

    /// Get settings param from the game integration module.
    #[inline]
    pub fn get_property(&self, name: impl AsRef<str>) -> Result<Option<LuaValue>, AsLuaError> {
        match self {
            Self::V1(engine) => engine.get_property(name)
        }
    }

    /// Set settings param value.
    #[inline]
    pub fn set_property(&self, name: impl AsRef<str>, value: LuaValue) -> Result<(), AsLuaError> {
        match self {
            Self::V1(engine) => engine.set_property(name, value)
        }
    }

    /// Get game settings UI layout.
    #[inline]
    pub fn get_settings_layout(&self, variant: impl AsRef<GameVariant>) -> Result<Option<Vec<GameSettingsGroup>>, AsLuaError> {
        match self {
            Self::V1(engine) => engine.get_settings_layout(variant)
        }
    }
}
