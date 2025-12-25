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

use std::path::PathBuf;

use crate::format::ResourceFormat;
use crate::hash::Hash;
use crate::storage::Storage;

use agl_core::export::tasks::tokio;
use agl_core::network::downloader::Downloader;

const TESTS_DIR_URL: &str = "https://github.com/an-anime-team/anime-games-launcher/raw/refs/heads/next/crates/agl-packages/tests";
// const TESTS_DIR_URL: &str = "http://127.0.0.1:8080";

fn get_test_dir(name: &str) -> std::io::Result<PathBuf> {
    let path = std::env::temp_dir()
        .join(".agl-packages-test")
        .join(name);

    if path.exists() {
        std::fs::remove_dir_all(&path)?;
    }

    std::fs::create_dir_all(&path)?;

    Ok(path)
}

#[tokio::test]
async fn simple_no_inputs() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_url = format!("{TESTS_DIR_URL}/simple_no_inputs/package.json");

    let manifest_hash = Hash::from_base32("0b5s62guc7us2").unwrap();

    let resources = [
        ("example_file",     format!("{TESTS_DIR_URL}/simple_no_inputs/example_file.txt"),          Hash::from_base32("dfhtkkli693ji").unwrap(), ResourceFormat::File),
        ("example_tar",      format!("{TESTS_DIR_URL}/simple_no_inputs/example_tar.tar"),           Hash::from_base32("bfcut078nb5sq").unwrap(), ResourceFormat::Archive),
        ("example_tar_gz",   format!("{TESTS_DIR_URL}/simple_no_inputs/example_tar_gz.tar.gz"),     Hash::from_base32("nuasi909r9ek2").unwrap(), ResourceFormat::Archive),
        ("example_tar_bz2",  format!("{TESTS_DIR_URL}/simple_no_inputs/example_tar_bz2.tar.bz2"),   Hash::from_base32("rl09ekeb9s9sm").unwrap(), ResourceFormat::Archive),
        ("example_tar_zstd", format!("{TESTS_DIR_URL}/simple_no_inputs/example_tar_zstd.tar.zstd"), Hash::from_base32("4ib8sfl2v57te").unwrap(), ResourceFormat::Archive),
        ("example_zip",      format!("{TESTS_DIR_URL}/simple_no_inputs/example_zip.zip"),           Hash::from_base32("s4lst5543nd1k").unwrap(), ResourceFormat::Archive),
        ("example_7z",       format!("{TESTS_DIR_URL}/simple_no_inputs/example_7z.7z"),             Hash::from_base32("i8bois3gmu8mk").unwrap(), ResourceFormat::Archive)
    ];

    let downloader = Downloader::default();
    let storage = Storage::open(get_test_dir("simple_no_inputs")?)?;

    let lock = storage.install_packages(&downloader, [
        manifest_url.clone()
    ]).await?;

    assert!(lock.verify());
    assert!(storage.verify_lock(&lock)?);

    assert!(lock.root.iter().all(|root| root == &manifest_hash));
    assert_eq!(lock.packages.len(), 1);
    assert_eq!(lock.resources.len(), resources.len());

    for (_, url, hash, _) in &resources {
        assert_eq!(lock.resources.get(hash), Some(url));
    }

    let Some(package_info) = lock.packages.get(&manifest_hash) else {
        return Err("missing package info".into());
    };

    assert_eq!(package_info.url, manifest_url);
    assert!(package_info.inputs.is_empty());
    assert_eq!(package_info.outputs.len(), resources.len());

    for (name, url, hash, format) in &resources {
        let Some(resource_info) = package_info.outputs.get(*name) else {
            return Err(format!("missing resource '{name}' info").into());
        };

        assert_eq!(resource_info.url, *url);
        assert_eq!(resource_info.format, *format);
        assert_eq!(resource_info.hash, *hash);
    }

    Ok(())
}

#[tokio::test]
async fn simple_no_outputs() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_url = format!("{TESTS_DIR_URL}/simple_no_outputs/package.json");

    let manifest_hash = Hash::from_base32("unol2o7d19fl6").unwrap();

    let resources = [
        ("example_file",     format!("{TESTS_DIR_URL}/simple_no_outputs/example_file.txt"),          Hash::from_base32("dfhtkkli693ji").unwrap(), ResourceFormat::File),
        ("example_tar",      format!("{TESTS_DIR_URL}/simple_no_outputs/example_tar.tar"),           Hash::from_base32("bfcut078nb5sq").unwrap(), ResourceFormat::Archive),
        ("example_tar_gz",   format!("{TESTS_DIR_URL}/simple_no_outputs/example_tar_gz.tar.gz"),     Hash::from_base32("nuasi909r9ek2").unwrap(), ResourceFormat::Archive),
        ("example_tar_bz2",  format!("{TESTS_DIR_URL}/simple_no_outputs/example_tar_bz2.tar.bz2"),   Hash::from_base32("rl09ekeb9s9sm").unwrap(), ResourceFormat::Archive),
        ("example_tar_zstd", format!("{TESTS_DIR_URL}/simple_no_outputs/example_tar_zstd.tar.zstd"), Hash::from_base32("4ib8sfl2v57te").unwrap(), ResourceFormat::Archive),
        ("example_zip",      format!("{TESTS_DIR_URL}/simple_no_outputs/example_zip.zip"),           Hash::from_base32("s4lst5543nd1k").unwrap(), ResourceFormat::Archive),
        ("example_7z",       format!("{TESTS_DIR_URL}/simple_no_outputs/example_7z.7z"),             Hash::from_base32("i8bois3gmu8mk").unwrap(), ResourceFormat::Archive)
    ];

    let downloader = Downloader::default();
    let storage = Storage::open(get_test_dir("simple_no_outputs")?)?;

    let lock = storage.install_packages(&downloader, [
        manifest_url.clone()
    ]).await?;

    assert!(lock.verify());
    assert!(storage.verify_lock(&lock)?);

    assert!(lock.root.iter().all(|root| root == &manifest_hash));
    assert_eq!(lock.packages.len(), 1);
    assert_eq!(lock.resources.len(), resources.len());

    for (_, url, hash, _) in &resources {
        assert_eq!(lock.resources.get(hash), Some(url));
    }

    let Some(package_info) = lock.packages.get(&manifest_hash) else {
        return Err("missing package info".into());
    };

    assert_eq!(package_info.url, manifest_url);
    assert_eq!(package_info.inputs.len(), resources.len());
    assert!(package_info.outputs.is_empty());

    for (name, url, hash, format) in &resources {
        let Some(resource_info) = package_info.inputs.get(*name) else {
            return Err(format!("missing resource '{name}' info").into());
        };

        assert_eq!(resource_info.url, *url);
        assert_eq!(resource_info.format, *format);
        assert_eq!(resource_info.hash, *hash);
    }

    Ok(())
}

#[tokio::test]
async fn duplicate_input_output() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_url = format!("{TESTS_DIR_URL}/duplicate_input_output/package.json");
    let file_url = format!("{TESTS_DIR_URL}/duplicate_input_output/example_file.txt");

    let manifest_hash = Hash::from_base32("ukkgn5btkiu8s").unwrap();
    let file_hash = Hash::from_base32("dfhtkkli693ji").unwrap();

    let downloader = Downloader::default();
    let storage = Storage::open(get_test_dir("duplicate_input_output")?)?;

    let lock = storage.install_packages(&downloader, [
        manifest_url.clone()
    ]).await?;

    assert!(lock.verify());
    assert!(storage.verify_lock(&lock)?);

    assert!(lock.root.iter().all(|root| root == &manifest_hash));
    assert_eq!(lock.packages.len(), 1);
    assert_eq!(lock.resources.len(), 1);

    assert_eq!(lock.resources.get(&file_hash), Some(&file_url));

    let Some(package_info) = lock.packages.get(&manifest_hash) else {
        return Err("missing package info".into());
    };

    assert_eq!(package_info.url, manifest_url);
    assert_eq!(package_info.inputs.len(), 1);
    assert_eq!(package_info.outputs.len(), 1);

    let Some(resource_info) = package_info.inputs.get("example_file") else {
        return Err("missing input resource info".into());
    };

    assert_eq!(resource_info.url.as_str(), &file_url);
    assert_eq!(resource_info.format, ResourceFormat::File);
    assert_eq!(resource_info.hash, file_hash);

    let Some(resource_info) = package_info.outputs.get("example_file") else {
        return Err("missing output resource info".into());
    };

    assert_eq!(resource_info.url.as_str(), &file_url);
    assert_eq!(resource_info.format, ResourceFormat::File);
    assert_eq!(resource_info.hash, file_hash);

    Ok(())
}

#[tokio::test]
async fn self_reference() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_url = format!("{TESTS_DIR_URL}/self_reference/package.json");

    let manifest_hash = Hash::from_base32("6j83ocpqdtt36").unwrap();

    let downloader = Downloader::default();
    let storage = Storage::open(get_test_dir("self_reference")?)?;

    let lock = storage.install_packages(&downloader, [
        manifest_url.clone()
    ]).await?;

    assert!(lock.verify());
    assert!(storage.verify_lock(&lock)?);

    assert!(lock.root.iter().all(|root| root == &manifest_hash));
    assert_eq!(lock.packages.len(), 1);
    assert_eq!(lock.resources.len(), 1);

    assert_eq!(lock.resources.get(&manifest_hash), Some(&manifest_url));

    let Some(package_info) = lock.packages.get(&manifest_hash) else {
        return Err("missing package info".into());
    };

    assert_eq!(package_info.url, manifest_url);
    assert_eq!(package_info.inputs.len(), 2);
    assert!(package_info.outputs.is_empty());

    let Some(resource_info) = package_info.inputs.get("as_file") else {
        return Err("missing as_file resource info".into());
    };

    assert_eq!(resource_info.url.as_str(), &manifest_url);
    assert_eq!(resource_info.format, ResourceFormat::File);
    assert_eq!(resource_info.hash, manifest_hash);

    let Some(resource_info) = package_info.inputs.get("as_package") else {
        return Err("missing as_package resource info".into());
    };

    assert_eq!(resource_info.url.as_str(), &manifest_url);
    assert_eq!(resource_info.format, ResourceFormat::Package);
    assert_eq!(resource_info.hash, manifest_hash);

    Ok(())
}

#[tokio::test]
async fn cycle() -> Result<(), Box<dyn std::error::Error>> {
    let package_1_url = format!("{TESTS_DIR_URL}/cycle/package_1.json");
    let package_2_url = format!("{TESTS_DIR_URL}/cycle/package_2.json");

    let package_1_hash = Hash::from_base32("30mc77m5bcqje").unwrap();
    let package_2_hash = Hash::from_base32("s7lffv3k21im0").unwrap();

    let downloader = Downloader::default();
    let storage = Storage::open(get_test_dir("cycle")?)?;

    let lock = storage.install_packages(&downloader, [
        package_1_url.clone()
    ]).await?;

    assert!(lock.verify());
    assert!(storage.verify_lock(&lock)?);

    assert!(lock.root.iter().all(|root| root == &package_1_hash));
    assert_eq!(lock.packages.len(), 2);
    assert!(lock.resources.is_empty());

    let Some(package_info) = lock.packages.get(&package_1_hash) else {
        return Err("missing package 1 info".into());
    };

    assert_eq!(package_info.url, package_1_url);
    assert_eq!(package_info.inputs.len(), 1);
    assert!(package_info.outputs.is_empty());

    let Some(package_info) = lock.packages.get(&package_2_hash) else {
        return Err("missing package 2 info".into());
    };

    assert_eq!(package_info.url, package_2_url);
    assert_eq!(package_info.inputs.len(), 1);
    assert!(package_info.outputs.is_empty());

    Ok(())
}

#[tokio::test]
async fn nested_packages() -> Result<(), Box<dyn std::error::Error>> {
    let package_1_url = format!("{TESTS_DIR_URL}/nested_packages/package_1.json");
    let package_2_url = format!("{TESTS_DIR_URL}/nested_packages/package_2.json");
    let package_3_url = format!("{TESTS_DIR_URL}/nested_packages/package_3.json");
    let file_url = format!("{TESTS_DIR_URL}/nested_packages/example_file.txt");

    let package_1_hash = Hash::from_base32("30mc77m5bcqje").unwrap();
    let package_2_hash = Hash::from_base32("rj722sqs9o8je").unwrap();
    let package_3_hash = Hash::from_base32("gndmqnirsuij8").unwrap();
    let file_hash = Hash::from_base32("dfhtkkli693ji").unwrap();

    let downloader = Downloader::default();
    let storage = Storage::open(get_test_dir("nested_packages")?)?;

    let lock = storage.install_packages(&downloader, [
        package_1_url.clone()
    ]).await?;

    assert!(lock.verify());
    assert!(storage.verify_lock(&lock)?);

    assert!(lock.root.iter().all(|root| root == &package_1_hash));
    assert_eq!(lock.packages.len(), 3);
    assert_eq!(lock.resources.len(), 1);

    let Some(package_info) = lock.packages.get(&package_1_hash) else {
        return Err("missing package 1 info".into());
    };

    assert_eq!(package_info.url, package_1_url);
    assert_eq!(package_info.inputs.len(), 1);
    assert!(package_info.outputs.is_empty());

    let Some(package_info) = lock.packages.get(&package_2_hash) else {
        return Err("missing package 2 info".into());
    };

    assert_eq!(package_info.url, package_2_url);
    assert_eq!(package_info.inputs.len(), 1);
    assert!(package_info.outputs.is_empty());

    let Some(package_info) = lock.packages.get(&package_3_hash) else {
        return Err("missing package 3 info".into());
    };

    assert_eq!(package_info.url, package_3_url);
    assert_eq!(package_info.inputs.len(), 1);
    assert!(package_info.outputs.is_empty());

    let Some(resource_info) = package_info.inputs.get("example_file") else {
        return Err("missing resource info".into());
    };

    assert_eq!(resource_info.url.as_str(), &file_url);
    assert_eq!(resource_info.format, ResourceFormat::File);
    assert_eq!(resource_info.hash, file_hash);

    Ok(())
}
