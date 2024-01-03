use std::collections::HashMap;
use std::ffi::{OsStr, OsString};

use anime_game_core::filesystem::DriverExt;

use crate::config;

pub mod integrations;

static mut GAMES_SINGLETON: Option<HashMap<OsString, integrations::Game>> = None;

fn init() -> anyhow::Result<()> {
    let driver = config::get().games.integrations.to_dyn_trait();

    let mut games = HashMap::new();

    for entry in driver.read_dir(OsStr::new(""))?.flatten() {
        if entry.path().is_dir() {
            let game = integrations::Game::new(&driver, format!("{}/manifest.json", entry.file_name().to_string_lossy()))?;

            games.insert(entry.file_name(), game);
        }
    }

    unsafe {
        GAMES_SINGLETON = Some(games);
    }

    Ok(())
}

pub fn get<'a>(name: impl AsRef<str>) -> anyhow::Result<Option<&'a integrations::Game>> {
    unsafe {
        let Some(singleton) = &mut GAMES_SINGLETON else {
            init()?;

            return get(name);
        };

        if let Some(result) = singleton.get(OsStr::new(name.as_ref())) {
            return Ok(Some(result));
        }

        Ok(None)
    }
}

pub fn list<'a>() -> anyhow::Result<&'a HashMap<OsString, integrations::Game>> {
    unsafe {
        match &GAMES_SINGLETON {
            Some(singleton) => Ok(singleton),
            None => {
                init()?;

                list()
            }
        }
    }
}
