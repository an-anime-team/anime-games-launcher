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
pub fn register_games_styles() -> anyhow::Result<()> {
    let sus = games::list()?.iter()
        .map(|(name, game)| game.get_game_editions_list()
            .map(|editions| editions.into_iter()
                .map(|edition| game.get_details_background_style(&edition.name)
                .map(|style| (name, edition.name, style)))
                .collect::<Result<Vec<_>, _>>()))
        .collect::<Result<Result<Vec<_>, _>, _>>()??;

    let mut styles = String::new();

    for entries in sus {
        for (game, edition, style) in entries {
            if let Some(style) = style {
                styles = format!("{styles} .game-details--{game}--{edition} {{ {style} }}");
            }
        }
    }

    gtk::glib::MainContext::default().spawn(async move {
        relm4::set_global_css(&styles);
    });

    Ok(())
}

#[inline]
pub fn get_games_list() -> anyhow::Result<GamesList> {
    let settings = config::get().games;

    let games = games::list()?;

    let mut installed = Vec::new();
    let mut available = Vec::with_capacity(games.len());

    for game in games.values() {
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
