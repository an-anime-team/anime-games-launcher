// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-games
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

use std::collections::HashSet;

use serde_json::{json, Value as Json};

use agl_locale::string::LocalizableString;

use super::game_tag::GameTag;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GameInfoDeserializeError {
    #[error("game title is not specified")]
    MissingTitle,

    #[error("game description is not specified")]
    MissingDescription,

    #[error("game developer is not specified")]
    MissingDeveloper,

    #[error("game publisher is not specified")]
    MissingPublisher,

    #[error("game images are not provided")]
    MissingImages,

    #[error("game icon URL is not specified")]
    MissingGameIcon,

    #[error("game poster URL is not specified")]
    MissingGamePoster,

    #[error("game background image URL is not specified")]
    MissingGameBackground,

    #[error("game slide images URLs are not provided")]
    MissingGameSlides,

    #[error("invalid localizable string format")]
    InvalidLocalizableString
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameInfo {
    /// Unique game identifier.
    ///
    /// This identifier will be used to select a single manifest for the same
    /// game. For example, an application could pull manifests from different
    /// locations. This field could hint the application that some of the pulled
    /// manifests are made for the same game, and the application could choose
    /// one of the manifests to show to the user.
    ///
    /// If unset - upstream implementation will decide how to identify the game.
    pub name: Option<String>,

    /// Game title.
    pub title: LocalizableString,

    /// Game description.
    pub description: LocalizableString,

    /// Short information about the game's developer. Normally displayed in
    /// a single line with limited width.
    pub developer: LocalizableString,

    /// Short information about the game's publisher. Normally displayed in
    /// a single line with limited width.
    pub publisher: LocalizableString,

    /// Information about the game's images (icon, background, etc.).
    pub images: GameImages,

    /// List of game tags.
    pub tags: HashSet<GameTag>,

    /// Information displayed to the user before adding this game to their
    /// library. Can be a license agreement, a warning message, contain
    /// instructions, or be used in any other way.
    ///
    /// The user can either accept this agreement and then the game integration
    /// will be installed and displayed in their library page, or decline it
    /// so the game integration will not be installed.
    pub agreement: Option<LocalizableString>
}

impl GameInfo {
    pub fn to_json(&self) -> Json {
        json!({
            "name": self.name,
            "title": self.title.to_json(),
            "description": self.description.to_json(),
            "developer": self.developer.to_json(),
            "publisher": self.publisher.to_json(),
            "images": self.images.to_json(),
            "tags": self.tags.iter()
                .map(|tag| tag.to_string())
                .collect::<Vec<_>>(),
            "agreement": self.agreement.as_ref()
                .map(LocalizableString::to_json)
        })
    }

    pub fn from_json(value: &Json) -> Result<Self, GameInfoDeserializeError> {
        Ok(Self {
            name: value.get("name")
                .and_then(Json::as_str)
                .map(String::from),

            title: value.get("title")
                .ok_or(GameInfoDeserializeError::MissingTitle)
                .and_then(|title| {
                    LocalizableString::from_json(title)
                        .ok_or(GameInfoDeserializeError::InvalidLocalizableString)
                })?,

            description: value.get("description")
                .ok_or(GameInfoDeserializeError::MissingDescription)
                .and_then(|description| {
                    LocalizableString::from_json(description)
                        .ok_or(GameInfoDeserializeError::InvalidLocalizableString)
                })?,

            developer: value.get("developer")
                .ok_or(GameInfoDeserializeError::MissingDeveloper)
                .and_then(|developer| {
                    LocalizableString::from_json(developer)
                        .ok_or(GameInfoDeserializeError::InvalidLocalizableString)
                })?,

            publisher: value.get("publisher")
                .ok_or(GameInfoDeserializeError::MissingPublisher)
                .and_then(|publisher| {
                    LocalizableString::from_json(publisher)
                        .ok_or(GameInfoDeserializeError::InvalidLocalizableString)
                })?,

            images: value.get("images")
                .ok_or(GameInfoDeserializeError::MissingImages)
                .and_then(GameImages::from_json)?,

            tags: value.get("tags")
                .and_then(Json::as_array)
                .map(|tags| {
                    tags.iter()
                        .flat_map(|tag| tag.as_str().map(GameTag::from))
                        .collect::<HashSet<_>>()
                })
                .unwrap_or_default(),

            agreement: value.get("agreement")
                .and_then(LocalizableString::from_json)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameImages {
    pub icon: String,
    pub poster: String,
    pub background: String,
    pub slides: Vec<String>
}

impl GameImages {
    pub fn to_json(&self) -> Json {
        json!({
            "icon": self.icon,
            "poster": self.poster,
            "background": self.background,
            "slides": self.slides
        })
    }

    pub fn from_json(value: &Json) -> Result<Self, GameInfoDeserializeError> {
        Ok(Self {
            icon: value.get("icon").and_then(Json::as_str)
                .ok_or(GameInfoDeserializeError::MissingGameIcon)?
                .to_string(),

            poster: value.get("poster").and_then(Json::as_str)
                .ok_or(GameInfoDeserializeError::MissingGamePoster)?
                .to_string(),

            background: value.get("background").and_then(Json::as_str)
                .ok_or(GameInfoDeserializeError::MissingGameBackground)?
                .to_string(),

            slides: value.get("slides")
                .and_then(Json::as_array)
                .and_then(|slides| {
                    slides.iter()
                        .map(|url| url.as_str().map(String::from))
                        .collect::<Option<Vec<_>>>()
                })
                .ok_or(GameInfoDeserializeError::MissingGameSlides)?
        })
    }
}
