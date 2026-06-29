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

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Context;

use agl_packages::hash::Hash;

use crate::consts::CACHE_FOLDER;

/// Get path to a file with provided cache key.
#[inline]
pub fn get_path(key: impl AsRef<[u8]>) -> PathBuf {
    CACHE_FOLDER.join(Hash::from_bytes(key.as_ref()).to_base32())
}

/// Check if a file with provided path is expired.
///
/// Return `Ok(true)` if the file doesn't exist or file system doesn't support
/// modification and creation timestamps.
pub async fn is_expired(path: &Path, ttl: Duration) -> anyhow::Result<bool> {
    if !path.exists() {
        return Ok(true);
    }

    let metadata = agl_core::tasks::fs::metadata(path).await
        .context("failed to read cached file metadata")?;

    match metadata.modified() {
        Ok(modified_at) => Ok(modified_at.elapsed()? > ttl),
        Err(_) => match metadata.created() {
            Ok(created_at) => Ok(created_at.elapsed()? > ttl),
            Err(_) => Ok(true)
        }
    }
}
