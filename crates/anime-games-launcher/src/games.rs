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

use anyhow::Context;
use serde_json::{json, Value as Json};

use agl_core::export::hashes::seahash;
use agl_core::tasks;
use agl_core::network::downloader::{Downloader, DownloadOptions};
use agl_packages::hash::Hash;
use agl_packages::storage::Storage;
use agl_packages::lock::Lock as PackageLock;
use agl_runtime::module::ModuleScope;
use agl_games::manifest::GameManifest;

use crate::config;
use crate::cache;

/// Get sanitized game name from optional manifest's `name` field and manifest
/// downloading URL.
pub fn get_name(name: Option<&str>, manifest_url: &str) -> String {
    match name {
        Some(name) => {
            let hash = seahash::hash(name.as_bytes());

            let name = name.chars()
                .map(|char| {
                    if char.is_ascii_alphanumeric() {
                        char
                    } else {
                        '_'
                    }
                })
                .collect::<String>();

            let name = name.trim_matches('_')
                .replace("__", "_");

            if name.is_empty() {
                format!("{hash:x}")
            } else {
                format!("{hash:x}-{name}")
            }
        }

        None => Hash::digitize(manifest_url.as_bytes()).to_base32()
    }
}

/// Lock file for a game package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameLock {
    /// URL to the game manifest.
    pub url: String,

    /// Manifest of the locked game.
    pub manifest: GameManifest,

    /// Lock of the game package.
    pub lock: PackageLock,

    /// Optional sandbox scope applied to all the runtime modules used by the
    /// game integration.
    pub scope: Option<ModuleScope>
}

impl GameLock {
    pub fn to_json(&self) -> Json {
        json!({
            "format": 1,
            "url": self.url,
            "manifest": self.manifest.to_json(),
            "lock": self.lock.to_json(),
            "scope": self.scope.as_ref()
                .map(ModuleScope::to_json)
        })
    }

    pub fn from_json(value: &Json) -> anyhow::Result<Self> {
        if value.get("format").and_then(Json::as_u64) != Some(1)
            && value.get("version").and_then(Json::as_u64) != Some(1)
        {
            anyhow::bail!("unsupported game lock file format");
        }

        Ok(Self {
            url: value.get("url")
                .and_then(Json::as_str)
                .map(String::from)
                .ok_or_else(|| anyhow::anyhow!("missing 'url' field in game lock"))?,

            manifest: value.get("manifest")
                .ok_or_else(|| anyhow::anyhow!("missing 'manifest' field in game lock"))
                .and_then(|game| {
                    GameManifest::from_json(game)
                        .map_err(|err| anyhow::anyhow!(err))
                })?,

            lock: value.get("lock")
                .ok_or_else(|| anyhow::anyhow!("missing 'lock' field in game lock"))
                .and_then(|game| {
                    PackageLock::from_json(game)
                        .ok_or_else(|| anyhow::anyhow!("invalid 'lock' field value in game lock"))
                })?,

            scope: value.get("scope")
                .and_then(|scope| {
                    if scope.is_null() {
                        None
                    } else {
                        Some(ModuleScope::from_json(scope))
                    }
                })
        })
    }

    /// Get sanitized game name derived from either game manifest's `name` field
    /// or the manifest's download URL.
    #[inline]
    pub fn name(&self) -> String {
        get_name(self.manifest.game.name.as_deref(), self.url.as_str())
    }

    /// Download game package and manifest files and lock them.
    pub async fn download(
        manifest_url: impl ToString,
        storage: &Storage
    ) -> anyhow::Result<Self> {
        // Prepare files downloader.
        let config = config::get().await;

        let client = config.client_builder()?
            .build()?;

        let downloader = Downloader::from_client(client);

        // Check if manifest is already downloaded or download it.
        let manifest_url = manifest_url.to_string();

        let manifest_path = cache::get_path(&manifest_url);

        if cache::is_expired(
            &manifest_path,
            config.cache_game_manifests_duration
        ).await? {
            let task = downloader.download_with_options(
                &manifest_url,
                &manifest_path,
                DownloadOptions {
                    continue_download: false,
                    on_update: None,
                    on_finish: None
                }
            );

            task.wait().await
                .context("failed to download game manifest")?;
        }

        // Read manifest file.
        let manifest = tasks::fs::read(&manifest_path).await?;

        let manifest = serde_json::from_slice::<Json>(&manifest)
            .context("failed to decode json file with game manifest")?;

        let manifest = GameManifest::from_json(&manifest)
            .context("failed to deserialize game manifest")?;

        // Install game package.
        let result = storage.install_packages(&downloader, [
            manifest.package.url.clone()
        ]).await;

        let lock = match result {
            Ok(lock) => lock,
            Err(err) => {
                return Err(anyhow::anyhow!(err)
                    .context("failed to install game package"));
            }
        };

        Ok(Self {
            url: manifest_url,
            manifest,
            lock,
            scope: None
        })
    }
}
