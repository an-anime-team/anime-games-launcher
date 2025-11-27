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
    /// Allow module to access String API.
    ///
    /// This API allows module to perform conversions between different string
    /// encodings (UTF-8, ASCII, etc.) and formats (hex, base64, JSON, etc.).
    ///
    /// Default: `true`.
    pub allow_string_api: bool,

    /// Allow module to access Path API.
    ///
    /// This API allows module to combine different path parts, normalize and
    /// resolve them, check if files or folders exist and if they're accessible.
    ///
    /// Default: `true`.
    pub allow_path_api: bool,

    /// Allow module to access basic Filesystem API.
    ///
    /// This API allows module to perform read/write/create operations on files
    /// and folders of the host filesystem, with sandboxed access to only
    /// allowed files and folders.
    ///
    /// Default: `true`.
    pub allow_basic_fs_api: bool,

    /// Allow module to access network API.
    ///
    /// This API allows module to perform HTTP requests.
    ///
    /// Default: `true`.
    pub allow_network_api: bool,

    /// Allow module to access Downloader API.
    ///
    /// This API allows module to download files from HTTP servers. Similar to
    /// the Network API, except it has more user niceness in it.
    ///
    /// Default: `true`.
    pub allow_downloader_api: bool,

    /// Allow module to access Archives API.
    ///
    /// This API allows module to extract archives or list their info.
    ///
    /// Default: `true`.
    pub allow_archives_api: bool,

    /// Allow module to access Hashes API.
    ///
    /// This API allows module to calculate different hashes of files or
    /// folders.
    ///
    /// Default: `true`.
    pub allow_hashes_api: bool,

    /// Allow module to access Compression API.
    ///
    /// This API allows module to compress or decompress data with different
    /// compression algorithms.
    ///
    /// Default: `true`.
    pub allow_compression_api: bool,

    /// Allow module to access Sqlite API.
    ///
    /// This API allows module to work with a sqlite database.
    ///
    /// Default: `true`.
    pub allow_sqlite_api: bool,

    /// Allow module to access Process API.
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
    /// can use Filesystem and other APIs to read or write into provided files
    /// or folders/subfolders.
    ///
    /// Default: none.
    pub sandbox_allowed_paths: Vec<PathBuf>
}

impl Default for ModuleScope {
    fn default() -> Self {
        Self {
            allow_string_api: true,
            allow_path_api: true,
            allow_basic_fs_api: true,
            allow_network_api: true,
            allow_downloader_api: true,
            allow_archives_api: true,
            allow_hashes_api: true,
            allow_compression_api: true,
            allow_sqlite_api: true,
            allow_process_api: false,
            sandbox_allowed_paths: vec![]
        }
    }
}
