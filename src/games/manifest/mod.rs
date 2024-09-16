use serde_json::{json, Value as Json};

use crate::core::prelude::*;
use crate::packages::prelude::*;

pub mod localizable_string;
pub mod game;
pub mod package;
pub mod info;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameManifest {
    pub standard: u64,
    pub game: game::Game,
    pub package: package::Package,
    pub info: Option<info::Info>
}

impl AsJson for GameManifest {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "standard": self.standard,
            "game": self.game.to_json()?,
            "package": self.package.to_json()?,
            "info": self.info.as_ref()
                .map(info::Info::to_json)
                .transpose()?
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            standard: json.get("standard")
                .ok_or_else(|| AsJsonError::FieldNotFound("standard"))?
                .as_u64()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("standard"))?,

            game: json.get("game")
                .map(game::Game::from_json)
                .ok_or_else(|| AsJsonError::FieldNotFound("game"))??,

            package: json.get("package")
                .map(package::Package::from_json)
                .ok_or_else(|| AsJsonError::FieldNotFound("pacakge"))??,

            info: json.get("info")
                .map(info::Info::from_json)
                .transpose()?
        })
    }
}

impl AsHash for GameManifest {
    fn hash(&self) -> Hash {
        self.standard.hash()
            .chain(self.game.hash())
            .chain(self.package.hash())
            .chain(self.info.hash())
    }
}
