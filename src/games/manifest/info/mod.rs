use std::collections::HashSet;

use serde_json::{json, Value as Json};

use crate::core::prelude::*;

pub mod hardware_requirements;
pub mod game_tag;

use hardware_requirements::GameHardwareRequirements;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Info {
    pub hardware_requirements: Option<GameHardwareRequirements>,
    pub tags: Option<HashSet<game_tag::GameTag>>
}

impl AsJson for Info {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "hardware_requirements": self.hardware_requirements.as_ref()
                .map(GameHardwareRequirements::to_json)
                .transpose()?,

            "tags": self.tags.as_ref()
                .map(|tags| {
                    tags.iter()
                        .map(|tag| tag.to_string())
                        .collect::<Vec<_>>()
                })
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            hardware_requirements: {
                match json.get("hardware_requirements") {
                    Some(hardware_requirements) => {
                        if hardware_requirements.is_null() {
                            None
                        } else {
                            GameHardwareRequirements::from_json(hardware_requirements)
                                .map(Some)?
                        }
                    }

                    None => None
                }
            },

            tags: {
                match json.get("tags") {
                    Some(tags) => {
                        if tags.is_null() {
                            None
                        } else if let Some(tags) = tags.as_array() {
                            let tags = tags.iter()
                                .map(|tag| {
                                    tag.as_str()
                                        .ok_or_else(|| AsJsonError::InvalidFieldValue("info.tags[]"))
                                        .and_then(|tag| {
                                            tag.parse::<game_tag::GameTag>()
                                                .map_err(|err| AsJsonError::Other(err.into()))
                                        })
                                })
                                .collect::<Result<HashSet<_>, _>>()?;

                            Some(tags)
                        } else {
                            return Err(AsJsonError::InvalidFieldValue("info.tags"));
                        }
                    }

                    None => None
                }
            }
        })
    }
}
