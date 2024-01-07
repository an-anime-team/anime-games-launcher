use std::path::PathBuf;

use crate::games;

use crate::config;
use crate::config::games::settings::GameSettings;

use crate::ui::components::game_card::CardInfo;
use crate::ui::components::tasks_queue::download_diff_task::DownloadDiffQueuedTask;

use crate::games::integrations::Game;
use crate::games::integrations::standards::diff::DiffInfo;

use crate::games::integrations::standards::addons::{
    Addon,
    AddonsGroup,
    AddonType
};

use super::MainAppMsg;

type HeapResult<T> = Result<T, Box<MainAppMsg>>;

#[inline]
fn is_installed(
    game: &Game,
    group_name: impl AsRef<str>,
    addon_name: impl AsRef<str>,
    addon_path: impl AsRef<str>,
    edition: impl AsRef<str>
) -> HeapResult<bool> {
    game.is_addon_installed(group_name, addon_name, addon_path, edition)
        .map_err(|err| Box::new(MainAppMsg::ShowToast {
            title: format!("Unable to verify addon installation for {}", game.game_title),
            message: Some(err.to_string())
        }))
}

#[inline]
fn get_diff(
    game: &Game,
    group_name: impl AsRef<str>,
    addon_name: impl AsRef<str>,
    addon_path: impl AsRef<str>,
    edition: impl AsRef<str>
) -> HeapResult<DiffInfo> {
    game.get_addon_diff(group_name, addon_name, addon_path, edition)
        .map_err(|err| MainAppMsg::ShowToast {
            title: format!("Unable to find {} addon version diff", game.game_title),
            message: Some(err.to_string())
        })
        .map(|diff| diff.diff)?
        .ok_or_else(|| Box::new(MainAppMsg::ShowToast {
            title: format!("{} addon is not installed", game.game_title),
            message: None
        }))
}

#[inline]
fn get_download(
    game: &Game,
    group_name: impl AsRef<str>,
    addon_name: impl AsRef<str>,
    edition: impl AsRef<str>
) -> HeapResult<DiffInfo> {
    game.get_addon_download(group_name, addon_name, edition)
        .map_err(|err| Box::new(MainAppMsg::ShowToast {
            title: format!("Unable to find {} addon download info", game.game_title),
            message: Some(err.to_string())
        }))
        .map(|download| download.download)
}

#[inline]
fn get_diff_or_download(
    game: &Game,
    group_name: impl AsRef<str> + Copy,
    addon_name: impl AsRef<str> + Copy,
    addon_path: impl AsRef<str>,
    edition: impl AsRef<str> + Copy
) -> HeapResult<DiffInfo> {
    is_installed(game, group_name, addon_name, addon_path.as_ref(), edition)?
        .then(|| get_diff(game, group_name, addon_name, addon_path, edition))
        .unwrap_or_else(|| get_download(game, group_name, addon_name, edition))
}

#[inline]
fn get_settings(game_info: &CardInfo, config: &config::Config) -> HeapResult<GameSettings> {
    config.games.get_game_settings(game_info.get_name())
        .map_err(|err| Box::new(MainAppMsg::ShowToast {
            title: format!("Unable to find {} settings", game_info.get_title()),
            message: Some(err.to_string())
        }))
}

// TODO: reuse get_game_path function from get_game_task ?

#[inline]
fn get_game_path<'a>(game_info: &'a CardInfo, config: &'a config::Config) -> HeapResult<PathBuf> {
    get_settings(game_info, config)?
        .paths.get(game_info.get_edition())
        .ok_or_else(|| Box::new(MainAppMsg::ShowToast {
            title: format!("Unable to find {} installation path", game_info.get_title()),
            message: None
        }))
        .map(|paths| paths.game.clone())

}

#[inline]
fn get_addons_path<'a>(game_info: &'a CardInfo, config: &'a config::Config) -> HeapResult<PathBuf> {
    get_settings(game_info, config)?
        .paths.get(game_info.get_edition())
        .ok_or_else(|| Box::new(MainAppMsg::ShowToast {
            title: format!("Unable to find {} installation path", game_info.get_title()),
            message: None
        }))
        .map(|paths| paths.addons.clone())

}

#[inline]
pub fn get_download_addon_task(game_info: &CardInfo, addon: &Addon, group: &AddonsGroup, config: &config::Config) -> HeapResult<Box<DownloadDiffQueuedTask>> {
    let addon_path = get_addons_path(game_info, config)?
        .join(&group.name)
        .join(&addon.name);

    let download_path = if addon.r#type == AddonType::Module {
        get_game_path(game_info, config)?
    } else {
        addon_path.clone()
    };

    let game = unsafe {
        games::get_unsafe(game_info.get_name())
    };

    Ok(Box::new(DownloadDiffQueuedTask {
        card_info: game_info.clone(),
        download_path,
        diff_info: get_diff_or_download(
            game,
            &group.name,
            &addon.name,
            addon_path.to_string_lossy(),
            game_info.get_edition()
        )?
    }))
}
