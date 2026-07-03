// SPDX-License-Identifier: GPL-3.0-or-later
//
// anime-games-launcher
// Copyright (C) 2025 - 2026  Nikita Podvirnyi <krypt0nn@dawn.wine>
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
use agl_core::tasks;
use agl_locale::unic_langid::LanguageIdentifier;

use crate::consts::{HOME_DIR, DATA_DIR, CONFIG_FILE};

lazy_static::lazy_static! {
    static ref STARTUP_CONFIG: Config = tasks::block_on(get());
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
    /// `general.network.proxy`
    pub general_network_proxy: Option<String>,

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

    /// Duration of the runtime modules scopes lists cache in seconds. If `0`
    /// is set then no cache is used. Default is `28800` (8 hours).
    ///
    /// `cache.modules_scopes_lists.duration`
    pub cache_modules_scopes_lists_duration: Duration,

    /// Duration since cache entry creation after which it will be automatically
    /// removed.
    ///
    /// `cache.collect_garbage_after`
    pub cache_collect_garbage_after: Duration,

    /// URLs to the modules scopes lists files.
    ///
    /// `packages.scopes_lists`
    pub packages_scopes_lists: Vec<String>,

    /// Path to the directory where package resources should be stored.
    ///
    /// `packages.resources.path`
    pub packages_resources_path: PathBuf,

    /// Remove resources that are not used by any game package.
    ///
    /// `packages.resources.collect_garbage`
    pub packages_resources_collect_garbage: bool,

    /// Path to the directory where modules-specific files should be stored.
    ///
    /// These will be kept privately for each module, so they can be used as
    /// secrets storage.
    ///
    /// `packages.modules.path`
    pub packages_modules_path: PathBuf,

    /// Remove modules directories if the module is not used by any package.
    ///
    /// `packages.modules.collect_garbage`
    pub packages_modules_collect_garbage: bool,

    /// Path to the directory where persistent packages files should be stored.
    ///
    /// These will be kept for as long as possible, and the primary usecase is
    /// to allow different modules to use the same files without needing to
    /// download them again.
    ///
    /// `packages.persistent.path`
    pub packages_persistent_path: PathBuf,

    /// Path to the directory where temporary packages files should be stored.
    ///
    /// These will be deleted automatically, depending on launcher
    /// configuration.
    ///
    /// `packages.temporary.path`
    pub packages_temporary_path: PathBuf,

    /// Clear temporary directory on launcher startup.
    ///
    /// `packages.temporary.collect_garbage`
    pub packages_temporary_collect_garbage: bool,

    /// Maximal amount of memory in bytes allowed to be consumed by packages
    /// runtime (lua engine). If `0` is set then no limit is applied. Default is
    ///  `1073741824` (1 GiB).
    ///
    /// `runtime.memory_limit`
    pub runtime_memory_limit: usize,

    /// List of paths that runtime modules will be forbidden to access.
    ///
    /// `runtime.private_paths`
    pub runtime_private_paths: Vec<PathBuf>,

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

    /// Path to the secrets API database file.
    ///
    /// `runtime.secrets.path`
    pub runtime_secrets_path: PathBuf,

    /// URLs of the game registry files.
    ///
    /// `games.registries`
    pub games_registries: Vec<String>,

    /// Path to the directory where game locks are stored.
    ///
    /// `games.path`
    pub games_path: PathBuf
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general_language: None,
            general_network_timeout: Duration::from_secs(5),
            general_network_proxy: None,

            cache_images_duration: Duration::from_hours(8),
            cache_game_registries_duration: Duration::from_hours(16),
            cache_game_manifests_duration: Duration::from_hours(24),
            cache_game_packages_duration: Duration::from_hours(8),
            cache_modules_scopes_lists_duration: Duration::from_hours(8),
            cache_collect_garbage_after: Duration::from_hours(72),

            packages_scopes_lists: vec![
                String::from("https://raw.githubusercontent.com/an-anime-team/game-integrations/refs/heads/master/scopes.json")
            ],

            packages_resources_path: DATA_DIR.join("packages").join("resources"),
            packages_resources_collect_garbage: true,

            packages_modules_path: DATA_DIR.join("packages").join("modules"),
            packages_modules_collect_garbage: true,

            packages_persistent_path: DATA_DIR.join("packages").join("persistent"),

            packages_temporary_path: DATA_DIR.join("packages").join("temporary"),
            packages_temporary_collect_garbage: true,

            runtime_memory_limit: 1024 * 1024 * 1024,

            runtime_private_paths: {
                let mut paths = vec![
                    // Linux accounts.
                    PathBuf::from("/etc/shadow"),
                    PathBuf::from("/etc/gshadow"),
                    PathBuf::from("/etc/passwd"),
                    PathBuf::from("/etc/group"),
                    PathBuf::from("/etc/sudoers"),
                    PathBuf::from("/etc/static/sudoers"),
                    PathBuf::from("/etc/.pwd.lock"),

                    // BSD accounts.
                    //
                    // https://man.freebsd.org/cgi/man.cgi?query=master.passwd&sektion=5&n=1
                    PathBuf::from("/etc/master.passwd"),

                    // Linux SSH and GnuPG info.
                    PathBuf::from("/etc/ssh"),
                    PathBuf::from("/etc/gnupg"),
                    PathBuf::from("/etc/static/ssh"),
                    PathBuf::from("/etc/static/gnupg"),

                    // Linux LUKS encryption data.
                    PathBuf::from("/etc/crypttab"),

                    // Linux WIFI credentials.
                    PathBuf::from("/etc/wpa_supplicant"),
                    PathBuf::from("/etc/static/wpa_supplicant"),

                    // Linux boot partition.
                    PathBuf::from("/boot")
                ];

                // SSH private and public keys.
                if let Some(path) = HOME_DIR.as_ref() {
                    paths.push(path.join(".ssh"));
                }

                // GnuPG private and public keys.
                if let Some(path) = HOME_DIR.as_ref() {
                    paths.push(path.join(".gnupg"));
                }

                paths
            },

            runtime_torrent_enable: false,
            runtime_torrent_enable_dht: true,
            runtime_torrent_enable_upnp: false,
            runtime_torrent_trackers: vec![],
            runtime_torrent_blocklist_url: Some(String::from("https://raw.githubusercontent.com/Naunter/BT_BlockLists/master/bt_blocklists.gz")),

            runtime_secrets_path: DATA_DIR.join("secrets.db"),

            games_registries: vec![
                String::from("https://raw.githubusercontent.com/an-anime-team/game-integrations/refs/heads/master/games/registry.json")
            ],
            games_path: DATA_DIR.join("games")
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
            proxy = (self.general_network_proxy.as_deref().unwrap_or("system"))

            [cache]
            collect_garbage_after = (self.cache_collect_garbage_after.as_secs())

            [cache.images]
            duration = (self.cache_images_duration.as_secs())

            [cache.game_registries]
            duration = (self.cache_game_registries_duration.as_secs())

            [cache.game_manifests]
            duration = (self.cache_game_manifests_duration.as_secs())

            [cache.game_packages]
            duration = (self.cache_game_packages_duration.as_secs())

            [cache.modules_scopes_lists]
            duration = (self.cache_modules_scopes_lists_duration.as_secs())

            [packages]
            scopes_lists = (self.packages_scopes_lists.iter().map(|url| url.as_str()).collect::<Vec<_>>())

            [packages.resources]
            path = (self.packages_resources_path.to_string_lossy())
            collect_garbage = (self.packages_resources_collect_garbage)

            [packages.modules]
            path = (self.packages_modules_path.to_string_lossy())
            collect_garbage = (self.packages_modules_collect_garbage)

            [packages.persistent]
            path = (self.packages_persistent_path.to_string_lossy())

            [packages.temporary]
            path = (self.packages_temporary_path.to_string_lossy())
            collect_garbage = (self.packages_temporary_collect_garbage)

            [runtime]
            memory_limit = (self.runtime_memory_limit)
            private_paths = (self.runtime_private_paths.iter()
                .map(|path| path.to_string_lossy().to_string())
                .collect::<Vec<_>>())

            [runtime.torrent]
            enable = (self.runtime_torrent_enable)
            enable_dht = (self.runtime_torrent_enable_dht)
            enable_upnp = (self.runtime_torrent_enable_upnp)
            trackers = (self.runtime_torrent_trackers.iter().map(|url| url.as_str()).collect::<Vec<_>>())
            blocklist_url = (self.runtime_torrent_blocklist_url.as_deref().unwrap_or("none"))

            [runtime.secrets]
            path = (self.runtime_secrets_path.to_string_lossy())

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

                // `general.network.proxy`
                if let Some(proxy) = network.get("proxy") {
                    // New syntax (`general.network.proxy`)
                    if let Some(url) = proxy.as_str() {
                        config.general_network_proxy = if url == "system" {
                            None
                        } else {
                            Some(url.to_string())
                        };
                    }

                    // Old syntax (`general.network.proxy.url`)
                    else if let Some(url) = proxy.get("url").and_then(Toml::as_str) {
                        config.general_network_proxy = if url == "system" {
                            None
                        } else {
                            Some(url.to_string())
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

            // New syntax (`cache.modules_scopes_lists.*`)
            if let Some(modules_scopes_lists) = cache.get("modules_scopes_lists") {
                // `cache.modules_scopes_lists.duration`
                if let Some(duration) = modules_scopes_lists.get("duration").and_then(Toml::as_integer) {
                    config.cache_modules_scopes_lists_duration = Duration::from_secs(duration as u64);
                }
            }

            // Old syntax (`cache.packages_allow_lists.*`)
            else if let Some(packages_allow_lists) = cache.get("packages_allow_lists") {
                // `cache.packages_allow_lists.duration`
                if let Some(duration) = packages_allow_lists.get("duration").and_then(Toml::as_integer) {
                    config.cache_modules_scopes_lists_duration = Duration::from_secs(duration as u64);
                }
            }

            // `cache.collect_garbage_after`
            if let Some(collect_garbage_after) = cache.get("collect_garbage_after").and_then(Toml::as_integer) {
                config.cache_collect_garbage_after = Duration::from_secs(collect_garbage_after as u64);
            }
        }

        // `packages.*`
        if let Some(packages) = value.get("packages") {
            // New syntax (`packages.scopes_lists`)
            if let Some(scopes_list) = packages.get("scopes_lists").and_then(Toml::as_array) {
                config.packages_scopes_lists = scopes_list.iter()
                    .flat_map(Toml::as_str)
                    .map(String::from)
                    .collect();
            }

            // Old syntax (`packages.allow_lists`)
            else if let Some(allow_lists) = packages.get("allow_lists").and_then(Toml::as_array) {
                config.packages_scopes_lists = allow_lists.iter()
                    .flat_map(Toml::as_str)
                    .map(|url| {
                        if url == "https://raw.githubusercontent.com/an-anime-team/game-integrations/refs/heads/master/packages/allow_list.json" {
                            String::from("https://raw.githubusercontent.com/an-anime-team/game-integrations/refs/heads/master/scopes.json")
                        } else {
                            String::from(url)
                        }
                    })
                    .collect();
            }

            // `packages.resources.*`
            if let Some(resources) = packages.get("resources") {
                // `packages.resources.path`
                if let Some(path) = resources.get("path").and_then(Toml::as_str) {
                    config.packages_resources_path = PathBuf::from(path);
                }

                // `packages.resources.collect_garbage`
                if let Some(value) = resources.get("collect_garbage").and_then(Toml::as_bool) {
                    config.packages_resources_collect_garbage = value;
                }
            }

            // `packages.modules.*`
            if let Some(modules) = packages.get("modules") {
                // `packages.modules.path`
                if let Some(path) = modules.get("path").and_then(Toml::as_str) {
                    config.packages_modules_path = PathBuf::from(path);
                }

                // `packages.modules.collect_garbage`
                if let Some(value) = modules.get("collect_garbage").and_then(Toml::as_bool) {
                    config.packages_modules_collect_garbage = value;
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

                // `packages.temporary.collect_garbage`
                if let Some(value) = temporary.get("collect_garbage").and_then(Toml::as_bool) {
                    config.packages_temporary_collect_garbage = value;
                }
            }
        }

        // `runtime.*`
        if let Some(runtime) = value.get("runtime") {
            // `runtime.memory_limit`
            if let Some(memory_limit) = runtime.get("memory_limit").and_then(Toml::as_integer) {
                config.runtime_memory_limit = memory_limit as usize;
            }

            // `runtime.private_paths`
            if let Some(private_paths) = runtime.get("private_paths").and_then(Toml::as_array) {
                config.runtime_private_paths = private_paths.iter()
                    .flat_map(Toml::as_str)
                    .map(PathBuf::from)
                    .collect();
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

            // `runtime.secrets.*`
            if let Some(secrets) = runtime.get("secrets") {
                // `runtime.secrets.path`
                if let Some(path) = secrets.get("path").and_then(Toml::as_str) {
                    config.runtime_secrets_path = PathBuf::from(path);
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
            .user_agent(format!("anime-games-launcher/v{}", crate::consts::APP_VERSION))
            .connect_timeout(self.general_network_timeout);

        if let Some(proxy_url) = &self.general_network_proxy {
            builder = builder.proxy(reqwest::Proxy::all(proxy_url)?);
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
pub async fn get() -> Config {
    tasks::fs::read(CONFIG_FILE.as_path()).await.ok()
        .and_then(|config| toml::from_slice::<TomlTable>(&config).ok())
        .map(|config| Config::from_toml(&config))
        .unwrap_or_default()
}

/// Update configuration file.
pub async fn set(config: &Config) -> anyhow::Result<()> {
    tasks::fs::write(
        CONFIG_FILE.as_path(),
        toml::to_string_pretty(&config.to_toml())?
    ).await?;

    Ok(())
}
