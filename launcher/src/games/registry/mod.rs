use serde_json::{json, Value as Json};

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub format: u64,
    pub title: LocalizableString,
    pub games: Vec<GameManifestReference>
}

impl AsJson for Manifest {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "format": self.format,
            "title": self.title.to_json()?,
            "games": self.games.iter()
                .map(AsJson::to_json)
                .collect::<Result<Vec<_>, _>>()?
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            format: json.get("format")
                .ok_or_else(|| AsJsonError::FieldNotFound("format"))?
                .as_u64()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("format"))?,

            title: json.get("title")
                .map(LocalizableString::from_json)
                .ok_or_else(|| AsJsonError::FieldNotFound("title"))??,

            games: json.get("games")
                .ok_or_else(|| AsJsonError::FieldNotFound("games"))?
                .as_array()
                .map(|games| {
                    games.iter()
                        .map(GameManifestReference::from_json)
                        .collect::<Result<Vec<_>, _>>()
                })
                .ok_or_else(|| AsJsonError::InvalidFieldValue("games"))??
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameManifestReference {
    pub url: String,
    pub featured: Option<bool>
}

impl AsJson for GameManifestReference {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "url": self.url,
            "featured": self.featured
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            url: json.get("url")
                .ok_or_else(|| AsJsonError::FieldNotFound("games[].url"))?
                .as_str()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("games[].url"))?
                .to_string(),

            featured: json.get("featured")
                .and_then(Json::as_bool)
        })
    }
}
