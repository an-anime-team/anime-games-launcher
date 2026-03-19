// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-runtime
// Copyright (C) 2025 - 2026  Nikita Podvirnyi <krypt0nn@vk.com>
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

use mlua::prelude::*;

use agl_core::export::tasks::tokio;
use agl_core::export::network::reqwest;
use agl_core::network::downloader::Downloader;
use agl_locale::string::LocalizableString;

#[cfg(feature = "packages-support")]
use agl_packages::storage::Storage;

#[cfg(feature = "packages-support")]
use crate::allow_list::AllowList;

use crate::module::{Module, ModuleScope};
use crate::api::ApiOptions;
use crate::runtime::{Runtime, RuntimeError, ModulePaths};

#[cfg(feature = "packages-support")]
const TESTS_DIR_URL: &str = "https://github.com/an-anime-team/anime-games-launcher/raw/refs/heads/next/crates/agl-runtime/tests";
// const TESTS_DIR_URL: &str = "http://127.0.0.1:8080";

fn get_test_dir(name: &str) -> std::io::Result<PathBuf> {
    let path = std::env::temp_dir()
        .join(".agl-runtime-test")
        .join(name);

    if path.exists() {
        std::fs::remove_dir_all(&path)?;
    }

    std::fs::create_dir_all(&path)?;

    Ok(path)
}

fn get_runtime() -> Result<Runtime, RuntimeError> {
    fn translate(str: LocalizableString) -> String {
        str.default_translation().to_string()
    }

    Runtime::new(ApiOptions {
        lua: Lua::new(),
        reqwest_client: reqwest::Client::new(),

        #[cfg(feature = "torrent-api")]
        torrent_server: None,

        #[cfg(feature = "portal-api")]
        show_toast: Box::new(|_| {}),

        #[cfg(feature = "portal-api")]
        show_notification: Box::new(|_| {}),

        #[cfg(feature = "portal-api")]
        show_dialog: Box::new(|_| {}),

        translate
    })
}

#[test]
fn simple_module() -> Result<(), RuntimeError> {
    let runtime = get_runtime()?;

    let module = Module {
        path: PathBuf::from("tests/simple_module/module.luau"),
        scope: ModuleScope::default()
    };

    let paths = ModulePaths {
        temp_folder: std::env::temp_dir(),
        modules_folder: std::env::temp_dir(),
        persistent_folder: std::env::temp_dir()
    };

    runtime.load_module("module", module, paths)?;

    let Some(module) = runtime.get_value::<LuaFunction>("module")? else {
        panic!("missing loaded module value");
    };

    runtime.set_value("test_1", "Amogus")?;
    runtime.set_value("test_2", "Sugoma")?;

    runtime.set_named_reference("module", "test_1", "name")?;

    assert_eq!(module.call::<String>(())?, "Hello, Amogus!");

    runtime.set_named_reference("module", "test_2", "name")?;

    assert_eq!(module.call::<String>(())?, "Hello, Sugoma!");

    Ok(())
}

#[cfg(feature = "packages-support")]
#[tokio::test]
async fn simple_package() -> Result<(), Box<dyn std::error::Error>> {
    let downloader = Downloader::default();
    let storage = Storage::open(get_test_dir("simple_package")?)?;

    let lock = storage.install_packages(&downloader, [
        format!("{TESTS_DIR_URL}/simple_package/package.json")
    ]).await?;

    let runtime = get_runtime()?;

    let paths = ModulePaths {
        temp_folder: std::env::temp_dir(),
        modules_folder: std::env::temp_dir(),
        persistent_folder: std::env::temp_dir()
    };

    let allow_list = AllowList::default();

    runtime.load_packages(&lock, &storage, &paths, &allow_list)?;

    // Find some better and standardized way for querying loaded modules.
    let Some(module) = runtime.get_value::<LuaTable>("p9ffktad8ns1g#module")? else {
        panic!("missing loaded module value");
    };

    let module = module.raw_get::<LuaFunction>("value")?;

    assert_eq!(module.call::<String>(())?, "Hello, World!\n");

    Ok(())
}

#[cfg(feature = "packages-support")]
#[tokio::test]
async fn dependency_module() -> Result<(), Box<dyn std::error::Error>> {
    let downloader = Downloader::default();
    let storage = Storage::open(get_test_dir("dependency_module")?)?;

    let lock = storage.install_packages(&downloader, [
        format!("{TESTS_DIR_URL}/dependency_module/package.json")
    ]).await?;

    let runtime = get_runtime()?;

    let paths = ModulePaths {
        temp_folder: std::env::temp_dir(),
        modules_folder: std::env::temp_dir(),
        persistent_folder: std::env::temp_dir()
    };

    let allow_list = AllowList::default();

    runtime.load_packages(&lock, &storage, &paths, &allow_list)?;

    // Find some better and standardized way for querying loaded modules.
    let Some(module) = runtime.get_value::<LuaTable>("4rrnaukmvtkl4#module")? else {
        panic!("missing loaded module value");
    };

    let module = module.raw_get::<LuaFunction>("value")?;

    runtime.set_value("test", "World")?;
    runtime.set_named_reference("hlm1n2jp72hbg#module", "test", "name")?;

    assert_eq!(module.call::<String>(())?, "Hello, World!");

    Ok(())
}

#[cfg(feature = "packages-support")]
#[tokio::test]
async fn nested_package() -> Result<(), Box<dyn std::error::Error>> {
    let downloader = Downloader::default();
    let storage = Storage::open(get_test_dir("nested_package")?)?;

    let lock = storage.install_packages(&downloader, [
        format!("{TESTS_DIR_URL}/nested_package/package_1.json"),
        format!("{TESTS_DIR_URL}/nested_package/package_2.json")
    ]).await?;

    let runtime = get_runtime()?;

    let paths = ModulePaths {
        temp_folder: std::env::temp_dir(),
        modules_folder: std::env::temp_dir(),
        persistent_folder: std::env::temp_dir()
    };

    let allow_list = AllowList::default();

    runtime.load_packages(&lock, &storage, &paths, &allow_list)?;

    // Find some better and standardized way for querying loaded modules.
    let Some(module) = runtime.get_value::<LuaTable>("op5h5fuc7kqr4#module")? else {
        panic!("missing loaded module value");
    };

    let module = module.raw_get::<LuaFunction>("value")?;

    assert_eq!(module.call::<String>(())?, "Counter: 1");
    assert_eq!(module.call::<String>(())?, "Counter: 2");
    assert_eq!(module.call::<String>(())?, "Counter: 3");

    Ok(())
}
