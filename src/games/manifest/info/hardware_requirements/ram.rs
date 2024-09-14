use serde_json::{json, Value as Json};

use crate::core::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RamHardwareRequirements {
    pub size: u64,
    pub frequency: Option<u64>
}

impl AsJson for RamHardwareRequirements {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "size": self.size,
            "frequency": self.frequency
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            size: json.get("size")
                .ok_or_else(|| AsJsonError::FieldNotFound("info.hardware_requirements[].ram.size"))?
                .as_u64()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("info.hardware_requirements[].ram.size"))?,

            frequency: {
                match json.get("frequency") {
                    Some(frequency) => {
                        if frequency.is_null() {
                            None
                        } else {
                            frequency.as_u64()
                                .map(Some)
                                .ok_or_else(|| AsJsonError::InvalidFieldValue("info.hardware_requirements[].ram.frequency"))?
                        }
                    }

                    None => None
                }
            }
        })
    }
}
