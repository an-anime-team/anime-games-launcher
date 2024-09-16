use serde::{Serialize, Deserialize};
use serde_json::{json, Value as Json};

use crate::core::prelude::*;
use crate::packages::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Resource {
    pub uri: String,
    pub format: PackageResourceFormat,
    pub hash: Option<Hash>
}

impl AsJson for Resource {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "uri": self.uri,
            "format": self.format.to_string(),
            "hash": self.hash.as_ref()
                .map(Hash::to_base32)
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            uri: json.get("uri")
                .ok_or_else(|| AsJsonError::FieldNotFound("uri"))?
                .as_str()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("uri"))?
                .to_string(),

            format: json.get("format")
                .and_then(Json::as_str)
                .map(PackageResourceFormat::predict)
                .ok_or_else(|| AsJsonError::InvalidFieldValue("format"))?,

            hash: {
                match json.get("hash") {
                    Some(hash) if hash.is_null() => None,

                    Some(hash) => hash.as_str()
                        .and_then(Hash::from_base32)
                        .map(Some)
                        .ok_or_else(|| AsJsonError::InvalidFieldValue("hash"))?,

                    None => None
                }
            }
        })
    }
}

impl AsHash for Resource {
    fn hash(&self) -> Hash {
        self.uri.hash()
            .chain(self.format.hash())
            .chain(self.hash.hash())
    }
}
