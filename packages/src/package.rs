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

use std::collections::HashMap;
use std::str::FromStr;

use serde_json::{json, Value as Json};

use crate::hash::Hash;

/// Anime Games Launcher package manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    /// Table of the package's inputs.
    ///
    /// `[name] => [resource]`
    pub inputs: HashMap<String, ResourceInfoManifest>,

    /// Table of the package's outputs.
    ///
    /// `[name] => [resource]`
    pub outputs: HashMap<String, ResourceInfoManifest>
}

impl Manifest {
    pub fn to_json(&self) -> Json {
        json!({
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
            inputs: value.get("inputs")
                .and_then(Json::as_object)?
                .iter()
                .map(|(k, v)| {
                    ResourceInfoManifest::from_json(v)
                        .map(|v| (k.to_string(), v))
                })
                .collect::<Option<HashMap<_, _>>>()?,

            outputs: value.get("outputs")
                .and_then(Json::as_object)?
                .iter()
                .map(|(k, v)| {
                    ResourceInfoManifest::from_json(v)
                        .map(|v| (k.to_string(), v))
                })
                .collect::<Option<HashMap<_, _>>>()?
        })
    }
}

/// Information about an input/output resource of an Anime Games Launcher
/// package.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceInfoManifest {
    /// URI of the resource.
    pub uri: String,

    /// Format of the resource.
    pub format: Option<ResourceFormat>,

    /// (optional) base32 seahash of the resource.
    pub hash: Option<Hash>
}

impl ResourceInfoManifest {
    pub fn to_json(&self) -> Json {
        match (self.format.as_ref(), self.hash.as_ref()) {
            (Some(format), Some(hash)) => json!({
                "uri": self.uri,
                "format": format.to_string(),
                "hash": hash.to_base32()
            }),

            (Some(format), None) => json!({
                "uri": self.uri,
                "format": format.to_string()
            }),

            (None, Some(hash)) => json!({
                "uri": self.uri,
                "hash": hash.to_base32()
            }),

            (None, None) => json!(self.uri)
        }
    }

    pub fn from_json(value: &Json) -> Option<Self> {
        if let Some(uri) = value.as_str() {
            return Some(Self {
                uri: uri.to_string(),
                format: None,
                hash: None
            });
        }

        Some(Self {
            uri: value.get("uri")
                .and_then(Json::as_str)
                .map(String::from)?,

            format: value.get("format")
                .and_then(Json::as_str)
                .and_then(|format| ResourceFormat::from_str(format).ok()),

            hash: value.get("hash")
                .and_then(Json::as_str)
                .and_then(Hash::from_base32)
        })
    }
}

/// Format of an Anime Games Launcher package's input/output resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceFormat {
    /// Anime Games Launcher package manifest.
    Package,

    /// Arbitrary file.
    File,

    /// Arbitrary archive.
    Archive
}

impl std::fmt::Display for ResourceFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Package => f.write_str("package"),
            Self::File    => f.write_str("file"),
            Self::Archive => f.write_str("archive")
        }
    }
}

impl FromStr for ResourceFormat {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "package" => Ok(Self::Package),
            "file"    => Ok(Self::File),
            "archive" => Ok(Self::Archive),

            _ => Err(())
        }
    }
}
