// SPDX-License-Identifier: GPL-3.0-or-later
//
// anime-games-launcher
// Copyright (C) 2025  Nikita Podvirnyi <krypt0nn@vk.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::fs::File;

use relm4::prelude::*;

use tracing_subscriber::prelude::*;
use tracing_subscriber::filter::*;

pub mod consts;
pub mod utils;
pub mod config;
pub mod cache;
pub mod games;
pub mod ui;

lazy_static::lazy_static! {
    pub static ref STARTUP_LANG: agl_locale::unic_langid::LanguageIdentifier = {
        config::startup()
            .language()
            .unwrap_or_else(|_| agl_locale::SYSTEM_LANG.clone())
    };
}

/// Custom `i18n` macro based on the `agl-locale`'s one which respects
/// launcher's config file.
///
/// - `i18n("string_key") -> Option<String>`
/// - `i18n("string_key", { arg => "value", ... }) -> Option<String>`
#[macro_export]
macro_rules! i18n {
    ($key:expr) => {
        agl_locale::i18n!($crate::STARTUP_LANG.as_ref(), $key)
    };

    ($key:expr, {$( $arg_key:expr => $arg_value:expr $(,)* )+}) => {
        agl_locale::i18n!(
            $crate::STARTUP_LANG.as_ref(),
            $key,
            {$( $arg_key => $arg_value )+}
        )
    };
}


#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> anyhow::Result<()> {
    // Setup custom panic handler.
    if !*consts::APP_DEBUG {
        human_panic::setup_panic!(human_panic::metadata!());
    }

    // Include translations file.
    agl_locale::include_i18n!("../assets/locales/interface.toml");
    agl_locale::include_i18n!("../assets/locales/game_tags.toml");
    agl_locale::include_i18n!("../assets/locales/error_messages.toml");

    // Prepare stdout logger.
    let stdout_log = tracing_subscriber::fmt::layer()
        // .pretty()
        .with_filter({
            filter_fn(|metadata| {
                if metadata.target().starts_with("anime_games_launcher") {
                    return true;
                }

                if metadata.target().starts_with("agl_runtime")
                    || metadata.target().starts_with("agl_games")
                {
                    return metadata.level() != &tracing::Level::TRACE;
                }

                false
            })
        })
        .with_filter({
            if *consts::APP_DEBUG {
                LevelFilter::TRACE
            } else {
                LevelFilter::WARN
            }
        });

    // Prepare debug files logger.
    if let Some(parent) = consts::DEBUG_FILE.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if let Some(parent) = consts::TRACE_FILE.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let debug_log = tracing_subscriber::fmt::layer()
        .with_writer(File::create(consts::DEBUG_FILE.as_path())?)
        .with_ansi(false)
        .with_filter({
            filter_fn(|metadata| {
                metadata.target().starts_with("anime_games_launcher")
                    || metadata.target().starts_with("agl_")
            })
        });

    let trace_log = tracing_subscriber::fmt::layer()
        .with_writer(File::create(consts::TRACE_FILE.as_path())?)
        .with_ansi(false)
        .with_filter({
            filter_fn(|metadata| {
                if metadata.target().starts_with("librqbit") {
                    return metadata.level() != &tracing::Level::TRACE;
                }

                !metadata.target().starts_with("h2") &&
                !metadata.target().starts_with("hyper_util")
            })
        });

    // Setup loggers.
    tracing_subscriber::registry()
        .with(stdout_log)
        .with(debug_log)
        .with(trace_log)
        .init();

    // Initialize libadwaita and GTK.
    tracing::info!(
        platform = consts::CURRENT_PLATFORM.to_string(),
        launcher_version = consts::APP_VERSION,
        core_version = agl_core::VERSION,
        packages_version = agl_packages::VERSION,
        runtime_version = agl_runtime::VERSION,
        games_version = agl_games::VERSION,
        "starting application"
    );

    adw::init().expect("failed to initializa libadwaita");

    // Register and include resources.
    gtk::gio::resources_register_include!("resources.gresource")
        .expect("failed to register resources");

    // Set icons search path.
    if let Some(display) = gtk::gdk::Display::default() {
        let theme = gtk::IconTheme::for_display(&display);

        theme.add_resource_path(&format!("{}/icons", consts::APP_RESOURCE_PREFIX));
    }

    // Set application's title.
    gtk::glib::set_application_name("Anime Games Launcher");
    gtk::glib::set_program_name(Some("Anime Games Launcher"));

    // Set global css.
    relm4::set_global_css("
        .warning-action {
            background-color: #BFB04D;
        }

        flowboxchild:hover {
            background: unset;
        }
    ");

    // Check for WINE_CANONICAL_HOLE variable.
    if let Ok(value) = std::env::var("WINE_CANONICAL_HOLE") && !value.is_empty() {
        tracing::warn!("WINE_CANONICAL_HOLE={value} is not supported, please contact <https://github.com/NelloKudo> to fix it");
    }

    // Create the app.
    let app = RelmApp::new(consts::APP_ID);

    // Show loading window.
    app.run_async::<ui::windows::main_window::MainWindow>(());

    Ok(())
}
