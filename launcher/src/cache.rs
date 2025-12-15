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

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Context;

use agl_packages::hash::Hash;

use crate::consts::CACHE_FOLDER;

/// A general files cache storage.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FilesCache {
    /// Path to the folder where cached files will be stored.
    path: PathBuf,

    /// Amount of time before file cache is considered expired.
    cache_time: Duration
}

impl Default for FilesCache {
    #[inline]
    fn default() -> Self {
        Self::open(CACHE_FOLDER.as_path())
    }
}

impl FilesCache {
    /// Open files cache in provided folder.
    pub fn open(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            cache_time: Duration::from_secs(8 * 3600) // 8 hours
        }
    }

    /// Set cache expiry time.
    #[inline]
    pub fn with_cache_time(mut self, cache_time: Duration) -> Self {
        self.cache_time = cache_time;

        self
    }

    /// Get path to a file with provided cache key.
    #[inline]
    pub fn get_path(&self, key: impl AsRef<[u8]>) -> PathBuf {
        self.path.join(Hash::from_bytes(key.as_ref()).to_base32())
    }

    /// Check if a file with provided cache key is expired. Return `Ok(false)`
    /// if such file doesn't exist.
    pub fn is_expired(&self, key: impl AsRef<[u8]>) -> anyhow::Result<bool> {
        let path = self.get_path(key);

        if !path.exists() {
            return Ok(false);
        }

        let metadata = path.metadata()
            .context("failed to read cached file metadata")?;

        let created_at = metadata.created()
            .context("failed to read cached file creation time")?;

        Ok(created_at.elapsed()? > self.cache_time)
    }

    /// Copy file with provided path to the cache folder under specified key.
    /// The original file is kept unchanged.
    pub fn cache_file(
        &self,
        key: impl AsRef<[u8]>,
        path: impl AsRef<Path>
    ) -> anyhow::Result<()> {
        std::fs::copy(path.as_ref(), self.get_path(key))
            .context("failed to copy file to the cache folder")?;

        Ok(())
    }
}
