use crate::config;
use crate::config::games::settings::GameSettings;

use crate::games;
use crate::games::integrations::Game;
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
fn get_game_entries(game: &Game, settings: GameSettings) -> anyhow::Result<Vec<(bool, GameListEntry)>> {
    game.get_game_editions_list()?
        .into_iter()
        .map(|edition| game.get_card_picture(&edition.name)
        .map(|card_picture| GameListEntry {
            game_name: game.game_name.clone(),
            game_title: game.game_title.clone(),
            game_developer: game.game_developer.clone(),
            card_picture,
            edition
        })
        .and_then(|entry| {
            game.is_game_installed(settings.paths[&entry.edition.name].game.to_string_lossy())
                .map(|installed| (installed, entry))
        }))
        .collect::<anyhow::Result<Vec<_>>>()
}

#[inline]
pub fn get_games_list() -> anyhow::Result<GamesList> {
    let settings = config::get().games;

    let games = games::list()?;

    let mut installed = Vec::new();
    let mut available = Vec::with_capacity(games.len());

    for (name, game) in games {
        let settings = settings.get_game_settings(game)?;

        let entries = get_game_entries(game, settings)?;

        let installed_entries = entries.iter()
            .filter_map(|(installed, entry)| installed.then_some(entry))
            .cloned();

        let available_entries = entries.iter()
            .filter_map(|(installed, entry)| (!installed).then_some(entry))
            .cloned();

        installed.extend(installed_entries);
        available.extend(available_entries);
    }

    Ok(GamesList {
        installed,
        available
    })
}
