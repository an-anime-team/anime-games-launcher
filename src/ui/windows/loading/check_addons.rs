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
fn check_addon(
    game_info: &CardInfo,
    game: &Game,
    edition: impl AsRef<str>,
    enabled_addons: &[GameEditionAddon],
    addon: &Addon,
    group: &AddonsGroup
) -> anyhow::Result<Option<AddonsListEntry>> {
    is_addon_enabled(enabled_addons, addon, group)
        .then(|| {
            let addon_path = addon.get_installation_path(&group.name, &game.manifest.game_name, edition.as_ref())?;

            let installed = game.is_addon_installed(
                &group.name,
                &addon.name,
                addon_path.to_string_lossy(),
                edition.as_ref()
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
                addon_path.to_string_lossy(),
                edition.as_ref()
            )?;

            // TODO: handle "unavailable" status
            if let Some(Diff { status: DiffStatus::Outdated, .. }) = diff {
                return Ok(Some(entry));
            }

            Ok(None)
        })
        .unwrap_or(Ok(None))
}

#[inline]
fn get_game_addons(
    game_info: &CardInfo,
    game: &Game,
    edition: impl AsRef<str>,
    enabled_addons: &[GameEditionAddon]
) -> anyhow::Result<Vec<Option<AddonsListEntry>>> {
    game.get_addons_list(edition.as_ref())?
        .into_iter()
        .fold(vec![], |mut result, group| {
            let group_addons = group.addons.iter()
                .map(|addon| check_addon(game_info, game, edition.as_ref(), enabled_addons, addon, &group));

            result.extend(group_addons);

            result
        })
        .into_iter()
        .collect::<anyhow::Result<Vec<_>>>()
}

#[inline]
pub fn check_addons() -> anyhow::Result<Vec<AddonsListEntry>> {
    let pool = rusty_pool::Builder::new()
        .name(String::from("check_addons"))
        .build();

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
                let enabled_addons = &settings.addons[&edition.name];
    
                let game_info = CardInfo::Game {
                    name: game.manifest.game_name.clone(),
                    title: game.manifest.game_title.clone(),
                    developer: game.manifest.game_developer.clone(),
                    picture_uri: game.get_card_picture(&edition.name)?,
                    edition: edition.name.clone()
                };
    
                let game_addons = get_game_addons(&game_info, game, &edition.name, enabled_addons)?
                    .into_iter()
                    .flatten();
    
                addons.extend(game_addons);
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
