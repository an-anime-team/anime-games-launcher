use std::path::PathBuf;

use crate::prelude::*;

pub const APP_ID: &str = "moe.launcher.anime-games-launcher";
pub const APP_RESOURCE_PREFIX: &str = "/moe/launcher/anime-games-launcher";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

lazy_static::lazy_static! {
    pub static ref APP_DEBUG: bool = cfg!(debug_assertions) || std::env::args().any(|arg| arg == "--debug");

    pub static ref CURRENT_PLATFORM: Option<TargetPlatform> = {
        let platform = TargetPlatform::current();

        tracing::info!("Current platform: {:?}", platform.map(|platform| platform.to_string()));

        platform
    };

    /// Path to the data folder.
    ///
    /// Default is `$XDG_DATA_HOME/anime-games-launcher`.
    pub static ref DATA_FOLDER: PathBuf = std::env::var("XDG_DATA_HOME")
        .map(|data| format!("{data}/anime-games-launcher"))
        .or_else(|_| {
            std::env::var("HOME")
                .map(|home| {
                    format!("{home}/.local/share/anime-games-launcher")
                })
        })
        .or_else(|_| {
            std::env::var("USERNAME")
                .map(|username| {
                    format!("/home/{username}/.local/share/anime-games-launcher")
                })
        })
        .map(PathBuf::from)
        .or_else(|_| {
            std::env::current_dir()
                .map(|current| current.join("data"))
        })
        .expect("Couldn't locate data directory");

    /// Path to the config folder.
    ///
    /// Default is `$XDG_CONFIG_HOME/anime-games-launcher`.
    pub static ref CONFIG_FOLDER: PathBuf = std::env::var("XDG_CONFIG_HOME")
        .map(|config| format!("{config}/anime-games-launcher"))
        .or_else(|_| {
            std::env::var("HOME")
                .map(|home| {
                    format!("{home}/.config/anime-games-launcher")
                })
        })
        .or_else(|_| {
            std::env::var("USERNAME")
                .map(|username| {
                    format!("/home/{username}/.config/anime-games-launcher")
                })
        })
        .map(PathBuf::from)
        .or_else(|_| {
            std::env::current_dir()
                .map(|current| current.join("config"))
        })
        .expect("Couldn't locate config directory");

    /// Path to the cache folder.
    ///
    /// Default is `$XDG_CACHE_HOME/anime-games-launcher`.
    pub static ref CACHE_FOLDER: PathBuf = std::env::var("XDG_CACHE_HOME")
        .map(|cache| format!("{cache}/anime-games-launcher"))
        .or_else(|_| {
            std::env::var("HOME")
                .map(|home| {
                    format!("{home}/.cache/anime-games-launcher")
                })
        })
        .or_else(|_| {
            std::env::var("USERNAME")
                .map(|username| {
                    format!("/home/{username}/.cache/anime-games-launcher")
                })
        })
        .map(PathBuf::from)
        .or_else(|_| {
            std::env::current_dir()
                .map(|current| current.join("cache"))
        })
        .expect("Couldn't locate cache directory");

    /// Path to the config file.
    ///
    /// Default is `CONFIG_FOLDER/config.json`.
    pub static ref CONFIG_FILE: PathBuf = CONFIG_FOLDER.join("config.json");

    /// Path to the debug log file.
    ///
    /// Default is `DATA_FOLDER/debug.log`.
    pub static ref DEBUG_FILE: PathBuf = DATA_FOLDER.join("debug.log");
}
