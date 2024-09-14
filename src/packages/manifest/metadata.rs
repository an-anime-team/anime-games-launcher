use serde::{Serialize, Deserialize};
use serde_json::{json, Value as Json};

use crate::core::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub maintainers: Option<Vec<String>>
}

impl AsJson for PackageMetadata {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "maintainers": self.maintainers
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            maintainers: {
                match json.get("maintainers") {
                    Some(maintainers) if maintainers.is_null() => None,

                    Some(maintainers) => maintainers.as_array()
                        .and_then(|maintainers| {
                            maintainers.iter()
                                .map(|maintainer| {
                                    maintainer.as_str()
                                        .map(String::from)
                                })
                                .collect::<Option<Vec<_>>>()
                        })
                        .map(Some)
                        .ok_or_else(|| AsJsonError::InvalidFieldValue("package.maintainers"))?,

                    None => None
                }
            }
        })
    }
}
