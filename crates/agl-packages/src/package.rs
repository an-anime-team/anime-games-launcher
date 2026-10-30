// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-packages
// Copyright (C) 2025 - 2026  Nikita Podvirnyi <krypt0nn@dawn.wine>
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
use crate::format::ResourceFormat;

/// Anime Games Launcher package manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageManifest {
    /// Table of the package's inputs.
    ///
    /// `[name] => [resource]`
    pub inputs: HashMap<String, ResourceInfoManifest>,

    /// Table of the package's outputs.
    ///
    /// `[name] => [resource]`
    pub outputs: HashMap<String, ResourceInfoManifest>
}

impl PackageManifest {
    pub fn to_json(&self) -> Json {
        json!({
            "format": 1,

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
                .and_then(Json::as_object)
                .map(|inputs| {
                    inputs.iter()
                        .map(|(k, v)| {
                            ResourceInfoManifest::from_json(v)
                                .map(|v| (k.to_string(), v))
                        })
                        .collect::<Option<HashMap<_, _>>>()
                })
                // First `None` is if `inputs` is missing, second `None` if
                // we couldn't build the `ResourceInfoManifest`, thus error.
                .unwrap_or_else(|| Some(HashMap::new()))?,

            outputs: value.get("outputs")
                .and_then(Json::as_object)
                .map(|outputs| {
                    outputs.iter()
                        .map(|(k, v)| {
                            ResourceInfoManifest::from_json(v)
                                .map(|v| (k.to_string(), v))
                        })
                        .collect::<Option<HashMap<_, _>>>()
                })
                // First `None` is if `outputs` is missing, second `None` if
                // we couldn't build the `ResourceInfoManifest`, thus error.
                .unwrap_or_else(|| Some(HashMap::new()))?
        })
    }
}

/// Information about an input/output resource of an Anime Games Launcher
/// package.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceInfoManifest {
    /// URI of the resource. If multiple provided, they will be fetched in
    /// sequential order until one responds with valid content.
    pub uri: Vec<String>,

    /// Format of the resource.
    pub format: Option<ResourceFormat>,

    /// (optional) base32 seahash of the resource.
    pub hash: Option<Hash>
}

impl ResourceInfoManifest {
    pub fn to_json(&self) -> Json {
        let uri = match self.uri.len() {
            0 => json!([]),
            1 => json!(&self.uri[0]),
            _ => json!(&self.uri)
        };

        match (self.format.as_ref(), self.hash.as_ref()) {
            (Some(format), Some(hash)) => json!({
                "uri": uri,
                "format": format.to_string(),
                "hash": hash.to_base32()
            }),

            (Some(format), None) => json!({
                "uri": uri,
                "format": format.to_string()
            }),

            (None, Some(hash)) => json!({
                "uri": uri,
                "hash": hash.to_base32()
            }),

            (None, None) => uri
        }
    }

    pub fn from_json(value: &Json) -> Option<Self> {
        match value {
            Json::String(uri) => Some(Self {
                uri: vec![uri.clone()],
                format: None,
                hash: None
            }),

            Json::Array(uris) => Some(Self {
                uri: uris.iter()
                    .flat_map(|uri| uri.as_str())
                    .map(String::from)
                    .collect(),
                format: None,
                hash: None
            }),

            Json::Object(package) => {
                Some(Self {
                    uri: package.get("uri")
                        .and_then(|uri| {
                            match uri {
                                Json::String(uri) => Some(vec![uri.clone()]),
                                Json::Array(uris) => Some(uris.iter()
                                    .flat_map(|uri| uri.as_str())
                                    .map(String::from)
                                    .collect()),
                                _ => None
                            }
                        })?,

                    format: package.get("format")
                        .and_then(Json::as_str)
                        .and_then(|format| ResourceFormat::from_str(format).ok()),

                    hash: package.get("hash")
                        .and_then(Json::as_str)
                        .and_then(Hash::from_base32)
                })
            }

            _ => None
        }
    }
}
