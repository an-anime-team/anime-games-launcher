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

use std::collections::HashMap;

use serde_json::{json, Value as Json};

use agl_packages::hash::Hash;

use super::module::ModuleScope;

/// List of scopes for luau modules.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct AllowList(HashMap<Hash, ModuleScope>);

impl AllowList {
    pub fn to_json(&self) -> Json {
        json!(self.0.iter()
            .map(|(k, v)| (k.to_base32(), v.to_json()))
            .collect::<HashMap<_, _>>())
    }

    pub fn from_json(value: &Json) -> Option<Self> {
        let mut scopes = HashMap::new();

        for (hash, scope) in value.as_object()? {
            let hash = Hash::from_base32(hash)?;
            let scope = ModuleScope::from_json(scope);

            scopes.insert(hash, scope);
        }

        Some(Self(scopes))
    }

    #[inline]
    pub const fn scopes(&self) -> &HashMap<Hash, ModuleScope> {
        &self.0
    }

    /// Merge current allow list with provided one.
    pub fn merge_with(&mut self, another_list: Self) {
        for (hash, scope) in another_list.0 {
            self.add_module_scope(hash, scope);
        }
    }

    /// Add module scope to the allow list, merging it with existing one if it
    /// is available.
    pub fn add_module_scope(&mut self, hash: Hash, scope: ModuleScope) {
        let entry = self.0.entry(hash)
            .or_default();

        entry.allow_string_api      |= scope.allow_string_api;
        entry.allow_path_api        |= scope.allow_path_api;
        entry.allow_task_api        |= scope.allow_task_api;
        entry.allow_filesystem_api  |= scope.allow_filesystem_api;
        entry.allow_http_api        |= scope.allow_http_api;
        entry.allow_downloader_api  |= scope.allow_downloader_api;
        entry.allow_archive_api     |= scope.allow_archive_api;
        entry.allow_hash_api        |= scope.allow_hash_api;
        entry.allow_compression_api |= scope.allow_compression_api;
        entry.allow_sqlite_api      |= scope.allow_sqlite_api;
        entry.allow_torrent_api     |= scope.allow_torrent_api;
        entry.allow_portal_api      |= scope.allow_portal_api;
        entry.allow_process_api     |= scope.allow_process_api;

        entry.sandbox_read_paths.extend(scope.sandbox_read_paths);
        entry.sandbox_write_paths.extend(scope.sandbox_write_paths);

        entry.sandbox_read_paths.dedup();
        entry.sandbox_write_paths.dedup();
    }

    /// Try to get scope for a module with provided hash if it's stored in the
    /// allow list.
    #[inline]
    pub fn get_module_scope(&self, hash: &Hash) -> Option<&ModuleScope> {
        self.0.get(hash)
    }
}
