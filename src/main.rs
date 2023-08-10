use std::path::PathBuf;

use relm4::prelude::*;

pub mod ui;
pub mod config;

use ui::windows::main::MainApp;

pub const APP_ID: &str = "moe.launcher.anime-games-launcher";

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

    /// Path to the launcher's config file
    pub static ref CONFIG_FILE: PathBuf = LAUNCHER_FOLDER.join("config.json");
}

fn main() {
    adw::init().expect("Libadwaita initialization failed");

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

        .game-details--genshin {
            background: radial-gradient(#f4cc99, #3b4b7c);
        }

        .game-details--honkai {
            background: radial-gradient(#f8c2d0, #4078c5);
        }

        .game-details--star-rail {
            background: radial-gradient(#c2fafb, #1c1328);
        }

        .game-details--pgr {
            background: radial-gradient(#534232, #c6b297);
        }
    ");

    // Create the app
    let app = RelmApp::new(APP_ID);

    // Show first run window
    app.run::<MainApp>(());
}
