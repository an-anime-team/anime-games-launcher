// SPDX-License-Identifier: GPL-3.0-or-later
//
// anime-games-launcher
// Copyright (C) 2025 - 2026  Nikita Podvirnyi <krypt0nn@vk.com>
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

use std::path::PathBuf;

use agl_games::platform::Platform;

pub const APP_ID: &str = "moe.launcher.anime-games-launcher";
pub const APP_RESOURCE_PREFIX: &str = "/moe/launcher/anime-games-launcher";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

lazy_static::lazy_static! {
    pub static ref APP_DEBUG: bool = cfg!(debug_assertions) || std::env::args().any(|arg| arg == "--agl-debug");

    pub static ref CURRENT_PLATFORM: Platform = {
        let platform = Platform::current();

        platform.expect("failed to detect current system platform")
    };

    /// Path to the current user's home directory.
    pub static ref HOME_DIR: Option<PathBuf> = std::env::var("HOME")
        .or_else(|_| {
            std::env::var("USER")
                .or_else(|_| std::env::var("USERNAME"))
                .map(|username| format!("/home/{username}"))
        })
        .map(PathBuf::from)
        .map_err(std::io::Error::other)
        .and_then(|path| path.canonicalize())
        .ok();

    /// Path to the data folder.
    ///
    /// Default is `$XDG_DATA_HOME/anime-games-launcher`.
    /// Can be overriden by `LAUNCHER_DATA_DIR` variable.
    pub static ref DATA_DIR: PathBuf = {
        if let Ok(path) = std::env::var("LAUNCHER_DATA_DIR")
            .or_else(|_| std::env::var("LAUNCHER_DATA_FOLDER"))
        {
            return PathBuf::from(path);
        }

        let path = std::env::var("XDG_DATA_HOME")
            .map(|data| PathBuf::from(data).join("anime-games-launcher"))
            .or_else(|_| {
                HOME_DIR.as_ref()
                    .map(|dir| dir.join(".local/share/anime-games-launcher"))
                    .ok_or(std::env::VarError::NotPresent)
            })
            .or_else(|_| {
                std::env::current_dir()
                    .map(|current| current.join("data"))
            })
            .expect("failed to locate data directory");

        path.canonicalize().unwrap_or(path)
    };

    /// Path to the config folder.
    ///
    /// Default is `$XDG_CONFIG_HOME/anime-games-launcher`.
    /// Can be overriden by `LAUNCHER_CONFIG_DIR` variable.
    pub static ref CONFIG_DIR: PathBuf = {
        if let Ok(path) = std::env::var("LAUNCHER_CONFIG_DIR")
            .or_else(|_| std::env::var("LAUNCHER_CONFIG_FOLDER"))
        {
            return PathBuf::from(path);
        }

        let path = std::env::var("XDG_CONFIG_HOME")
            .map(|config| PathBuf::from(config).join("anime-games-launcher"))
            .or_else(|_| {
                HOME_DIR.as_ref()
                    .map(|dir| dir.join(".config/anime-games-launcher"))
                    .ok_or(std::env::VarError::NotPresent)
            })
            .or_else(|_| {
                std::env::current_dir()
                    .map(|current| current.join("config"))
            })
            .expect("failed to locate config directory");

        path.canonicalize().unwrap_or(path)
    };

    /// Path to the cache folder.
    ///
    /// Default is `$XDG_CACHE_HOME/anime-games-launcher`.
    /// Can be overriden by `LAUNCHER_CACHE_DIR` variable.
    pub static ref CACHE_DIR: PathBuf = {
        if let Ok(path) = std::env::var("LAUNCHER_CACHE_DIR")
            .or_else(|_| std::env::var("LAUNCHER_CACHE_FOLDER"))
        {
            return PathBuf::from(path);
        }

        let path = std::env::var("XDG_CACHE_HOME")
            .map(|cache| PathBuf::from(cache).join("anime-games-launcher"))
            .or_else(|_| {
                HOME_DIR.as_ref()
                    .map(|dir| dir.join(".cache/anime-games-launcher"))
                    .ok_or(std::env::VarError::NotPresent)
            })
            .or_else(|_| {
                std::env::current_dir()
                    .map(|current| current.join("cache"))
            })
            .expect("failed to locate cache directory");

        path.canonicalize().unwrap_or(path)
    };

    /// Path to the config file.
    ///
    /// Default is `CONFIG_FOLDER/config.toml`.
    pub static ref CONFIG_FILE: PathBuf = CONFIG_DIR.join("config.toml");

    /// Path to the debug log file.
    ///
    /// Default is `DATA_FOLDER/debug.log`.
    pub static ref DEBUG_FILE: PathBuf = DATA_DIR.join("debug.log");

    /// Path to the trace log file.
    ///
    /// Default is `DATA_FOLDER/trace.log`.
    pub static ref TRACE_FILE: PathBuf = DATA_DIR.join("trace.log");
}
