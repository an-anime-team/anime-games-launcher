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
use std::time::Duration;

use anyhow::Context;
use toml::{toml, Value as Toml, Table as TomlTable};

use agl_core::export::network::reqwest;
use agl_locale::unic_langid::LanguageIdentifier;

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

    /// Duration of the images cache in seconds. If `0` is set then no cache is
    /// used. Default is `28800` (8 hours).
    ///
    /// `cache.images.duration`
    pub cache_images_duration: Duration,

    /// Duration of the game registries cache in seconds. If `0` is set then no
    /// cache is used. Default is `57600` (16 hours).
    ///
    /// `cache.game_registries.duration`
    pub cache_game_registries_duration: Duration,

    /// Duration of the game manifests cache in seconds. If `0` is set then no
    /// cache is used. Default is `86400` (24 hours).
    ///
    /// `cache.game_manifests.duration`
    pub cache_game_manifests_duration: Duration,

    /// Duration of the game packages cache in seconds. If `0` is set then no
    /// cache is used. Default is `28800` (8 hours).
    ///
    /// `cache.game_packages.duration`
    pub cache_game_packages_duration: Duration,

    /// Duration of the runtime packages allow lists cache in seconds. If `0` is
    /// set then no cache is used. Default is `28800` (8 hours).
    ///
    /// `cache.packages_allow_lists.duration`
    pub cache_packages_allow_lists_duration: Duration,

    /// Proxy mode: `http`, `https` or `all`.
    ///
    /// `general.network.proxy.mode`
    pub general_network_proxy_mode: Option<String>,

    /// URLs to the modules allow lists files.
    ///
    /// `packages.allow_lists`
    pub packages_allow_lists: Vec<String>,

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

    /// Maximal amount of memory in bytes allowed to be consumed by packages
    /// runtime (lua engine). If `0` is set then no limit is applied. Default is
    ///  `1073741824` (1 GiB).
    ///
    /// `runtime.memory_limit`
    pub runtime_memory_limit: usize,

    /// Enable torrent API support. If disabled - no runtime module will be able
    /// to interact with it, and no background service will be started at all.
    ///
    /// `runtime.torrent.enable`
    pub runtime_torrent_enable: bool,

    /// Enable background DHT node.
    ///
    /// `runtime.torrent.dht`
    pub runtime_torrent_enable_dht: bool,

    /// Open BitTorrent protocol port using UPnP.
    ///
    /// `runtime.torrent.upnp`
    pub runtime_torrent_enable_upnp: bool,

    /// List of torrent trackers used by the torrent API.
    ///
    /// `runtime.torrent.trackers`
    pub runtime_torrent_trackers: Vec<String>,

    /// URL to the torrent peers blocklist.
    ///
    /// `runtime.torrent.blocklist_url`
    pub runtime_torrent_blocklist_url: Option<String>,

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

            cache_images_duration: Duration::from_hours(8),
            cache_game_registries_duration: Duration::from_hours(16),
            cache_game_manifests_duration: Duration::from_hours(24),
            cache_game_packages_duration: Duration::from_hours(8),
            cache_packages_allow_lists_duration: Duration::from_hours(8),

            packages_allow_lists: vec![
                String::from("https://raw.githubusercontent.com/an-anime-team/game-integrations/refs/heads/rewrite/packages/allow_list.json")
            ],

            packages_resources_path: DATA_FOLDER.join("packages").join("resources"),
            packages_modules_path: DATA_FOLDER.join("packages").join("modules"),
            packages_persistent_path: DATA_FOLDER.join("packages").join("persistent"),
            packages_temporary_path: DATA_FOLDER.join("packages").join("temporary"),

            runtime_memory_limit: 1024 * 1024 * 1024,

            runtime_torrent_enable: false,
            runtime_torrent_enable_dht: true,
            runtime_torrent_enable_upnp: false,
            runtime_torrent_trackers: vec![
                String::from("udp://tracker.opentrackr.org:1337/announce")
            ],
            runtime_torrent_blocklist_url: Some(String::from("https://raw.githubusercontent.com/Naunter/BT_BlockLists/master/bt_blocklists.gz")),

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

            [cache.images]
            duration = (self.cache_images_duration.as_secs())

            [cache.game_registries]
            duration = (self.cache_game_registries_duration.as_secs())

            [cache.game_manifests]
            duration = (self.cache_game_manifests_duration.as_secs())

            [cache.game_packages]
            duration = (self.cache_game_packages_duration.as_secs())

            [cache.packages_allow_lists]
            duration = (self.cache_packages_allow_lists_duration.as_secs())

            [packages]
            allow_lists = (self.packages_allow_lists.iter().map(|url| url.as_str()).collect::<Vec<_>>())

            [packages.resources]
            path = (self.packages_resources_path.to_string_lossy())

            [packages.modules]
            path = (self.packages_modules_path.to_string_lossy())

            [packages.persistent]
            path = (self.packages_persistent_path.to_string_lossy())

            [packages.temporary]
            path = (self.packages_temporary_path.to_string_lossy())

            [runtime]
            memory_limit = (self.runtime_memory_limit)

            [runtime.torrent]
            enable = (self.runtime_torrent_enable)
            enable_dht = (self.runtime_torrent_enable_dht)
            enable_upnp = (self.runtime_torrent_enable_upnp)
            trackers = (self.runtime_torrent_trackers.iter().map(|url| url.as_str()).collect::<Vec<_>>())
            blocklist_url = (self.runtime_torrent_blocklist_url.as_deref().unwrap_or("none"))

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
                config.general_language = if language == "system" {
                    None
                } else {
                    Some(language.to_string())
                };
            }

            // `general.network.*`
            if let Some(network) = general.get("network") {
                // `general.network.timeout`
                if let Some(timeout) = network.get("timeout").and_then(Toml::as_integer) {
                    config.general_network_timeout = if timeout == 0 {
                        Duration::from_mins(5)
                    } else {
                        Duration::from_millis(timeout as u64)
                    };
                }

                // `general.network.proxy.*`
                if let Some(proxy) = network.get("proxy") {
                    // `general.network.proxy.url`
                    if let Some(url) = proxy.get("url").and_then(Toml::as_str) {
                        config.general_network_proxy_url = if url == "system" {
                            None
                        } else {
                            Some(url.to_string())
                        };
                    }

                    // `general.network.proxy.mode`
                    if let Some(mode) = proxy.get("mode").and_then(Toml::as_str) {
                        config.general_network_proxy_mode = if mode == "system" {
                            None
                        } else {
                            Some(mode.to_string())
                        };
                    }
                }
            }
        }

        // `cache.*`
        if let Some(cache) = value.get("cache") {
            // `cache.images.*`
            if let Some(images) = cache.get("images") {
                // `cache.images.duration`
                if let Some(duration) = images.get("duration").and_then(Toml::as_integer) {
                    config.cache_images_duration = Duration::from_secs(duration as u64);
                }
            }

            // `cache.game_registries.*`
            if let Some(game_registries) = cache.get("game_registries") {
                // `cache.game_registries.duration`
                if let Some(duration) = game_registries.get("duration").and_then(Toml::as_integer) {
                    config.cache_game_registries_duration = Duration::from_secs(duration as u64);
                }
            }

            // `cache.game_manifests.*`
            if let Some(game_manifests) = cache.get("game_manifests") {
                // `cache.game_manifests.duration`
                if let Some(duration) = game_manifests.get("duration").and_then(Toml::as_integer) {
                    config.cache_game_manifests_duration = Duration::from_secs(duration as u64);
                }
            }

            // `cache.game_packages.*`
            if let Some(game_packages) = cache.get("game_packages") {
                // `cache.game_packages.duration`
                if let Some(duration) = game_packages.get("duration").and_then(Toml::as_integer) {
                    config.cache_game_packages_duration = Duration::from_secs(duration as u64);
                }
            }

            // `cache.packages_allow_lists.*`
            if let Some(packages_allow_lists) = cache.get("packages_allow_lists") {
                // `cache.packages_allow_lists.duration`
                if let Some(duration) = packages_allow_lists.get("duration").and_then(Toml::as_integer) {
                    config.cache_packages_allow_lists_duration = Duration::from_secs(duration as u64);
                }
            }
        }

        // `packages.*`
        if let Some(packages) = value.get("packages") {
            // `packages.allow_lists`
            if let Some(allow_lists) = packages.get("allow_lists").and_then(Toml::as_array) {
                config.packages_allow_lists = allow_lists.iter()
                    .flat_map(Toml::as_str)
                    .map(String::from)
                    .collect();
            }

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

        // `runtime.*`
        if let Some(runtime) = value.get("runtime") {
            // `runtime.memory_limit`
            if let Some(memory_limit) = runtime.get("memory_limit").and_then(Toml::as_integer) {
                config.runtime_memory_limit = memory_limit as usize;
            }

            // `runtime.torrent.*`
            if let Some(torrent) = runtime.get("torrent") {
                // `runtime.torrent.enable`
                if let Some(enable) = torrent.get("enable").and_then(Toml::as_bool) {
                    config.runtime_torrent_enable = enable;
                }

                // `runtime.torrent.enable_dht`
                if let Some(enable_dht) = torrent.get("enable_dht").and_then(Toml::as_bool) {
                    config.runtime_torrent_enable_dht = enable_dht;
                }

                // `runtime.torrent.enable_upnp`
                if let Some(enable_upnp) = torrent.get("enable_upnp").and_then(Toml::as_bool) {
                    config.runtime_torrent_enable_upnp = enable_upnp;
                }

                // `runtime.torrent.trackers`
                if let Some(trackers) = torrent.get("trackers").and_then(Toml::as_array) {
                    config.runtime_torrent_trackers = trackers.iter()
                        .flat_map(Toml::as_str)
                        .map(String::from)
                        .collect();
                }

                // `runtime.torrent.blocklist_url`
                if let Some(blocklist_url) = torrent.get("blocklist_url").and_then(Toml::as_str) {
                    config.runtime_torrent_blocklist_url = if blocklist_url == "none" {
                        None
                    } else {
                        Some(blocklist_url.to_string())
                    };
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

    /// Get language identifier specified in the launcher config or, if absent,
    /// from the system settings.
    pub fn language(&self) -> anyhow::Result<LanguageIdentifier> {
        let Some(lang) = &self.general_language else {
            return Ok(agl_locale::SYSTEM_LANG.clone());
        };

        lang.parse::<LanguageIdentifier>()
            .map_err(|err| anyhow::anyhow!(err))
            .context("failed to parse language identifier")
    }

    /// Get `reqwest` crate client builder from the current config file's
    /// network settings.
    pub fn client_builder(&self) -> anyhow::Result<reqwest::ClientBuilder> {
        let mut builder = reqwest::ClientBuilder::new()
            .user_agent(format!("anime-games-launcher/{}", crate::consts::APP_VERSION))
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
