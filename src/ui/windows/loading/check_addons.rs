use crate::config;

use crate::config::games::settings::edition_addons::GameEditionAddon;

use crate::games;
use crate::games::integrations::Game;
use crate::games::integrations::standards::diff::DiffStatus;

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
fn is_addon_enabled(enabled_addons: &[GameEditionAddon], addon: &Addon, group: &AddonsGroup) -> bool {
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
            let addon_path = addon.get_installation_path(&group.name, &game.game_name, edition.as_ref())?;

            let diff = game.get_addon_diff(&group.name, &addon.name, addon_path.to_string_lossy(), edition.as_ref())?;

            // TODO: handle "unavailable" status
            if diff.status == DiffStatus::Outdated {
                Ok(Some(AddonsListEntry {
                    game_info: game_info.clone(),
                    addon: addon.clone(),
                    group: group.clone()
                }))
            }

            else {
                Ok(None)
            }
        })
        .unwrap_or(Ok(None))
}

#[inline]
pub fn check_addons() -> anyhow::Result<Vec<AddonsListEntry>> {
    let settings = config::get().games.settings;

    let mut addons = Vec::new();

    for game in games::list()?.values() {
        if let Some(game_settings) = settings.get(&game.game_name) {
            for edition in game.get_game_editions_list()? {
                if let Some(enabled_addons) = game_settings.addons.get(&edition.name) {
                    let game_info = CardInfo::Game {
                        name: game.game_name.clone(),
                        title: game.game_title.clone(),
                        developer: game.game_developer.clone(),
                        picture_uri: game.get_card_picture(&edition.name)?,
                        edition: edition.name.clone()
                    };

                    for group in game.get_addons_list(&edition.name)? {
                        for addon in &group.addons {
                            if let Some(addon) = check_addon(&game_info, game, &edition.name, enabled_addons, addon, &group)? {
                                addons.push(addon);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(addons)
}
