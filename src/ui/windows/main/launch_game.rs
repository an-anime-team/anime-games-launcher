use std::process::Command;

use adw::prelude::*;

use anime_game_core::filesystem::merge::MergeTree;

use crate::games;
use crate::config;

use crate::components::wine::Wine;
use crate::games::integrations::standards::addons::AddonType;

use crate::games::integrations::standards::diff::{
    Diff,
    DiffStatus
};

use crate::ui::components::game_card::CardInfo;

use crate::ui::windows::main::WINDOW as MAIN_WINDOW;
use crate::ui::windows::loading::check_addons::is_addon_enabled;

pub fn addon_unavailable(addon_title: impl AsRef<str>, group_title: impl AsRef<str>) -> anyhow::Result<String> {
    let message = format!("Addon {} from group {} is unavailable or outdated. You can launch the game without it or continue to use old version", addon_title.as_ref(), group_title.as_ref());

    let (sender, receiver) = std::sync::mpsc::channel();

    gtk::glib::MainContext::default().spawn(async move {
        let window = unsafe {
            MAIN_WINDOW.as_ref().unwrap_unchecked()
        };

        let dialog = adw::MessageDialog::new(
            Some(window),
            Some("Addon is unavailable"),
            Some(&message)
        );

        dialog.add_response("stop", "Stop");
        dialog.add_response("disable", "Disable");
        dialog.add_response("continue", "Continue");

        dialog.set_response_appearance("stop", adw::ResponseAppearance::Suggested);
        dialog.set_response_appearance("disable", adw::ResponseAppearance::Destructive);
        dialog.set_response_appearance("continue", adw::ResponseAppearance::Default);

        dialog.connect_response(None, move |_, id| sender.send(id.to_string()).unwrap());

        dialog.present();
    });

    Ok(receiver.recv()?)
}

pub fn launch_game(info: &CardInfo) -> anyhow::Result<()> {
    // Get game driver
    let game = unsafe {
        games::get_unsafe(info.get_name())
    };

    // Get game settings
    let settings = config::get().games.get_game_settings(info.get_name())?;

    // Get game paths
    let Some(paths) = settings.paths.get(info.get_edition()) else {
        anyhow::bail!("Unable to find {} paths", info.get_title());
    };

    // Get game addons
    let Some(enabled_addons) = settings.addons.get(info.get_edition()) else {
        anyhow::bail!("Unable to find {} enabled addons", info.get_title());
    };

    // Init game merge tree filesystem
    let mut tree = MergeTree::create(&paths.game)?;

    // Go through game addons list
    for group in game.get_addons_list(info.get_edition())? {
        for addon in &group.addons {
            let addon_path = addon.get_installation_path(&group.name, info.get_name(), info.get_edition())?;

            // Is the addon is enabled in the settings
            if is_addon_enabled(enabled_addons, addon, &group) {
                // Get its version diff
                let diff = game.get_addon_diff(&group.name, &addon.name, addon_path.to_string_lossy(), info.get_edition())?;

                // If the addon is installed and its version is latest
                if let Some(Diff { status: DiffStatus::Latest, .. }) = diff {
                    // Merge it to the game folder if its type is "layer"
                    if addon.r#type == AddonType::Layer {
                        tree.add_layer(&addon_path)?;
                    }

                    continue;
                }

                // Ask user what to do with outdated / not installed addon
                match addon_unavailable(&addon.title, &group.title)?.as_str() {
                    // Stop the launching function
                    "stop" => return Ok(()),

                    // We technically can disable only layer addons so just continue here
                    "disable" => continue,

                    // Merge the layer if needed
                    "continue" => {
                        // Merge it to the game folder if its type is "layer"
                        if addon.r#type == AddonType::Layer {
                            tree.add_layer(&addon_path)?;
                        }
                    }

                    _ => unreachable!()
                }
            }
        }
    }

    // Prepare deployment folder
    if paths.deployment.exists() {
        std::fs::remove_dir_all(&paths.deployment)?;
    }

    std::fs::create_dir_all(&paths.deployment)?;

    let game_path = paths.deployment.join("game");
    let addons_path = paths.deployment.join("addons");

    // Symlink addons to the deployment folder
    std::os::unix::fs::symlink(&paths.addons, &addons_path)?;

    // Create game folder
    std::fs::create_dir_all(&game_path)?;

    // Mount merged game folder to the deployment folder
    tree.mount(&game_path)?;

    // Request game launch options
    let options = game.get_launch_options(
        game_path.to_string_lossy(),
        addons_path.to_string_lossy(),
        info.get_edition()
    )?;

    // Prepare launch command
    let mut command = vec![
        format!("{:?}", Wine::from_config()?.get_executable()),
        format!("{:?}", options.executable)
    ];

    command.extend(options.options);

    // Run the game
    Command::new("bash")
        .arg("-c")
        .arg(command.join(" "))
        .envs(options.environment)
        .current_dir(&game_path)
        .spawn()?
        .wait()?;

    Ok(())
}
