use serde_json::{json, Value as Json};

use crate::core::prelude::*;

pub mod cpu;
pub mod gpu;
pub mod ram;
pub mod disk;
pub mod disk_type;
pub mod requirements;

use requirements::HardwareRequirements;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameHardwareRequirements {
    pub minimal: HardwareRequirements,
    pub optimal: Option<HardwareRequirements>
}

impl AsJson for GameHardwareRequirements {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "minimal": self.minimal.to_json()?,
            "optimal": self.optimal.as_ref()
                .map(HardwareRequirements::to_json)
                .transpose()?
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            minimal: json.get("minimal")
                .ok_or_else(|| AsJsonError::FieldNotFound("info.hardware_requirements.minimal"))
                .and_then(HardwareRequirements::from_json)?,

            optimal: {
                match json.get("optimal") {
                    Some(optimal) => {
                        if optimal.is_null() {
                            None
                        } else {
                            HardwareRequirements::from_json(optimal)
                                .map(Some)?
                        }
                    }

                    None => None
                }
            }
        })
    }
}
