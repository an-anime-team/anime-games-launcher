use crate::games;

use crate::ui::components::game_card::CardInfo;

use crate::ui::components::tasks_queue::download_diff_task::{
    DownloadDiffQueuedTask,
    DiffOrigin
};

use crate::games::integrations::Game;
use crate::games::integrations::standards::diff::DiffInfo;

use crate::games::integrations::standards::addons::{
    Addon,
    AddonsGroup
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
        })?
        .and_then(|diff| diff.diff)
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
pub fn get_download_addon_task(game_info: &CardInfo, addon: &Addon, group: &AddonsGroup) -> HeapResult<Box<DownloadDiffQueuedTask>> {
    let download_path = addon.get_installation_path(&group.name, game_info.get_name(), game_info.get_edition())
        .map_err(|err| Box::new(MainAppMsg::ShowToast {
            title: format!("Unable to find {} addon installation path", game_info.get_title()),
            message: Some(err.to_string())
        }))?;

    let game = unsafe {
        games::get_unsafe(game_info.get_name())
    };

    Ok(Box::new(DownloadDiffQueuedTask {
        card_info: game_info.clone(),
        diff_info: get_diff_or_download(
            game,
            &group.name,
            &addon.name,
            download_path.to_string_lossy(),
            game_info.get_edition()
        )?,
        diff_origin: DiffOrigin::Addon {
            group_name: group.name.clone(),
            addon_name: addon.name.clone()
        },
        download_path
    }))
}
