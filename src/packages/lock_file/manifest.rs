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
    pub root: Vec<u32>,
    pub resources: Vec<ResourceLock>
}

impl AsJson for Manifest {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "standard": self.standard,
            "metadata": self.metadata.to_json()?,
            "root": self.root,

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
                .map(|root| root.as_u64().map(|root| root as u32))
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

impl AsHash for Manifest {
    fn hash(&self) -> Hash {
        self.standard.hash()
            .chain(self.metadata.hash())
            .chain(self.root.hash())
            .chain(self.resources.hash())
    }

    fn partial_hash(&self) -> Hash {
        self.standard.partial_hash()
            .chain(self.root.partial_hash())
            .chain(self.resources.partial_hash())
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

impl AsHash for LockFileMetadata {
    #[inline]
    fn hash(&self) -> Hash {
        self.generated_at.hash()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceLock {
    pub url: String,
    pub format: PackageResourceFormat,
    pub lock: ResourceLockData,
    pub inputs: Option<HashMap<String, u32>>,
    pub outputs: Option<HashMap<String, u32>>
}

impl AsJson for ResourceLock {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "url": self.url,
            "format": self.format.to_string(),
            "lock": self.lock.to_json()?,
            "inputs": self.inputs,
            "outputs": self.outputs
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
                            v.as_u64().map(|v| (k.to_string(), v as u32))
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
                            v.as_u64().map(|v| (k.to_string(), v as u32))
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

impl AsHash for ResourceLock {
    fn hash(&self) -> Hash {
        self.url.hash()
            .chain(self.format.hash())
            .chain(self.lock.hash())
            .chain(self.inputs.hash())
            .chain(self.outputs.hash())
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

impl AsHash for ResourceLockData {
    #[inline]
    fn hash(&self) -> Hash {
        self.hash
    }
}
