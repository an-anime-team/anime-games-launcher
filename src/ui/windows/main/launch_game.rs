use std::process::Command;

use crate::games;
use crate::config;

use crate::components::wine::Wine;
use crate::ui::components::game_card::CardInfo;

pub fn launch_game(info: &CardInfo) -> anyhow::Result<()> {
    // Get game driver
    let game = unsafe {
        games::get_unsafe(info.get_name())
    };

    // Get game settings
    let settings = config::get().games.get_game_settings(info.get_name())?;

    // Get installation folder driver
    let Some(paths) = settings.paths.get(info.get_edition()) else {
        anyhow::bail!("Unable to find {} installation path", info.get_title());
    };

    // Request game launch options
    let options = game.get_launch_options(paths.game.to_string_lossy(), info.get_edition())?;

    // Prepare launch command
    let command = [
        format!("{:?}", Wine::from_config()?.get_executable()),
        format!("{:?}", paths.game.join(options.executable))
    ];

    // TODO: support addons

    // Run the game
    Command::new("bash")
        .arg("-c")
        .arg(command.join(" "))
        .envs(options.environment)
        .current_dir(&paths.game)
        .spawn()?
        .wait()?;

    Ok(())
}
