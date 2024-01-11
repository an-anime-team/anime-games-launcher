use crate::config;

use crate::config::games::settings::edition_addons::GameEditionAddon;

use crate::games;
use crate::games::integrations::Game;

use crate::games::integrations::standards::diff::{
    Diff,
    DiffStatus
};

use crate::games::integrations::standards::addons::{
    Addon,
    AddonsGroup
};

use crate::ui::components::game_card::CardInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddonsListEntry {
    pub game_info: CardInfo,
    pub addon: Addon,
    pub group: AddonsGroup
}

#[inline]
pub fn is_addon_enabled(enabled_addons: &[GameEditionAddon], addon: &Addon, group: &AddonsGroup) -> bool {
    addon.required || enabled_addons.iter().any(|enabled_addon| {
        enabled_addon.group == group.name && enabled_addon.name == addon.name
    })
}

#[inline]
fn get_addon_download(
    game_info: &CardInfo,
    game: &Game,
    edition: &str,
    enabled_addons: &[GameEditionAddon],
    addon: &Addon,
    group: &AddonsGroup
) -> anyhow::Result<Option<AddonsListEntry>> {
    if is_addon_enabled(enabled_addons, addon, group) {
        let addon_path = addon.get_installation_path(&group.name, &game.manifest.game_name, edition)?;

        let installed = game.is_addon_installed(
            &group.name,
            &addon.name,
            &addon_path.to_string_lossy(),
            edition
        )?;

        let entry = AddonsListEntry {
            game_info: game_info.clone(),
            addon: addon.clone(),
            group: group.clone()
        };

        if !installed {
            return Ok(Some(entry));
        }

        let diff = game.get_addon_diff(
            &group.name,
            &addon.name,
            &addon_path.to_string_lossy(),
            edition
        )?;

        // TODO: handle "unavailable" status
        if let Some(Diff { status: DiffStatus::Outdated, .. }) = diff {
            return Ok(Some(entry));
        }
    }

    Ok(None)
}

#[inline]
pub fn get_game_addons_downloads(
    game_info: &CardInfo,
    game: &Game,
    edition: &str,
    enabled_addons: &[GameEditionAddon]
) -> anyhow::Result<Vec<AddonsListEntry>> {
    let mut result = Vec::new();

    for group in game.get_addons_list(edition)? {
        for addon in &group.addons {
            if let Some(addon) = get_addon_download(game_info, game, edition, enabled_addons, addon, &group)? {
                result.push(addon);
            }
        }
    }

    Ok(result)
}

#[inline]
pub fn get_download(pool: &rusty_pool::ThreadPool) -> anyhow::Result<Vec<AddonsListEntry>> {
    let config = config::get();

    let mut tasks = Vec::new();

    for game in games::list()?.values() {
        let settings = config.games.get_game_settings(game)?;

        // Unfortunately it's impossible to run get_game_addons and other tasks
        // in the threads as well because lua executer cannot be run in different threads
        // See multithread-lua branch for details
        tasks.push(pool.evaluate(move || -> anyhow::Result<Vec<AddonsListEntry>> {
            let mut addons = Vec::new();

            for edition in game.get_game_editions_list()? {
                let installed = game.is_game_installed(
                    &settings.paths[&edition.name].game.to_string_lossy(),
                    &edition.name
                )?;

                // Check if addons should be installed only if the game itself is installed
                if installed {
                    let enabled_addons = &settings.addons[&edition.name];

                    let game_info = CardInfo::Game {
                        name: game.manifest.game_name.clone(),
                        title: game.manifest.game_title.clone(),
                        developer: game.manifest.game_developer.clone(),
                        picture_uri: game.get_card_picture(&edition.name)?,
                        edition: edition.name.clone()
                    };

                    addons.extend(get_game_addons_downloads(&game_info, game, &edition.name, enabled_addons)?);
                }
            }

            Ok(addons)
        }));
    }

    let mut addons = Vec::new();

    for task in tasks {
        addons.extend(task.await_complete()?);
    }

    Ok(addons)
}
