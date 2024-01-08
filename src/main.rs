use std::path::PathBuf;

use relm4::prelude::*;

pub mod ui;
pub mod config;
pub mod components;
pub mod games;
pub mod utils;

use ui::windows::loading::LoadingApp;

pub const APP_ID: &str = "moe.launcher.anime-games-launcher";
pub const APP_RESOURCE_PREFIX: &str = "/moe/launcher/anime-games-launcher";

lazy_static::lazy_static! {
    /// Path to the launcher's data folder
    /// 
    /// Resolution order:
    /// 
    /// - `$LAUNCHER_FOLDER`
    /// - `$XDG_DATA_HOME/anime-games-launcher`
    /// - `$HOME/.local/share/anime-games-launcher`
    pub static ref LAUNCHER_FOLDER: PathBuf = {
        std::env::var("LAUNCHER_FOLDER")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::var("XDG_DATA_HOME")
                .map(|data| PathBuf::from(data).join("anime-games-launcher"))
                .unwrap_or_else(|_| std::env::var("HOME")
                    .map(|home| PathBuf::from(home).join(".local/share/anime-games-launcher"))
                    .expect("Failed to locate launcher data folder")
                ))
    };

    /// Launcher components folder
    pub static ref COMPONENTS_FOLDER: PathBuf = LAUNCHER_FOLDER.join("components");

    /// Path to the launcher's config file
    pub static ref CONFIG_FILE: PathBuf = LAUNCHER_FOLDER.join("config.json");
}

fn main() -> anyhow::Result<()> {
    adw::init().expect("Libadwaita initialization failed");

    // Register and include resources
    gtk::gio::resources_register_include!("resources.gresource")
        .expect("Failed to register resources");

    // Set icons search path
    gtk::IconTheme::for_display(&gtk::gdk::Display::default().unwrap())
        .add_resource_path(&format!("{APP_RESOURCE_PREFIX}/icons"));

    // Set application's title
    gtk::glib::set_application_name("Anime Games Launcher");
    gtk::glib::set_program_name(Some("Anime Games Launcher"));

    // Set global css
    relm4::set_global_css("
        .game-card {
            transition: transform 0.2s ease;
        }

        .game-card:hover {
            transform: scale(1.03);
        }
    ");

    // Create the app
    let app = RelmApp::new(APP_ID);

    // Show first run window
    app.run::<LoadingApp>(());

    Ok(())
}
