use crate::config;

use crate::games;
use crate::games::integrations::standards::game::Edition;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameListEntry {
    pub game_name: String,
    pub game_title: String,
    pub game_developer: String,
    pub edition: Edition,
    pub card_picture: String
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GamesList {
    pub installed: Vec<GameListEntry>,
    pub available: Vec<GameListEntry>
}

#[inline]
pub fn init_games() -> anyhow::Result<()> {
    games::init()
}

#[inline]
pub fn get_games_list() -> anyhow::Result<GamesList> {
    let settings = config::get().games;

    let games = games::list()?;

    let mut installed = Vec::new();
    let mut available = Vec::with_capacity(games.len());

    for (name, game) in games {
        let settings = settings.get_game_settings(name)?;

        for edition in game.get_game_editions_list()? {
            let entry = GameListEntry {
                game_name: game.game_name.clone(),
                game_title: game.game_title.clone(),
                game_developer: game.game_developer.clone(),
                card_picture: game.get_card_picture(&edition.name)?,
                edition
            };

            if game.is_game_installed(settings.paths[&entry.edition.name].game.to_string_lossy())? {
                installed.push(entry);
            } else {
                available.push(entry);
            }
        }
    }

    Ok(GamesList {
        installed,
        available
    })
}
