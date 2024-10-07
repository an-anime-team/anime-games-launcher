use serde_json::{json, Value as Json};

use crate::core::prelude::*;
use crate::packages::prelude::*;

use super::localizable_string::LocalizableString;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    pub title: LocalizableString,
    pub description: LocalizableString,
    pub developer: LocalizableString,
    pub publisher: LocalizableString,
    pub images: GameImages
}

impl AsJson for Game {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "title": self.title.to_json()?,
            "description": self.description.to_json()?,
            "developer": self.developer.to_json()?,
            "publisher": self.publisher.to_json()?,
            "images": self.images.to_json()?
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            title: json.get("title")
                .ok_or_else(|| AsJsonError::FieldNotFound("game.title"))
                .and_then(LocalizableString::from_json)?,

            description: json.get("description")
                .ok_or_else(|| AsJsonError::FieldNotFound("game.description"))
                .and_then(LocalizableString::from_json)?,

            developer: json.get("developer")
                .ok_or_else(|| AsJsonError::FieldNotFound("game.developer"))
                .and_then(LocalizableString::from_json)?,

            publisher: json.get("publisher")
                .ok_or_else(|| AsJsonError::FieldNotFound("game.publisher"))
                .and_then(LocalizableString::from_json)?,

            images: json.get("images")
                .ok_or_else(|| AsJsonError::FieldNotFound("game.images"))
                .and_then(GameImages::from_json)?
        })
    }
}

impl AsHash for Game {
    fn hash(&self) -> Hash {
        self.title.hash()
            .chain(self.description.hash())
            .chain(self.developer.hash())
            .chain(self.publisher.hash())
            .chain(self.images.hash())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameImages {
    pub icon: String,
    pub poster: String,
    pub background: String,
    pub slides: Vec<String>
}

impl AsJson for GameImages {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "icon": self.icon,
            "poster": self.poster,
            "background": self.background
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            icon: json.get("icon")
                .ok_or_else(|| AsJsonError::FieldNotFound("game.images.icon"))?
                .as_str()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("game.images.icon"))?
                .to_string(),

            poster: json.get("poster")
                .ok_or_else(|| AsJsonError::FieldNotFound("game.images.poster"))?
                .as_str()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("game.images.poster"))?
                .to_string(),

            background: json.get("background")
                .ok_or_else(|| AsJsonError::FieldNotFound("game.images.background"))?
                .as_str()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("game.images.background"))?
                .to_string(),

            slides: json.get("slides")
                .ok_or_else(|| AsJsonError::FieldNotFound("game.images.slides"))?
                .as_array()
                .and_then(|slides| {
                    slides.iter()
                        .map(|url| url.as_str().map(String::from))
                        .collect::<Option<Vec<_>>>()
                })
                .ok_or_else(|| AsJsonError::InvalidFieldValue("game.images.slides"))?
        })
    }
}

impl AsHash for GameImages {
    fn hash(&self) -> Hash {
        self.icon.hash()
            .chain(self.poster.hash())
            .chain(self.background.hash())
            .chain(self.slides.hash())
    }
}
