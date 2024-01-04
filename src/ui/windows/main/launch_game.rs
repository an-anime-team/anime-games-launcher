use std::process::Command;

use anime_game_core::filesystem::DriverExt;

use crate::games;
use crate::config;

use crate::components::wine::Wine;
use crate::ui::components::game_card::CardInfo;

pub fn launch_game(info: &CardInfo) -> anyhow::Result<()> {
    // Get game driver
    let Some(game) = games::get(info.get_name())? else {
        anyhow::bail!("Unable to find {} integration script", info.get_title());
    };

    // Get game settings
    let settings = config::get().games.get_game_settings(info.get_name())?;

    // Get installation folder driver
    let Some(driver) = settings.paths.get(info.get_edition()) else {
        anyhow::bail!("Unable to find {} installation path", info.get_title());
    };

    let driver = driver.to_dyn_trait();

    // Deploy the game
    let path = driver.deploy()?;

    // Request game launch options
    let options = game.get_launch_options(path.to_string_lossy(), info.get_edition())?;

    // Prepare launch command
    let command = [
        format!("{:?}", Wine::from_config()?.get_executable()),
        format!("{:?}", path.join(options.executable))
    ];

    // Run the game
    Command::new("bash")
        .arg("-c")
        .arg(command.join(" "))
        .envs(options.environment)
        .current_dir(path)
        .spawn()?
        .wait()?;

    // Dismantle installation fodler
    driver.dismantle()?;

    Ok(())
}
