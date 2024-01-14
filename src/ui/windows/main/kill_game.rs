use crate::games;
use crate::config;

use crate::ui::components::game_card::CardInfo;

#[inline]
#[tracing::instrument]
pub fn kill_game(info: &CardInfo) -> anyhow::Result<()> {
    // Get game driver
    let game = unsafe {
        games::get_unsafe(info.get_name())
    };

    // Get game settings
    let config = config::get();
    let settings = config.games.get_game_settings(game)?;

    // Get game paths
    let Some(paths) = settings.paths.get(info.get_edition()) else {
        anyhow::bail!("Unable to find {} paths", info.get_title());
    };

    // Kill game process
    game.driver.kill_process(&paths.game.to_string_lossy(), info.get_edition())?;

    Ok(())
}
