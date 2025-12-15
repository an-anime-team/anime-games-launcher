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

use std::path::PathBuf;
use std::time::Duration;

use toml::{toml, Value as Toml, Table as TomlTable};

use agl_core::export::network::reqwest;

use crate::consts::{DATA_FOLDER, CONFIG_FILE};

lazy_static::lazy_static! {
    static ref STARTUP_CONFIG: Config = get();
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Config {
    /// Language of the launcher. If unset (`system`) - the system one is used.
    ///
    /// `general.language`
    pub general_language: Option<String>,

    /// Timeout for HTTP requests. Default is `5s`.
    ///
    /// `general.network.timeout`
    pub general_network_timeout: Duration,

    /// Proxy URL. If unset (`system`) - environment variable proxy is used
    /// (`http_proxy`, `https_proxy`, `all_proxy`).
    ///
    /// `general.network.proxy.url`
    pub general_network_proxy_url: Option<String>,

    /// Proxy mode: `http`, `https` or `all`.
    ///
    /// `general.network.proxy.mode`
    pub general_network_proxy_mode: Option<String>,

    /// Path to the folder where package resources should be stored.
    ///
    /// `packages.resources.path`
    pub packages_resources_path: PathBuf,

    /// Path to the folder where modules-specific files should be stored.
    ///
    /// These will be kept privately for each module, so they can be used as
    /// secrets storage.
    ///
    /// `packages.modules.path`
    pub packages_modules_path: PathBuf,

    /// Path to the folder where persistent packages files should be stored.
    ///
    /// These will be kept for as long as possible, and the primary usecase is
    /// to allow different modules to use the same files without needing to
    /// download them again.
    ///
    /// `packages.persistent.path`
    pub packages_persistent_path: PathBuf,

    /// Path to the folder where temporary packages files should be stored.
    ///
    /// These will be deleted automatically, depending on launcher
    /// configuration.
    ///
    /// `packages.temporary.path`
    pub packages_temporary_path: PathBuf,

    /// URLs of the game registry files.
    ///
    /// `games.registries`
    pub games_registries: Vec<String>,

    /// Path to the folder where game locks are stored.
    ///
    /// `games.path`
    pub games_path: PathBuf
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general_language: None,
            general_network_timeout: Duration::from_secs(5),
            general_network_proxy_url: None,
            general_network_proxy_mode: None,

            packages_resources_path: DATA_FOLDER.join("packages").join("resources"),
            packages_modules_path: DATA_FOLDER.join("packages").join("modules"),
            packages_persistent_path: DATA_FOLDER.join("packages").join("persistent"),
            packages_temporary_path: DATA_FOLDER.join("packages").join("temporary"),

            games_registries: vec![
                String::from("https://raw.githubusercontent.com/an-anime-team/game-integrations/refs/heads/rewrite/games/registry.json")
            ],
            games_path: DATA_FOLDER.join("games")
        }
    }
}

impl Config {
    pub fn to_toml(&self) -> TomlTable {
        toml! {
            [general]
            language = (self.general_language.as_deref().unwrap_or("system"))

            [general.network]
            timeout = (self.general_network_timeout.as_millis() as u64)

            [general.network.proxy]
            url = (self.general_network_proxy_url.as_deref().unwrap_or("system"))
            mode = (self.general_network_proxy_mode.as_deref().unwrap_or("system"))

            [packages.resources]
            path = (self.packages_resources_path.to_string_lossy())

            [packages.modules]
            path = (self.packages_modules_path.to_string_lossy())

            [packages.persistent]
            path = (self.packages_persistent_path.to_string_lossy())

            [packages.temporary]
            path = (self.packages_temporary_path.to_string_lossy())

            [games]
            registries = (self.games_registries.iter().map(|url| url.as_str()).collect::<Vec<_>>())
            path = (self.games_path.to_string_lossy())
        }
    }

    pub fn from_toml(value: &TomlTable) -> Self {
        let mut config = Self::default();

        // `general.*`
        if let Some(general) = value.get("general") {
            // `general.language`
            if let Some(language) = general.get("language").and_then(Toml::as_str) {
                config.general_language = Some(language.to_string());
            }

            // `general.network.*`
            if let Some(network) = general.get("network") {
                // `general.network.timeout`
                if let Some(timeout) = network.get("timeout").and_then(Toml::as_integer) {
                    config.general_network_timeout = Duration::from_millis(timeout as u64);
                }

                // `general.network.proxy.*`
                if let Some(proxy) = network.get("proxy") {
                    // `general.network.proxy.url`
                    if let Some(url) = proxy.get("url").and_then(Toml::as_str) {
                        config.general_network_proxy_url = Some(url.to_string());
                    }

                    // `general.network.proxy.mode`
                    if let Some(mode) = proxy.get("mode").and_then(Toml::as_str) {
                        config.general_network_proxy_mode = Some(mode.to_string());
                    }
                }
            }
        }

        // `packages.*`
        if let Some(packages) = value.get("packages") {
            // `packages.resources.*`
            if let Some(resources) = packages.get("resources") {
                // `packages.resources.path`
                if let Some(path) = resources.get("path").and_then(Toml::as_str) {
                    config.packages_resources_path = PathBuf::from(path);
                }
            }

            // `packages.modules.*`
            if let Some(modules) = packages.get("modules") {
                // `packages.modules.path`
                if let Some(path) = modules.get("path").and_then(Toml::as_str) {
                    config.packages_modules_path = PathBuf::from(path);
                }
            }

            // `packages.persistent.*`
            if let Some(persistent) = packages.get("persistent") {
                // `packages.persistent.path`
                if let Some(path) = persistent.get("path").and_then(Toml::as_str) {
                    config.packages_persistent_path = PathBuf::from(path);
                }
            }

            // `packages.temporary.*`
            if let Some(temporary) = packages.get("temporary") {
                // `packages.temporary.path`
                if let Some(path) = temporary.get("path").and_then(Toml::as_str) {
                    config.packages_temporary_path = PathBuf::from(path);
                }
            }
        }

        // `games.*`
        if let Some(games) = value.get("games") {
            // `games.registries`
            if let Some(registries) = games.get("registries").and_then(Toml::as_array) {
                config.games_registries = registries.iter()
                    .flat_map(Toml::as_str)
                    .map(String::from)
                    .collect();
            }

            // `games.path`
            if let Some(path) = games.get("path").and_then(Toml::as_str) {
                config.games_path = PathBuf::from(path);
            }
        }

        config
    }

    /// Get `reqwest` crate client builder from the current config file's
    /// network settings.
    pub fn client_builder(&self) -> anyhow::Result<reqwest::ClientBuilder> {
        let mut builder = reqwest::ClientBuilder::new()
            .connect_timeout(self.general_network_timeout);

        if let Some(proxy_url) = &self.general_network_proxy_url {
            let proxy = match self.general_network_proxy_mode.as_deref() {
                Some("http")  => reqwest::Proxy::http(proxy_url)?,
                Some("https") => reqwest::Proxy::https(proxy_url)?,
                Some("all")   => reqwest::Proxy::all(proxy_url)?,

                _ => reqwest::Proxy::all(proxy_url)?
            };

            builder = builder.proxy(proxy);
        }

        Ok(builder)
    }
}

/// Get startup config. It is loaded when the application is started and is not
/// changed afterwards. Use `config::get()` to get the actual config.
pub fn startup() -> &'static Config {
    &STARTUP_CONFIG
}

/// Read configuration from the file.
pub fn get() -> Config {
    std::fs::read(CONFIG_FILE.as_path()).ok()
        .and_then(|config| toml::from_slice::<TomlTable>(&config).ok())
        .map(|config| Config::from_toml(&config))
        .unwrap_or_default()
}

/// Update configuration file.
pub fn set(config: &Config) -> anyhow::Result<()> {
    std::fs::write(
        CONFIG_FILE.as_path(),
        toml::to_string_pretty(&config.to_toml())?
    )?;

    Ok(())
}
