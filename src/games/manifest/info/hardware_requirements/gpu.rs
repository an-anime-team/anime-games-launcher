use serde_json::{json, Value as Json};

use crate::core::prelude::*;
use crate::games::manifest::localizable_string::LocalizableString;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuHardwareRequirements {
    pub model: LocalizableString,
    pub vram: Option<u64>
}

impl AsJson for GpuHardwareRequirements {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "model": self.model.to_json()?,
            "vram": self.vram
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            model: json.get("model")
                .ok_or_else(|| AsJsonError::FieldNotFound("info.hardware_requirements[].gpu.model"))
                .and_then(LocalizableString::from_json)?,

            vram: {
                match json.get("vram") {
                    Some(vram) => {
                        if vram.is_null() {
                            None
                        } else {
                            vram.as_u64()
                                .map(Some)
                                .ok_or_else(|| AsJsonError::InvalidFieldValue("info.hardware_requirements[].gpu.vram"))?
                        }
                    }

                    None => None
                }
            }
        })
    }
}
