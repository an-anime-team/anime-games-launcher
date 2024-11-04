use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde_json::{json, Value as Json};

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub format: u64,
    pub title: LocalizableString,
    pub resources: HashMap<Hash, ResourceInfo>
}

impl AsJson for Manifest {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "format": self.format,
            "title": self.title.to_json()?,

            "resources": self.resources.iter()
                .map(|(hash, resource)| {
                    resource.to_json()
                        .map(|resource| (hash.to_base32(), resource))
                })
                .collect::<Result<HashMap<_, _>, _>>()?
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            format: json.get("format")
                .ok_or_else(|| AsJsonError::FieldNotFound("format"))?
                .as_u64()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("format"))?,

            title: json.get("title")
                .map(LocalizableString::from_json)
                .ok_or_else(|| AsJsonError::FieldNotFound("title"))??,

            resources: json.get("resources")
                .ok_or_else(|| AsJsonError::FieldNotFound("resources"))?
                .as_object()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("resources"))?
                .iter()
                .map(|(hash, resource)| {
                    let hash = Hash::from_base32(hash)
                        .ok_or_else(|| AsJsonError::InvalidFieldValue("resources[hash]"))?;

                    let resource = ResourceInfo::from_json(resource)?;

                    Ok((hash, resource))
                })
                .collect::<Result<HashMap<_, _>, AsJsonError>>()?
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceInfo {
    pub title: LocalizableString,
    pub description: LocalizableString,
    pub status: ResourceStatus
}

impl AsJson for ResourceInfo {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "title": self.title.to_json()?,
            "description": self.description.to_json()?,
            "status": self.status.to_json()?
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            title: json.get("title")
                .map(LocalizableString::from_json)
                .ok_or_else(|| AsJsonError::FieldNotFound("resources[].title"))??,

            description: json.get("description")
                .map(LocalizableString::from_json)
                .ok_or_else(|| AsJsonError::FieldNotFound("resources[].description"))??,

            status: json.get("status")
                .map(ResourceStatus::from_json)
                .ok_or_else(|| AsJsonError::FieldNotFound("resources[].status"))??
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceStatus {
    /// Compromised resources are general resources which were
    /// designed with good intentions but were proved to contain
    /// code exploitable by malicious actors. For example,
    /// a compromised resource can be a luau module with extended
    /// privileges which was using them for good purposes but
    /// contained a bug which could be abused by other luau modules
    /// without extended privileges to escape the sandbox themselves.
    /// Compromised resources don't have any special treatment.
    /// This category exists for statistical and UI purposes.
    Compromised {
        /// URL to the page with detailed explanation.
        details_url: Option<String>
    },

    /// Malicious resources are resources which were intentionally made
    /// to perform bad behavior on user system. These could be viruses
    /// or luau modules with hidden behavior.
    Malicious {
        /// URL to the page with detailed explanation.
        details_url: Option<String>
    },

    /// Trusted resources are made by known people, proved
    /// to not contain any malicious code and which require extended
    /// privileges to function.
    Trusted {
        /// ! Extended privilege: allow access to the Process API.
        /// 
        /// Process API allows luau module to run any binaries
        /// or shell commands on the host system without sandboxing.
        allow_process_api: bool
    }
}

impl AsJson for ResourceStatus {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        match self {
            Self::Compromised { details_url: Some(details_url) } => {
                Ok(json!({
                    "type": "compromised",
                    "details_url": details_url
                }))
            }

            Self::Compromised { details_url: None } => Ok(json!("compromised")),

            Self::Malicious { details_url: Some(details_url) } => {
                Ok(json!({
                    "type": "malicious",
                    "details_url": details_url
                }))
            }

            Self::Malicious { details_url: None } => Ok(json!("malicious")),

            Self::Trusted { allow_process_api } => {
                Ok(json!({
                    "type": "trusted",
                    "extended_privileges": {
                        "process_api": allow_process_api
                    }
                }))
            }
        }
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        if let Some(status) = json.as_str() {
            match status {
                "compromised" => Ok(Self::Compromised { details_url: None }),
                "malicious"   => Ok(Self::Malicious { details_url: None }),

                _ => Err(AsJsonError::InvalidFieldValue("resources[].status"))
            }
        }

        else {
            let status = json.get("type")
                .ok_or_else(|| AsJsonError::FieldNotFound("resources[].status.type"))?
                .as_str()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("resources[].status.type"))?;

            match status {
                "compromised" => Ok(Self::Compromised {
                    details_url: json.get("details_url")
                        .ok_or_else(|| AsJsonError::FieldNotFound("resources[].status.details_url"))?
                        .as_str()
                        .map(String::from)
                        .map(Some)
                        .ok_or_else(|| AsJsonError::InvalidFieldValue("resources[].status.details_url"))?
                }),

                "malicious" => Ok(Self::Malicious {
                    details_url: json.get("details_url")
                    .ok_or_else(|| AsJsonError::FieldNotFound("resources[].status.details_url"))?
                    .as_str()
                    .map(String::from)
                    .map(Some)
                    .ok_or_else(|| AsJsonError::InvalidFieldValue("resources[].status.details_url"))?
                }),

                "trusted" => {
                    let extended_privileges = json.get("extended_privileges")
                        .ok_or_else(|| AsJsonError::FieldNotFound("resources[].status.extended_privileges"))?;

                    Ok(Self::Trusted {
                        allow_process_api: extended_privileges.get("process_api")
                            .ok_or_else(|| AsJsonError::FieldNotFound("resources[].status.extended_privileges.process_api"))?
                            .as_bool()
                            .ok_or_else(|| AsJsonError::InvalidFieldValue("resources[].status.extended_privileges.process_api"))?
                    })
                }

                _ => Err(AsJsonError::InvalidFieldValue("resources[].status.type"))
            }
        }
    }
}
