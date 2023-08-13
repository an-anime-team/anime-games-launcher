use std::collections::HashMap;
use std::process::Command;
use std::path::PathBuf;

use crate::components::wine::Wine;

pub mod genshin;

pub trait RunGameExt {
    /// Get game binary path
    fn get_game_binary(&self) -> &'static str;

    /// Deploy game folder and return path to it using game files driver
    fn deploy_game_folder(&self) -> anyhow::Result<PathBuf>;

    /// Dismantle deployed game folder using game files driver
    fn dismantle_game_folder(&self) -> anyhow::Result<()>;

    /// Get user-defined environment values
    fn get_user_environment(&self) -> HashMap<String, String>;

    /// Run the game in current thread and wait until it's closed
    fn run(&self) -> anyhow::Result<()> {
        let command = [
            format!("{:?}", Wine::from_config()?.get_executable()),
            self.get_game_binary().to_string()
        ];

        let game_folder = self.deploy_game_folder()?;

        Command::new("bash")
            .arg("-c")
            .arg(command.join(" "))
            .envs(self.get_user_environment())
            .current_dir(game_folder)
            .spawn()?
            .wait()?;

        self.dismantle_game_folder()?;

        Ok(())
    }
}
