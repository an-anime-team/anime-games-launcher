use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use serde_json::{json, Value as Json};

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub format: u64,
    pub title: LocalizableString,
    pub resources: Vec<ResourceInfo>
}

impl AsJson for Manifest {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "format": self.format,
            "title": self.title.to_json()?,

            "resources": self.resources.iter()
                .map(ResourceInfo::to_json)
                .collect::<Result<Vec<_>, _>>()?
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
                .as_array()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("resources"))?
                .iter()
                .map(ResourceInfo::from_json)
                .collect::<Result<Vec<_>, AsJsonError>>()?
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceInfo {
    pub title: LocalizableString,
    pub description: Option<LocalizableString>,
    pub variants: HashMap<Hash, ResourceStatus>
}

impl AsJson for ResourceInfo {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "title": self.title.to_json()?,

            "description": self.description.as_ref()
                .map(LocalizableString::to_json)
                .transpose()?,

            "variants": self.variants.iter()
                .map(|(hash, status)| Ok::<_, AsJsonError>((hash.to_base32(), status.to_json()?)))
                .collect::<Result<HashMap<String, Json>, _>>()?
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            title: json.get("title")
                .map(LocalizableString::from_json)
                .ok_or_else(|| AsJsonError::FieldNotFound("resources[].title"))??,

            description: json.get("description")
                .map(|description| LocalizableString::from_json(description).map(Some))
                .unwrap_or(Ok(None))?,

            variants: json.get("variants")
                .ok_or_else(|| AsJsonError::FieldNotFound("resources[].variants"))?
                .as_object()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("resources[].variants"))?
                .iter()
                .map(|(hash, status)| {
                    let hash = Hash::from_base32(hash)
                        .ok_or_else(|| AsJsonError::InvalidFieldValue("resources[].variants[hash]"))?;

                    let status = ResourceStatus::from_json(status)?;

                    Ok((hash, status))
                })
                .collect::<Result<HashMap<_, _>, AsJsonError>>()?
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceStatus {
    /// Trusted resources are made by known people, proven
    /// to not contain any malicious code.
    Trusted {
        /// ! Extended privilege: allow access to the Process API.
        ///
        /// Process API allows luau module to run any binaries
        /// or shell commands on the host system without sandboxing.
        ext_process_api: Option<bool>,

        /// List of extra paths allowed to be accessed by the module.
        allowed_paths: Option<Vec<PathBuf>>
    },

    /// Compromised resources are general resources which were
    /// designed with good intentions but were proven to contain
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
    }
}

impl AsJson for ResourceStatus {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        match self {
            Self::Trusted { ext_process_api: Some(ext_process_api), allowed_paths: Some(allowed_paths) } => {
                Ok(json!({
                    "type": "trusted",
                    "privileges": {
                        "process_api": ext_process_api
                    },
                    "allowed_paths": allowed_paths
                }))
            },

            Self::Trusted { ext_process_api: Some(ext_process_api), allowed_paths: None } => {
                Ok(json!({
                    "type": "trusted",
                    "privileges": {
                        "process_api": ext_process_api
                    }
                }))
            },

            Self::Trusted { ext_process_api: None, allowed_paths: Some(allowed_paths) } => {
                Ok(json!({
                    "type": "trusted",
                    "allowed_paths": allowed_paths
                }))
            },

            Self::Trusted { ext_process_api: None, allowed_paths: None } => Ok(json!("trusted")),

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

            Self::Malicious { details_url: None } => Ok(json!("malicious"))
        }
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        if let Some(status) = json.as_str() {
            match status {
                "trusted"     => Ok(Self::Trusted { ext_process_api: None, allowed_paths: None }),
                "compromised" => Ok(Self::Compromised { details_url: None }),
                "malicious"   => Ok(Self::Malicious { details_url: None }),

                _ => Err(AsJsonError::InvalidFieldValue("resources[].variants[]"))
            }
        }

        else {
            let status = json.get("status")
                .ok_or_else(|| AsJsonError::FieldNotFound("resources[].variants[].status"))?
                .as_str()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("resources[].variants[].status"))?;

            match status {
                "trusted" => {
                    // TODO: handle invalid format instead of silencing it with None.
                    Ok(Self::Trusted {
                        ext_process_api: json.get("privileges")
                            .and_then(|privileges| {
                                privileges.get("process_api")
                                    .and_then(Json::as_bool)
                            }),

                        allowed_paths: json.get("allowed_paths")
                            .and_then(|allowed_paths| {
                                allowed_paths.as_array()
                                    .map(|allowed_paths| {
                                        allowed_paths.iter()
                                            .flat_map(Json::as_str)
                                            .map(PathBuf::from)
                                            .collect::<Vec<_>>()
                                    })
                            })
                    })
                }

                "compromised" => Ok(Self::Compromised {
                    details_url: json.get("details_url")
                        .ok_or_else(|| AsJsonError::FieldNotFound("resources[].variants[].details_url"))?
                        .as_str()
                        .map(String::from)
                        .map(Some)
                        .ok_or_else(|| AsJsonError::InvalidFieldValue("resources[].variants[].details_url"))?
                }),

                "malicious" => Ok(Self::Malicious {
                    details_url: json.get("details_url")
                        .ok_or_else(|| AsJsonError::FieldNotFound("resources[].variants[].details_url"))?
                        .as_str()
                        .map(String::from)
                        .map(Some)
                        .ok_or_else(|| AsJsonError::InvalidFieldValue("resources[].variants[].details_url"))?
                }),

                _ => Err(AsJsonError::InvalidFieldValue("resources[].variants[].status"))
            }
        }
    }
}
