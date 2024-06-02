use std::path::PathBuf;

use relm4::prelude::*;

use tracing_subscriber::prelude::*;
use tracing_subscriber::filter::*;

use clap::Parser;

pub mod i18n;
pub mod utils;
pub mod updater;
pub mod handlers;
pub mod packages;
pub mod config;
pub mod games;
pub mod profiles;
pub mod cli;
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup custom panic handler
    human_panic::setup_panic!(human_panic::metadata!());

    // Prepare stdout logger
    let stdout = tracing_subscriber::fmt::layer()
        // .pretty()
        .with_filter({
            if *APP_DEBUG {
                LevelFilter::TRACE
            } else {
                LevelFilter::WARN
            }
        })
        .with_filter(filter_fn(move |metadata| {
            !metadata.target().contains("rustls")
        }));

    // Prepare debug file logger
    let file = std::fs::File::create(DEBUG_FILE.as_path())?;

    let debug_log = tracing_subscriber::fmt::layer()
        .pretty()
        .with_ansi(false)
        .with_writer(std::sync::Arc::new(file))
        .with_filter(filter_fn(|metadata| {
            !metadata.target().contains("rustls")
        }));

    // Setup loggers
    tracing_subscriber::registry()
        .with(stdout)
        .with(debug_log)
        .init();

    // Try to parse and execute CLI command
    if std::env::args().len() > 1 {
        cli::Cli::parse()
            .execute()
            .await?;
    }

    // Otherwise start GUI app
    else {
        tracing::info!("Starting application ({APP_VERSION})");

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

        // --------------------------------------------------------------------------------------

        let storage = packages::storage::Storage::new("storage")
            .await.unwrap();

        let package = packages::package::Package::fetch("file:///home/observer/projects/new-anime-core/game-integrations/test/jadeite")
            .await.unwrap();

        storage.install(package, |curr, total, name| {
                println!("[{curr}/{total}] Installing {name}");
            })
            .await.unwrap();

        // --------------------------------------------------------------------------------------

        // Create the app
        let app = RelmApp::new(APP_ID);

        // Set global css
        app.set_global_css("
            .warning-action {
                background-color: #BFB04D;
            }
        ");

        // Show loading window
        app.run_async::<ui::windows::prelude::MainApp>(());
    }

    Ok(())
}
