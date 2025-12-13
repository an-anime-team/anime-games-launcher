// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-games
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

use serde_json::Value as Json;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GameManifestDeserializeError {
    #[error(transparent)]
    Game(#[from] GameInfoDeserializeError),

    #[error(transparent)]
    Package(#[from] PackageInfoDeserializeError),

    #[error("game info is not provided")]
    MissingGameInfo,

    #[error("package info is not provided")]
    MissingPackageInfo
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameManifest {
    pub game: GameInfo,
    pub package: PackageInfo
}

impl GameManifest {
    pub fn from_json(
        value: &Json
    ) -> Result<Self, GameManifestDeserializeError> {
        Ok(Self {
            game: value.get("game")
                .map(GameInfo::from_json)
                .ok_or(GameManifestDeserializeError::MissingGameInfo)??,

            package: value.get("package")
                .map(PackageInfo::from_json)
                .ok_or(GameManifestDeserializeError::MissingPackageInfo)??
        })
    }
}
