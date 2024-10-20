use relm4::prelude::*;

use tracing_subscriber::prelude::*;
use tracing_subscriber::filter::*;

use clap::Parser;

pub mod consts;
pub mod core;
pub mod config;
pub mod cache;
pub mod packages;
pub mod generations;
pub mod games;

pub mod i18n;
pub mod utils;
pub mod cli;
pub mod ui;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub mod prelude {
    pub use super::consts::*;
    pub use super::utils::*;
    pub use super::cache::*;

    pub use super::core::prelude::*;
    pub use super::packages::prelude::*;
    pub use super::generations::prelude::*;
    pub use super::games::prelude::*;
    pub use super::ui::prelude::*;

    pub use super::config::{
        STARTUP_CONFIG,
        Config,
        self
    };
}

use prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup custom panic handler.
    human_panic::setup_panic!(human_panic::metadata!());

    // Prepare stdout logger.
    let stdout = tracing_subscriber::fmt::layer()
        // .pretty()
        .with_filter({
            if *APP_DEBUG {
                LevelFilter::TRACE
            } else {
                LevelFilter::WARN
            }
        });

    // Prepare debug file logger.
    if let Some(parent) = consts::DEBUG_FILE.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = std::fs::File::create(consts::DEBUG_FILE.as_path())?;

    let debug_log = tracing_subscriber::fmt::layer()
        .pretty()
        .with_ansi(false)
        .with_writer(file);

    // Setup loggers.
    tracing_subscriber::registry()
        .with(stdout)
        .with(debug_log)
        .init();

    // Try to parse and execute CLI command.
    if std::env::args().len() > 1 {
        cli::Cli::parse()
            .execute()
            .await?;
    }

    // Otherwise start GUI app.
    else {
        tracing::info!("Starting application ({APP_VERSION})");

        adw::init().expect("Failed to initializa libadwaita");

        // Register and include resources.
        gtk::gio::resources_register_include!("resources.gresource")
            .expect("Failed to register resources");

        // Set icons search path.
        if let Some(display) = gtk::gdk::Display::default() {
            gtk::IconTheme::for_display(&display)
                .add_resource_path(&format!("{APP_RESOURCE_PREFIX}/icons"));
        }

        // Set application's title.
        gtk::glib::set_application_name("Anime Games Launcher");
        gtk::glib::set_program_name(Some("Anime Games Launcher"));

        // Set global css.
        relm4::set_global_css("
            .warning-action {
                background-color: #BFB04D;
            }
        ");

        // Create the app.
        let app = RelmApp::new(APP_ID);

        // Show loading window.
        app.run_async::<ui::windows::LoadingWindow>(());
    }

    Ok(())
}
