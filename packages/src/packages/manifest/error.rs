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

#[derive(Debug, Clone, thiserror::Error)]
pub enum PackageManifestError {
    #[error("unknown resource format: {0}")]
    ResourceUnknownFormat(String),

    #[error("unknown resource module format: {0}")]
    ResourceUnknownModuleFormat(String),

    #[error("unknown resource archive format: {0}")]
    ResourceUnknownArchiveFormat(String),

    #[error("resource is missing uri field")]
    ResourceMissingUri,

    #[error("invalid resource hash format: {0}")]
    ResourceInvalidHashFormat(String),

    #[error("unknown package format version: {0}")]
    PackageUnknownFormatVersion(u16),

    #[error("invalid package manifest field '{field}' format: expected '{expected}'")]
    PackageInvalidFieldFormat {
        field: &'static str,
        expected: &'static str
    }
}
