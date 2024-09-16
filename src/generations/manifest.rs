use serde_json::{json, Value as Json};

use crate::core::prelude::*;
use crate::games::prelude::*;
use crate::packages::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    /// Format of the generation.
    pub format: u64,

    /// UTC timestamp of the generation creation time.
    pub generated_at: u64,

    /// List of games added by the user.
    pub games: Vec<Game>,

    /// Lock file of the game integration packages.
    pub lock_file: LockFileManifest
}

impl Manifest {
    /// Compose new generation manifest from given parts.
    pub fn compose(games: impl Into<Vec<Game>>, lock_file: LockFileManifest) -> Self {
        Self {
            format: 1,
            generated_at: lock_file.metadata.generated_at,
            games: games.into(),
            lock_file
        }
    }
}

impl AsJson for Manifest {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "format": self.format,
            "generated_at": self.generated_at,

            "games": self.games.iter()
                .map(Game::to_json)
                .collect::<Result<Vec<_>, _>>()?,

            "lock_file": self.lock_file.to_json()?
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            format: json.get("format")
                .ok_or_else(|| AsJsonError::FieldNotFound("format"))?
                .as_u64()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("format"))?,

            generated_at: json.get("generated_at")
                .ok_or_else(|| AsJsonError::FieldNotFound("generated_at"))?
                .as_u64()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("generated_at"))?,

            games: json.get("games")
                .ok_or_else(|| AsJsonError::FieldNotFound("games"))?
                .as_array()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("games"))?
                .iter()
                .map(Game::from_json)
                .collect::<Result<Vec<_>, _>>()?,

            lock_file: json.get("lock_file")
                .map(LockFileManifest::from_json)
                .ok_or_else(|| AsJsonError::FieldNotFound("lock_file"))??
        })
    }
}

impl AsHash for Manifest {
    fn hash(&self) -> Hash {
        self.format.hash()
            .chain(self.generated_at.hash())
            .chain(self.games.hash())
            .chain(self.lock_file.hash())
    }

    fn partial_hash(&self) -> Hash {
        self.format.partial_hash()
            .chain(self.games.partial_hash())
            .chain(self.lock_file.partial_hash())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    /// URL of the game's manifest.
    pub url: String,

    /// Fetched manifest of the game.
    pub manifest: GameManifest
}

impl AsJson for Game {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "url": self.url,
            "manifest": self.manifest.to_json()?
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            url: json.get("url")
                .ok_or_else(|| AsJsonError::FieldNotFound("games[].url"))?
                .as_str()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("games[].url"))?
                .to_string(),

            manifest: json.get("manifest")
                .map(GameManifest::from_json)
                .ok_or_else(|| AsJsonError::FieldNotFound("games[].manifest"))??
        })
    }
}

impl AsHash for Game {
    #[inline]
    fn hash(&self) -> Hash {
        self.url.hash().chain(self.manifest.hash())
    }
}
