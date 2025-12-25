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

use std::collections::HashSet;
use std::str::FromStr;

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
    pub title: LocalizableString,
    pub description: LocalizableString,
    pub developer: LocalizableString,
    pub publisher: LocalizableString,
    pub images: GameImages,
    pub tags: HashSet<GameTag>
}

impl GameInfo {
    pub fn to_json(&self) -> Json {
        json!({
            "title": self.title.to_json(),
            "description": self.description.to_json(),
            "developer": self.developer.to_json(),
            "publisher": self.publisher.to_json(),
            "images": self.images.to_json(),
            "tags": self.tags.iter()
                .map(|tag| tag.to_string())
                .collect::<Vec<_>>()
        })
    }

    pub fn from_json(value: &Json) -> Result<Self, GameInfoDeserializeError> {
        Ok(Self {
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
                        .flat_map(|tag| {
                            tag.as_str()
                                .and_then(|tag| GameTag::from_str(tag).ok())
                        })
                        .collect::<HashSet<_>>()
                })
                .unwrap_or_default()
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
