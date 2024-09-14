use std::collections::HashMap;
use std::str::FromStr;

use serde::{Serialize, Deserialize};
use serde_json::{json, Value as Json};

use crate::core::prelude::*;
use crate::packages::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub standard: u64,
    pub metadata: LockFileMetadata,
    pub root: Vec<Hash>,
    pub resources: Vec<ResourceLock>
}

impl AsJson for Manifest {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "standard": self.standard,
            "metadata": self.metadata.to_json()?,

            "root": self.root.iter()
                .map(Hash::to_base32)
                .collect::<Vec<_>>(),

            "resources": self.resources.iter()
                .map(ResourceLock::to_json)
                .collect::<Result<Vec<_>, _>>()?
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            standard: json.get("standard")
                .ok_or_else(|| AsJsonError::FieldNotFound("standard"))?
                .as_u64()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("standard"))?,

            metadata: json.get("metadata")
                .map(LockFileMetadata::from_json)
                .ok_or_else(|| AsJsonError::FieldNotFound("metadata"))??,

            root: json.get("root")
                .ok_or_else(|| AsJsonError::FieldNotFound("root"))?
                .as_array()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("root"))?
                .iter()
                .map(|root| {
                    root.as_str()
                        .and_then(Hash::from_base32)
                })
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("root"))?,

            resources: json.get("resources")
                .ok_or_else(|| AsJsonError::FieldNotFound("resources"))?
                .as_array()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("resources"))?
                .iter()
                .map(ResourceLock::from_json)
                .collect::<Result<Vec<_>, _>>()?
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LockFileMetadata {
    pub generated_at: u64
}

impl AsJson for LockFileMetadata {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "generated_at": self.generated_at
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            generated_at: json.get("generated_at")
                .ok_or_else(|| AsJsonError::FieldNotFound("metadata.generated_at"))?
                .as_u64()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("metadata.generated_at"))?
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceLock {
    pub url: String,
    pub format: PackageResourceFormat,
    pub lock: ResourceLockData,
    pub inputs: Option<HashMap<String, Hash>>,
    pub outputs: Option<HashMap<String, Hash>>
}

impl AsJson for ResourceLock {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "url": self.url,
            "format": self.format.to_string(),
            "lock": self.lock.to_json()?,

            "inputs": self.inputs.as_ref()
                .map(|inputs| {
                    inputs.iter()
                        .map(|(k, v)| (k, v.to_base32()))
                        .collect::<HashMap<_, _>>()
                }),

            "outputs": self.outputs.as_ref()
                .map(|outputs| {
                    outputs.iter()
                        .map(|(k, v)| (k, v.to_base32()))
                        .collect::<HashMap<_, _>>()
                })
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            url: json.get("url")
                .ok_or_else(|| AsJsonError::FieldNotFound("resources[].url"))?
                .as_str()
                .map(String::from)
                .ok_or_else(|| AsJsonError::InvalidFieldValue("resources[].url"))?,

            format: json.get("format")
                .ok_or_else(|| AsJsonError::FieldNotFound("resources[].format"))?
                .as_str()
                .map(PackageResourceFormat::from_str)
                .ok_or_else(|| AsJsonError::InvalidFieldValue("resources[].format"))?
                .map_err(|err| AsJsonError::Other(err.into()))?,

            lock: json.get("lock")
                .map(ResourceLockData::from_json)
                .ok_or_else(|| AsJsonError::FieldNotFound("resources[].lock"))??,

            inputs: {
                match json.get("inputs") {
                    Some(inputs) if inputs.is_null() => None,

                    Some(inputs) => inputs.as_object()
                        .ok_or_else(|| AsJsonError::InvalidFieldValue("resources[].inputs"))?
                        .into_iter()
                        .map(|(k, v)| {
                            v.as_str()
                                .and_then(Hash::from_base32)
                                .map(|v| (k.to_string(), v))
                        })
                        .collect::<Option<HashMap<_, _>>>()
                        .map(Some)
                        .ok_or_else(|| AsJsonError::InvalidFieldValue("resources[].inputs"))?,

                    None => None
                }
            },

            outputs: {
                match json.get("outputs") {
                    Some(outputs) if outputs.is_null() => None,

                    Some(outputs) => outputs.as_object()
                        .ok_or_else(|| AsJsonError::InvalidFieldValue("resources[].outputs"))?
                        .into_iter()
                        .map(|(k, v)| {
                            v.as_str()
                                .and_then(Hash::from_base32)
                                .map(|v| (k.to_string(), v))
                        })
                        .collect::<Option<HashMap<_, _>>>()
                        .map(Some)
                        .ok_or_else(|| AsJsonError::InvalidFieldValue("resources[].outputs"))?,

                    None => None
                }
            }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceLockData {
    pub hash: Hash,
    pub size: u64
}

impl AsJson for ResourceLockData {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "hash": self.hash.to_base32(),
            "size": self.size
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            hash: json.get("hash")
                .ok_or_else(|| AsJsonError::FieldNotFound("resources[].lock.hash"))?
                .as_str()
                .and_then(Hash::from_base32)
                .ok_or_else(|| AsJsonError::InvalidFieldValue("resources[].lock.hash"))?,

            size: json.get("size")
                .ok_or_else(|| AsJsonError::FieldNotFound("resources[].lock.size"))?
                .as_u64()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("resources[].lock.size"))?
        })
    }
}