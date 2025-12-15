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

use serde_json::{json, Value as Json};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GamesRegistryManifestDeserializeError {
    #[error("games registry missing games")]
    MissingGames,

    #[error("game manifest url is not specified")]
    MissingGameManifestUrl
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GamesRegistryManifest {
    pub games: Vec<GameManifestReference>
}

impl GamesRegistryManifest {
    pub fn to_json(&self) -> Json {
        json!({
            "games": self.games.iter()
                .map(|game| game.to_json())
                .collect::<Vec<_>>()
        })
    }

    pub fn from_json(
        value: &Json
    ) -> Result<Self, GamesRegistryManifestDeserializeError> {
        Ok(Self {
            games: value.get("games")
                .and_then(Json::as_array)
                .map(|games| {
                    games.iter()
                        .map(GameManifestReference::from_json)
                        .collect::<Result<Vec<_>, _>>()
                })
                .ok_or(GamesRegistryManifestDeserializeError::MissingGames)??
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameManifestReference {
    /// URL to the game manifest.
    pub url: String,

    /// Whether the game is featured (advertised, promoted).
    pub featured: bool
}

impl GameManifestReference {
    pub fn to_json(&self) -> Json {
        json!({
            "url": self.url,
            "featured": self.featured
        })
    }

    pub fn from_json(
        value: &Json
    ) -> Result<Self, GamesRegistryManifestDeserializeError> {
        Ok(Self {
            url: value.get("url")
                .and_then(Json::as_str)
                .ok_or(GamesRegistryManifestDeserializeError::MissingGameManifestUrl)?
                .to_string(),

            featured: value.get("featured")
                .and_then(Json::as_bool)
                .unwrap_or_default()
        })
    }
}
