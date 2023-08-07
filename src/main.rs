use relm4::prelude::*;

pub mod windows;
pub mod components;

use windows::main::MainApp;

pub const APP_ID: &str = "moe.launcher.anime-games-launcher";

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
    ");

    // Create the app
    let app = RelmApp::new(APP_ID);

    // Show first run window
    app.run::<MainApp>(());
}
