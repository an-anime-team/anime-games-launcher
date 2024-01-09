use std::path::PathBuf;

use crate::games;

use crate::config;
use crate::config::games::settings::GameSettings;

use crate::ui::components::game_card::CardInfo;

use crate::ui::components::tasks_queue::download_diff_task::{
    DownloadDiffQueuedTask,
    DiffOrigin
};

use crate::games::integrations::Game;
use crate::games::integrations::standards::diff::DiffInfo;

use super::MainAppMsg;

type HeapResult<T> = Result<T, Box<MainAppMsg>>;

#[inline]
fn is_installed(game: &Game, game_path: impl AsRef<str>) -> HeapResult<bool> {
    game.is_game_installed(game_path.as_ref())
        .map_err(|err| Box::new(MainAppMsg::ShowToast {
            title: format!("Unable to verify {} installation", game.manifest.game_title),
            message: Some(err.to_string())
        }))
}

#[inline]
fn get_diff(game: &Game, edition: impl AsRef<str>, game_path: impl AsRef<str>) -> HeapResult<DiffInfo> {
    game.get_game_diff(game_path.as_ref(), edition.as_ref())
        .map_err(|err| MainAppMsg::ShowToast {
            title: format!("Unable to find {} version diff", game.manifest.game_title),
            message: Some(err.to_string())
        })?
        .and_then(|diff| diff.diff)
        .ok_or_else(|| Box::new(MainAppMsg::ShowToast {
            title: format!("{} is not installed", game.manifest.game_title),
            message: None
        }))
}

#[inline]
fn get_download(game: &Game, edition: impl AsRef<str>) -> HeapResult<DiffInfo> {
    game.get_game_download(edition.as_ref())
        .map_err(|err| Box::new(MainAppMsg::ShowToast {
            title: format!("Unable to find {} download info", game.manifest.game_title),
            message: Some(err.to_string())
        }))
        .map(|download| download.download)
}

#[inline]
fn get_diff_or_download(game: &Game, edition: impl AsRef<str> + Copy, game_path: impl AsRef<str>) -> HeapResult<DiffInfo> {
    is_installed(game, game_path.as_ref())?
        .then(|| get_diff(game, edition, game_path))
        .unwrap_or_else(|| get_download(game, edition))
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
fn get_game_path<'a>(game: &'a Game, edition: impl AsRef<str>, config: &'a config::Config) -> HeapResult<PathBuf> {
    get_settings(game, config)?
        .paths.get(edition.as_ref())
        .ok_or_else(|| Box::new(MainAppMsg::ShowToast {
            title: format!("Unable to find {} installation path", game.manifest.game_title),
            message: None
        }))
        .map(|paths| paths.game.clone())

}

#[inline]
pub fn get_download_game_task(game_info: &CardInfo, config: &config::Config) -> HeapResult<Box<DownloadDiffQueuedTask>> {
    let game = unsafe {
        games::get_unsafe(game_info.get_name())
    };

    let game_path = get_game_path(game, game_info.get_edition(), config)?;

    Ok(Box::new(DownloadDiffQueuedTask {
        card_info: game_info.clone(),
        download_path: game_path.clone(),
        diff_info: get_diff_or_download(
            game,
            game_info.get_edition(),
            game_path.to_string_lossy()
        )?,
        diff_origin: DiffOrigin::Game
    }))
}
