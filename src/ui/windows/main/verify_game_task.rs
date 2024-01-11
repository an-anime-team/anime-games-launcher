use std::path::PathBuf;

use crate::games;

use crate::config;
use crate::config::games::settings::GameSettings;

use crate::ui::components::game_card::CardInfo;
use crate::ui::components::tasks_queue::verify_integrity_task::VerifyIntegrityQueuedTask;

use crate::games::integrations::Game;
use crate::games::integrations::standards::prelude::IntegrityInfo;

use super::MainAppMsg;

type HeapResult<T> = Result<T, Box<MainAppMsg>>;

#[inline]
fn get_integrity_info(game: &Game, game_path: &str, edition: &str) -> HeapResult<Vec<IntegrityInfo>> {
    game.get_game_integrity(game_path, edition)
        .map_err(|err| Box::new(MainAppMsg::ShowToast {
            title: format!("Unable to get {} integrity info", game.manifest.game_title),
            message: Some(err.to_string())
        }))
}

#[inline]
fn get_settings(game: &Game, config: &config::Config) -> HeapResult<GameSettings> {
    config.games.get_game_settings(game)
        .map_err(|err| Box::new(MainAppMsg::ShowToast {
            title: format!("Unable to find {} settings", game.manifest.game_title),
            message: Some(err.to_string())
        }))
}

#[inline]
fn get_game_path<'a>(game: &'a Game, edition: &str, config: &'a config::Config) -> HeapResult<PathBuf> {
    get_settings(game, config)?
        .paths.get(edition)
        .ok_or_else(|| Box::new(MainAppMsg::ShowToast {
            title: format!("Unable to find {} installation path", game.manifest.game_title),
            message: None
        }))
        .map(|paths| paths.game.clone())

}

#[inline]
pub fn get_verify_game_task(game_info: &CardInfo, config: &config::Config) -> HeapResult<Box<VerifyIntegrityQueuedTask>> {
    let game = unsafe {
        games::get_unsafe(game_info.get_name())
    };

    let game_path = get_game_path(game, game_info.get_edition(), config)?;

    Ok(Box::new(VerifyIntegrityQueuedTask {
        card_info: game_info.clone(),
        integrity_info: get_integrity_info(
            game,
            &game_path.to_string_lossy(),
            game_info.get_edition()
        )?,
        path: game_path
    }))
}
