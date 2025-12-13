// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-games
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

use serde_json::Value as Json;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PackageInfoDeserializeError {
    #[error("package URL is not specified")]
    MissingUrl,

    #[error("package output is not specified")]
    MissingOutput
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageInfo {
    pub url: String,
    pub output: String
}

impl PackageInfo {
    pub fn from_json(
        value: &Json
    ) -> Result<Self, PackageInfoDeserializeError> {
        Ok(Self {
            url: value.get("url")
                .and_then(Json::as_str)
                .ok_or(PackageInfoDeserializeError::MissingUrl)?
                .to_string(),

            output: value.get("output")
                .and_then(Json::as_str)
                .ok_or(PackageInfoDeserializeError::MissingOutput)?
                .to_string()
        })
    }
}
