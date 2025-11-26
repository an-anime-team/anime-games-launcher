use serde_json::{json, Value as Json};

use crate::core::prelude::*;
use crate::packages::prelude::*;
use crate::games::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuHardwareRequirements {
    pub model: LocalizableString,
    pub cores: Option<u64>,
    pub frequency: Option<u64>
}

impl AsJson for CpuHardwareRequirements {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "model": self.model.to_json()?,
            "cores": self.cores,
            "frequency": self.frequency
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            model: json.get("model")
                .ok_or_else(|| AsJsonError::FieldNotFound("info.hardware_requirements[].cpu.model"))
                .and_then(LocalizableString::from_json)?,

            cores: {
                match json.get("cores") {
                    Some(cores) => {
                        if cores.is_null() {
                            None
                        } else {
                            cores.as_u64()
                                .map(Some)
                                .ok_or_else(|| AsJsonError::FieldNotFound("info.hardware_requirements[].cpu.cores"))?
                        }
                    }

                    None => None
                }
            },

            frequency: {
                match json.get("frequency") {
                    Some(frequency) => {
                        if frequency.is_null() {
                            None
                        } else {
                            frequency.as_u64()
                                .map(Some)
                                .ok_or_else(|| AsJsonError::FieldNotFound("info.hardware_requirements[].cpu.frequency"))?
                        }
                    }

                    None => None
                }
            }
        })
    }
}

impl AsHash for CpuHardwareRequirements {
    fn hash(&self) -> Hash {
        self.model.hash()
            .chain(self.cores.hash())
            .chain(self.frequency.hash())
    }
}
