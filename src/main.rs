use std::path::PathBuf;

use relm4::prelude::*;

pub mod i18n;
pub mod utils;
pub mod config;
pub mod games;
pub mod profiles;
pub mod ui;

pub const APP_ID: &str = "moe.launcher.anime-games-launcher";
pub const APP_RESOURCE_PREFIX: &str = "/moe/launcher/anime-games-launcher";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

lazy_static::lazy_static! {
    pub static ref APP_DEBUG: bool = cfg!(debug_assertions) || std::env::args().any(|arg| arg == "--debug");

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

    /// Path to launcher's debug log file
    pub static ref DEBUG_FILE: PathBuf = LAUNCHER_FOLDER.join("debug.log");
}

fn main() -> anyhow::Result<()> {
    // Setup custom panic handler
    human_panic::setup_panic!(human_panic::metadata!());

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
        .warning-action {
            background-color: #BFB04D;
        }
    ");

    // Create the app
    let app = RelmApp::new(APP_ID);

    // Show loading window
    app.run_async::<ui::windows::prelude::MainApp>(());

    Ok(())
}
