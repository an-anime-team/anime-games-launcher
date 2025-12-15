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

use serde_json::{json, Value as Json};

use agl_games::manifest::GameManifest;
use agl_packages::lock::Lock as PackageLock;

/// Lock file for a game package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameLock {
    /// Manifest of the locked game.
    pub game: GameManifest,

    /// Lock of the game package.
    pub package: PackageLock
}

impl GameLock {
    pub fn to_json(&self) -> Json {
        json!({
            "game": self.game.to_json(),
            "package": self.package.to_json()
        })
    }

    pub fn from_json(value: &Json) -> anyhow::Result<Self> {
        Ok(Self {
            game: value.get("game")
                .ok_or_else(|| anyhow::anyhow!("missing 'game' field in game lock"))
                .and_then(|game| {
                    GameManifest::from_json(game)
                        .map_err(|err| anyhow::anyhow!(err))
                })?,

            package: value.get("package")
                .ok_or_else(|| anyhow::anyhow!("missing 'package' field in game lock"))
                .and_then(|game| {
                    PackageLock::from_json(game)
                        .ok_or_else(|| anyhow::anyhow!("invalid 'package' field value in game lock"))
                })?
        })
    }
}
