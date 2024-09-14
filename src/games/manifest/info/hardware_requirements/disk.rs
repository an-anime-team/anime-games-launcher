use std::str::FromStr;

use serde_json::{json, Value as Json};

use crate::core::prelude::*;

use super::disk_type::DiskType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskHardwareRequirements {
    pub size: u64,
    pub disk_type: Option<DiskType>
}

impl AsJson for DiskHardwareRequirements {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "size": self.size,
            "type": self.disk_type.as_ref()
                .map(DiskType::to_string)
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            size: json.get("size")
                .ok_or_else(|| AsJsonError::FieldNotFound("info.hardware_requirements[].disk.size"))?
                .as_u64()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("info.hardware_requirements[].disk.size"))?,

            disk_type: {
                match json.get("type") {
                    Some(disk_type) => {
                        if disk_type.is_null() {
                            None
                        } else if let Some(disk_type) = disk_type.as_str() {
                            DiskType::from_str(disk_type)
                                .map(Some)
                                .map_err(|err| AsJsonError::Other(err.into()))?
                        } else {
                            return Err(AsJsonError::InvalidFieldValue("info.hardware_requirements[].disk.type"));
                        }
                    }

                    None => None
                }
            }
        })
    }
}
