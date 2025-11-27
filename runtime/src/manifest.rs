// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-runtime
// Copyright (C) 2025  Nikita Podvirnyi <krypt0nn@vk.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::collections::HashSet;
use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use serde_json::{json, Value as Json};

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
    pub variants: Vec<ResourceStatus>
}

impl AsJson for ResourceInfo {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "title": self.title.to_json()?,

            "description": self.description.as_ref()
                .map(LocalizableString::to_json)
                .transpose()?,

            "variants": self.variants.iter()
                .map(ResourceStatus::to_json)
                .collect::<Result<Vec<_>, _>>()?
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
                .as_array()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("resources[].variants"))?
                .iter()
                .map(ResourceStatus::from_json)
                .collect::<Result<Vec<_>, AsJsonError>>()?
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
        allowed_paths: Option<Vec<PathBuf>>,

        /// List of hashes of the resource.
        hashes: HashSet<Hash>
    },

    /// Compromised resources are general resources which were
    /// designed with good intentions but were proven to contain
    /// code exploitable by malicious actors. For example,
    /// a compromised resource can be a luau module with extended
    /// privileges which was using them for good purposes but
    /// contained a bug which could be abused by other luau modules
    /// without extended privileges to escape the sandbox themselves.
    Compromised {
        /// URL to the page with detailed explanation.
        details_url: Option<String>,

        /// List of hashes of the resource.
        hashes: HashSet<Hash>
    },

    /// Malicious resources are resources which were intentionally made
    /// to perform bad behavior on user system. These could be viruses
    /// or luau modules with hidden behavior.
    Malicious {
        /// URL to the page with detailed explanation.
        details_url: Option<String>,

        /// List of hashes of the resource.
        hashes: HashSet<Hash>
    }
}

impl ResourceStatus {
    /// Check if given hash is contained within the current resource status.
    #[inline]
    pub fn contains(&self, hash: &Hash) -> bool {
        match self {
            Self::Trusted { hashes, .. } |
            Self::Compromised { hashes, .. } |
            Self::Malicious { hashes, .. } => hashes.contains(hash)
        }
    }

    /// Check if current status is `trusted`.
    #[inline]
    pub const fn is_trusted(&self) -> bool {
        matches!(self, Self::Trusted { .. })
    }
}

impl AsJson for ResourceStatus {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        match self {
            Self::Trusted { ext_process_api: Some(ext_process_api), allowed_paths: Some(allowed_paths), hashes } => {
                Ok(json!({
                    "status": "trusted",
                    "privileges": {
                        "process_api": ext_process_api
                    },
                    "allowed_paths": allowed_paths,
                    "hashes": hashes.iter().map(Hash::to_base32).collect::<Vec<_>>()
                }))
            }

            Self::Trusted { ext_process_api: Some(ext_process_api), allowed_paths: None, hashes } => {
                Ok(json!({
                    "status": "trusted",
                    "privileges": {
                        "process_api": ext_process_api
                    },
                    "hashes": hashes.iter().map(Hash::to_base32).collect::<Vec<_>>()
                }))
            }

            Self::Trusted { ext_process_api: None, allowed_paths: Some(allowed_paths), hashes } => {
                Ok(json!({
                    "status": "trusted",
                    "allowed_paths": allowed_paths,
                    "hashes": hashes.iter().map(Hash::to_base32).collect::<Vec<_>>()
                }))
            }

            Self::Trusted { ext_process_api: None, allowed_paths: None, hashes } => {
                Ok(json!({
                    "status": "trusted",
                    "hashes": hashes.iter().map(Hash::to_base32).collect::<Vec<_>>()
                }))
            }

            Self::Compromised { details_url: Some(details_url), hashes } => {
                Ok(json!({
                    "status": "compromised",
                    "details_url": details_url,
                    "hashes": hashes.iter().map(Hash::to_base32).collect::<Vec<_>>()
                }))
            }

            Self::Compromised { details_url: None, hashes } => {
                Ok(json!({
                    "status": "compromised",
                    "hashes": hashes.iter().map(Hash::to_base32).collect::<Vec<_>>()
                }))
            }

            Self::Malicious { details_url: Some(details_url), hashes } => {
                Ok(json!({
                    "status": "malicious",
                    "details_url": details_url,
                    "hashes": hashes.iter().map(Hash::to_base32).collect::<Vec<_>>()
                }))
            }

            Self::Malicious { details_url: None, hashes } => {
                Ok(json!({
                    "status": "malicious",
                    "hashes": hashes.iter().map(Hash::to_base32).collect::<Vec<_>>()
                }))
            }
        }
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        let status = json.get("status")
            .ok_or_else(|| AsJsonError::FieldNotFound("resources[].variants[].status"))?
            .as_str()
            .ok_or_else(|| AsJsonError::InvalidFieldValue("resources[].variants[].status"))?;

        let hashes = json.get("hashes")
            .ok_or_else(|| AsJsonError::FieldNotFound("resources[].variants[].hashes"))?
            .as_array()
            .and_then(|hashes| {
                hashes.iter()
                    .map(|hash| hash.as_str().and_then(Hash::from_base32))
                    .collect::<Option<HashSet<_>>>()
            })
            .ok_or_else(|| AsJsonError::InvalidFieldValue("resources[].variants[].hashes"))?;

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
                        }),

                    hashes
                })
            }

            "compromised" => Ok(Self::Compromised {
                details_url: json.get("details_url")
                    .and_then(Json::as_str)
                    .map(String::from),

                hashes
            }),

            "malicious" => Ok(Self::Malicious {
                details_url: json.get("details_url")
                    .and_then(Json::as_str)
                    .map(String::from),

                hashes
            }),

            _ => Err(AsJsonError::InvalidFieldValue("resources[].variants[].status"))
        }
    }
}
