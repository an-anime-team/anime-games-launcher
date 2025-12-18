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

/// Get path to a file with provided cache key.
#[inline]
pub fn get_path(key: impl AsRef<[u8]>) -> PathBuf {
    CACHE_FOLDER.join(Hash::from_bytes(key.as_ref()).to_base32())
}

/// Check if a file with provided path is expired. Return `Ok(true)` if such
/// file doesn't exist.
pub fn is_expired(
    path: impl AsRef<Path>,
    ttl: Duration
) -> anyhow::Result<bool> {
    let path = path.as_ref();

    if !path.exists() {
        return Ok(true);
    }

    let metadata = path.metadata()
        .context("failed to read cached file metadata")?;

    let created_at = metadata.created()
        .context("failed to read cached file creation time")?;

    Ok(created_at.elapsed()? > ttl)
}
