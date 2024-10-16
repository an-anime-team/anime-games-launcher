use serde::{Serialize, Deserialize};
use serde_json::{json, Value as Json};

use crate::core::prelude::*;
use crate::packages::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Package {
    pub url: String,
    pub output: String
}

impl AsJson for Package {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "url": self.url,
            "output": self.output
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            url: json.get("url")
                .ok_or_else(|| AsJsonError::FieldNotFound("package.url"))?
                .as_str()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("package.url"))?
                .to_string(),

            output: json.get("output")
                .ok_or_else(|| AsJsonError::FieldNotFound("package.output"))?
                .as_str()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("package.output"))?
                .to_string()
        })
    }
}

impl AsHash for Package {
    #[inline]
    fn hash(&self) -> Hash {
        self.url.hash().chain(self.output.hash())
    }
}
