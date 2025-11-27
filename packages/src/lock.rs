// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-packages
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

use std::collections::{HashSet, HashMap};
use std::str::FromStr;

use serde_json::{json, Value as Json};

use crate::hash::Hash;
use crate::package::ResourceFormat;

/// Anime Games Launcher packages lock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lock {
    /// List of the root level packages hashes.
    pub root: HashSet<Hash>,

    /// Table of locked packages info.
    pub packages: HashMap<Hash, LockedPackageInfo>,

    /// Table of locked resources info.
    pub resources: HashMap<Hash, String>
}

impl Lock {
    /// Verify the lock structure.
    ///
    /// Return `false` if the lock is missing some packages or resource info.
    /// Otherwise return `true`.
    pub fn verify(&self) -> bool {
        // TODO: check that there's no module which has same-called input for
        //       different resource hashes

        for hash in self.root.iter() {
            let Some(info) = self.packages.get(hash) else {
                return false;
            };

            for resource in info.inputs.values() {
                if !self.resources.contains_key(&resource.hash) {
                    return false;
                }
            }

            for resource in info.outputs.values() {
                if !self.resources.contains_key(&resource.hash) {
                    return false;
                }
            }
        }

        true
    }

    pub fn to_json(&self) -> Json {
        json!({
            "root": self.root.iter()
                .map(Hash::to_base32)
                .collect::<Vec<String>>(),

            "packages": self.packages.iter()
                .map(|(k, v)| (k.to_base32(), v.to_json()))
                .collect::<HashMap<String, Json>>(),

            "resources": self.resources.iter()
                .map(|(k, v)| (k.to_base32(), v.to_string()))
                .collect::<HashMap<String, String>>()
        })
    }

    pub fn from_json(value: &Json) -> Option<Self> {
        Some(Self {
            root: value.get("root")
                .and_then(Json::as_array)
                .and_then(|root| {
                    root.iter()
                        .map(|hash| {
                            hash.as_str()
                                .and_then(Hash::from_base32)
                        })
                        .collect::<Option<HashSet<Hash>>>()
                })?,

            packages: value.get("packages")
                .and_then(Json::as_object)
                .and_then(|packages| {
                    packages.iter()
                        .map(|(k, v)| {
                            let k = Hash::from_base32(k)?;
                            let v = LockedPackageInfo::from_json(v)?;

                            Some((k, v))
                        })
                        .collect::<Option<HashMap<Hash, LockedPackageInfo>>>()
                })?,

            resources: value.get("resources")
                .and_then(Json::as_object)
                .and_then(|resources| {
                    resources.iter()
                        .map(|(k, v)| {
                            let k = Hash::from_base32(k)?;
                            let v = v.as_str()?;

                            Some((k, v.to_string()))
                        })
                        .collect::<Option<HashMap<Hash, String>>>()
                })?
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockedPackageInfo {
    /// Original package manifest URL.
    pub url: String,

    /// Inputs of the package.
    pub inputs: HashMap<String, LockedResourceInfo>,

    /// Outputs of the package.
    pub outputs: HashMap<String, LockedResourceInfo>
}

impl LockedPackageInfo {
    pub fn to_json(&self) -> Json {
        json!({
            "url": self.url,

            "inputs": self.inputs.iter()
                .map(|(k, v)| (k.to_string(), v.to_json()))
                .collect::<HashMap<String, Json>>(),

            "outputs": self.outputs.iter()
                .map(|(k, v)| (k.to_string(), v.to_json()))
                .collect::<HashMap<String, Json>>()
        })
    }

    pub fn from_json(value: &Json) -> Option<Self> {
        Some(Self {
            url: value.get("url")
                .and_then(Json::as_str)
                .map(String::from)?,

            inputs: value.get("inputs")
                .and_then(Json::as_object)?
                .iter()
                .map(|(k, v)| {
                    LockedResourceInfo::from_json(v)
                        .map(|v| (k.to_string(), v))
                })
                .collect::<Option<HashMap<_, _>>>()?,

            outputs: value.get("outputs")
                .and_then(Json::as_object)?
                .iter()
                .map(|(k, v)| {
                    LockedResourceInfo::from_json(v)
                        .map(|v| (k.to_string(), v))
                })
                .collect::<Option<HashMap<_, _>>>()?
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LockedResourceInfo {
    /// URI of the resource.
    pub uri: String,

    /// Format of the resource.
    pub format: ResourceFormat,

    /// Base32 seahash of the resource.
    pub hash: Hash
}

impl LockedResourceInfo {
    pub fn to_json(&self) -> Json {
        json!({
            "uri": self.uri,
            "format": self.format.to_string(),
            "hash": self.hash.to_base32()
        })
    }

    pub fn from_json(value: &Json) -> Option<Self> {
        Some(Self {
            uri: value.get("uri")
                .and_then(Json::as_str)
                .map(String::from)?,

            format: value.get("format")
                .and_then(Json::as_str)
                .and_then(|format| ResourceFormat::from_str(format).ok())?,

            hash: value.get("hash")
                .and_then(Json::as_str)
                .and_then(Hash::from_base32)?
        })
    }
}
