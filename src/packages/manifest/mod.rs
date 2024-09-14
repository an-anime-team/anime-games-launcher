use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde_json::{json, Value as Json};

use crate::core::prelude::*;

pub mod metadata;
pub mod resource_format;
pub mod resource;

pub mod prelude {
    pub use super::Manifest as PackageManifest;
    pub use super::metadata::PackageMetadata;
    pub use super::resource_format::ResourceFormat as PackageResourceFormat;
    pub use super::resource::Resource as PackageResource;
}

use prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub standard: u64,
    pub package: Option<PackageMetadata>,
    pub inputs: Option<HashMap<String, PackageResource>>,
    pub outputs: HashMap<String, PackageResource>
}

impl AsJson for Manifest {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "standard": self.standard,

            "package": self.package.as_ref()
                .map(PackageMetadata::to_json)
                .transpose()?,

            "inputs": self.inputs.as_ref()
                .map(|inputs| {
                    inputs.iter()
                        .map(|(k, v)| {
                            v.to_json().map(|v| (k, v))
                        })
                        .collect::<Result<HashMap<_, _>, _>>()
                }).transpose()?,

            "outputs": self.outputs.iter()
                .map(|(k, v)| {
                    v.to_json().map(|v| (k, v))
                })
                .collect::<Result<HashMap<_, _>, _>>()?
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            standard: json.get("standard")
                .ok_or_else(|| AsJsonError::FieldNotFound("standard"))?
                .as_u64()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("standard"))?,

            package: {
                match json.get("package") {
                    Some(package) if package.is_null() => None,

                    Some(package) => PackageMetadata::from_json(package)
                        .map(Some)?,

                    None => None
                }
            },

            inputs: {
                match json.get("inputs") {
                    Some(inputs) if inputs.is_null() => None,

                    Some(inputs) => inputs.as_object()
                        .map(|inputs| {
                            inputs.into_iter()
                                .map(|(k, v)| {
                                    PackageResource::from_json(v)
                                        .map(|v| (k.to_string(), v))
                                })
                                .collect::<Result<HashMap<_, _>, _>>()
                        })
                        .transpose()?,

                    None => None
                }
            },

            outputs: json.get("outputs")
                .ok_or_else(|| AsJsonError::FieldNotFound("outputs"))?
                .as_object()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("outputs"))?
                .into_iter()
                .map(|(k, v)| {
                    PackageResource::from_json(v)
                        .map(|v| (k.to_string(), v))
                })
                .collect::<Result<HashMap<_, _>, _>>()?
        })
    }
}
