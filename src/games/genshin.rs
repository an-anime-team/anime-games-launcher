use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anime_game_core::game::GameExt;
use anime_game_core::game::genshin::{Game, Edition};

use anime_game_core::filesystem::DriverExt;

use crate::config;

use super::RunGameExt;

pub struct Genshin {
    driver: Arc<dyn DriverExt>,
    edition: Edition
}

impl From<&Game> for Genshin {
    #[inline]
    fn from(game: &Game) -> Self {
        Self {
            driver: game.get_driver(),
            edition: game.get_edition()
        }
    }
}

impl RunGameExt for Genshin {
    #[inline]
    fn get_game_binary(&self) -> &'static str {
        match self.edition {
            Edition::Global => "GenshinImpact.exe",
            Edition::China  => "YuanShen.exe"
        }
    }

    #[inline]
    fn deploy_game_folder(&self) -> anyhow::Result<PathBuf> {
        Ok(self.driver.deploy()?)
    }

    #[inline]
    fn dismantle_game_folder(&self) -> anyhow::Result<()> {
        Ok(self.driver.dismantle()?)
    }

    #[inline]
    fn get_user_environment(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}
