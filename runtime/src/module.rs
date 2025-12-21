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

use std::path::PathBuf;

use serde_json::{json, Value as Json};

/// A luau module description.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Module {
    /// Path to the luau module file.
    pub path: PathBuf,

    /// Luau module permissions.
    pub scope: ModuleScope
}

/// Luau module permissions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModuleScope {
    /// Allow module to access string API.
    ///
    /// This API allows module to perform conversions between different string
    /// encodings (UTF-8, ASCII, etc.) and formats (hex, base64, JSON, etc.).
    ///
    /// Default: `true`.
    pub allow_string_api: bool,

    /// Allow module to access path API.
    ///
    /// This API allows module to combine different path parts, normalize and
    /// resolve them, check if files or folders exist and if they're accessible.
    ///
    /// Default: `true`.
    pub allow_path_api: bool,

    /// Allow module to access filesystem API.
    ///
    /// This API allows module to perform read/write/create operations on files
    /// and folders of the host filesystem, with sandboxed access to only
    /// allowed files and folders.
    ///
    /// Default: `true`.
    pub allow_fs_api: bool,

    /// Allow module to access network API.
    ///
    /// This API allows module to perform HTTP requests.
    ///
    /// Default: `true`.
    pub allow_network_api: bool,

    /// Allow module to access downloader API.
    ///
    /// This API allows module to download files from HTTP servers. Similar to
    /// the Network API, except it has more user niceness in it.
    ///
    /// Default: `true`.
    pub allow_downloader_api: bool,

    /// Allow module to access archive API.
    ///
    /// This API allows module to extract archives or list their info.
    ///
    /// Default: `true`.
    pub allow_archive_api: bool,

    /// Allow module to access hash API.
    ///
    /// This API allows module to calculate different hashes of files or
    /// folders.
    ///
    /// Default: `true`.
    pub allow_hash_api: bool,

    /// Allow module to access compression API.
    ///
    /// This API allows module to compress or decompress data with different
    /// compression algorithms.
    ///
    /// Default: `true`.
    pub allow_compression_api: bool,

    /// Allow module to access sqlite API.
    ///
    /// This API allows module to work with a sqlite database.
    ///
    /// Default: `true`.
    pub allow_sqlite_api: bool,

    /// Allow module to access process API.
    ///
    /// This API allows module to spawn and control new processes on the host
    /// system.
    ///
    /// > **Security warning:** This API can be used to escape the sandbox. You
    /// > must make sure that the module *really* needs this API.
    ///
    /// Default: `false`.
    pub allow_process_api: bool,

    /// Paths allowed to be accessed for this module. When provided, the module
    /// can use filesystem and other APIs to read provided files or
    /// folders/subfolders.
    ///
    /// Default: none.
    pub sandbox_read_paths: Vec<PathBuf>,

    /// Paths allowed to be written and read by this module. When provided, the
    /// module can use filesystem and other APIs to read and write provided
    /// files or folders/subfolders.
    ///
    /// Default: none.
    pub sandbox_write_paths: Vec<PathBuf>
}

impl Default for ModuleScope {
    fn default() -> Self {
        Self {
            allow_string_api: true,
            allow_path_api: true,
            allow_fs_api: true,
            allow_network_api: true,
            allow_downloader_api: true,
            allow_archive_api: true,
            allow_hash_api: true,
            allow_compression_api: true,
            allow_sqlite_api: true,
            allow_process_api: false,
            sandbox_read_paths: vec![],
            sandbox_write_paths: vec![]
        }
    }
}

impl ModuleScope {
    pub fn to_json(&self) -> Json {
        json!({
            "allow_api": {
                "string": self.allow_string_api,
                "path": self.allow_path_api,
                "fs": self.allow_fs_api,
                "network": self.allow_network_api,
                "downloader": self.allow_downloader_api,
                "archive": self.allow_archive_api,
                "hash": self.allow_hash_api,
                "compression": self.allow_compression_api,
                "sqlite": self.allow_sqlite_api,
                "process": self.allow_process_api
            },
            "sandbox": {
                "read_paths": self.sandbox_read_paths,
                "write_paths": self.sandbox_write_paths
            }
        })
    }

    pub fn from_json(value: &Json) -> Self {
        let mut scope = Self::default();

        if let Some(allow_api) = value.get("allow_api") {
            if let Some(allow) = allow_api.get("string").and_then(Json::as_bool) {
                scope.allow_string_api = allow;
            }

            if let Some(allow) = allow_api.get("path").and_then(Json::as_bool) {
                scope.allow_path_api = allow;
            }

            if let Some(allow) = allow_api.get("fs").and_then(Json::as_bool) {
                scope.allow_fs_api = allow;
            }

            if let Some(allow) = allow_api.get("network").and_then(Json::as_bool) {
                scope.allow_network_api = allow;
            }

            if let Some(allow) = allow_api.get("downloader").and_then(Json::as_bool) {
                scope.allow_downloader_api = allow;
            }

            if let Some(allow) = allow_api.get("archive").and_then(Json::as_bool) {
                scope.allow_archive_api = allow;
            }

            if let Some(allow) = allow_api.get("hash").and_then(Json::as_bool) {
                scope.allow_hash_api = allow;
            }

            if let Some(allow) = allow_api.get("compression").and_then(Json::as_bool) {
                scope.allow_compression_api = allow;
            }

            if let Some(allow) = allow_api.get("sqlite").and_then(Json::as_bool) {
                scope.allow_sqlite_api = allow;
            }

            if let Some(allow) = allow_api.get("process").and_then(Json::as_bool) {
                scope.allow_process_api = allow;
            }
        }

        if let Some(sandbox) = value.get("sandbox") {
            if let Some(read_paths) = sandbox.get("read_paths").and_then(Json::as_array) {
                scope.sandbox_read_paths = read_paths.iter()
                    .flat_map(Json::as_str)
                    .map(PathBuf::from)
                    .collect();
            }

            if let Some(write_paths) = sandbox.get("write_paths").and_then(Json::as_array) {
                scope.sandbox_write_paths = write_paths.iter()
                    .flat_map(Json::as_str)
                    .map(PathBuf::from)
                    .collect();
            }
        }

        scope
    }
}
