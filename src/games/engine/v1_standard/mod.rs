use std::str::FromStr;

use mlua::prelude::*;

mod game_edition;
mod game_component;
mod game_launch_status;
mod game_launch_info;
mod installation_status;
mod installation_diff;
mod pipeline_action;
mod progress_report;

pub use game_edition::*;
pub use game_component::*;
pub use game_launch_status::*;
pub use game_launch_info::*;
pub use installation_status::*;
pub use installation_diff::*;
pub use pipeline_action::*;
pub use progress_report::*;

#[derive(Debug, Clone)]
pub struct GameIntegration<'lua> {
    lua: &'lua Lua,

    editions: Vec<GameEdition>,
    components: Vec<GameComponent<'lua>>,

    game_get_status: LuaFunction<'lua>,
    game_get_diff: LuaFunction<'lua>,
    game_get_launch_info: LuaFunction<'lua>
}

impl<'lua> GameIntegration<'lua> {
    pub fn from_lua(lua: &'lua Lua, table: &LuaTable<'lua>) -> Result<Self, LuaError> {
        if table.get::<_, u32>("standard")? != 1 {
            return Err(LuaError::external("invalid game integration standard, v1 expected"));
        }

        let game = table.get::<_, LuaTable>("game")?;

        Ok(Self {
            lua,

            editions: table.get::<_, Vec<LuaTable>>("editions")
                .and_then(|editions| {
                    editions.iter()
                        .map(GameEdition::try_from)
                        .collect::<Result<Vec<_>, _>>()
                })?,

            components: table.get::<_, Vec<LuaTable>>("components")
                .and_then(|components| {
                    components.iter()
                        .map(|component| GameComponent::from_lua(lua, component))
                        .collect::<Result<Vec<_>, _>>()
                })?,

            game_get_status: game.get("get_status")?,
            game_get_diff: game.get("get_diff")?,
            game_get_launch_info: game.get("get_launch_info")?
        })
    }

    #[inline]
    /// Get list of available game editions.
    pub fn editions(&self) -> &[GameEdition] {
        &self.editions
    }

    #[inline]
    /// Get list of game components.
    pub fn components(&self) -> &[GameComponent] {
        &self.components
    }

    /// Get status of the game installation.
    pub fn game_status(&self) -> Result<InstallationStatus, LuaError> {
        self.game_get_status.call::<_, LuaString>(())
            .and_then(|status| InstallationStatus::from_str(&status.to_string_lossy()))
    }

    /// Get installation diff.
    pub fn game_diff(&self) -> Result<InstallationDiff, LuaError> {
        self.game_get_diff.call::<_, LuaTable>(())
            .and_then(|diff| InstallationDiff::from_lua(self.lua, &diff))
    }

    /// Get params used to launch the game.
    pub fn game_launch_info(&self) -> Result<GameLaunchInfo, LuaError> {
        self.game_get_launch_info.call::<_, LuaTable>(())
            .and_then(|info| GameLaunchInfo::try_from(&info))
    }
}
