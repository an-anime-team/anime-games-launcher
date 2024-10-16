use mlua::prelude::*;

pub mod v1_standard;

pub use v1_standard::{
    GameEdition,
    GameComponent,
    GameLaunchInfo,
    InstallationStatus,
    InstallationDiff
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
    pub fn editions(&self) -> &[GameEdition] {
        match self {
            Self::V1(engine) => engine.editions()
        }
    }

    #[inline]
    /// Get list of game components.
    pub fn components(&self) -> &[GameComponent] {
        match self {
            Self::V1(engine) => engine.components()
        }
    }

    #[inline]
    /// Get status of the game installation.
    pub fn game_status(&self) -> Result<InstallationStatus, LuaError> {
        match self {
            Self::V1(engine) => engine.game_status()
        }
    }

    #[inline]
    /// Get installation diff.
    pub fn game_diff(&self) -> Result<Option<InstallationDiff>, LuaError> {
        match self {
            Self::V1(engine) => engine.game_diff()
        }
    }

    #[inline]
    /// Get params used to launch the game.
    pub fn game_launch_info(&self) -> Result<GameLaunchInfo, LuaError> {
        match self {
            Self::V1(engine) => engine.game_launch_info()
        }
    }
}
