// SPDX-License-Identifier: GPL-3.0-or-later
//
// anirun
// Copyright (C) 2026  Nikita Podvirnyi <krypt0nn@vk.com>
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

use std::path::{Path, PathBuf};
use std::time::Duration;

use tracing_subscriber::prelude::*;
use tracing_subscriber::filter::*;

use anyhow::Context;
use clap::Parser;

use agl_core::export::network::reqwest;

use agl_core::tasks;
use agl_core::network::downloader::Downloader;
use agl_locale::string::LocalizableString;
use agl_locale::SYSTEM_LANG;
use agl_packages::hash::Hash;
use agl_packages::format::ResourceFormat;
use agl_packages::storage::Storage;
use agl_packages::lock::Lock;
use agl_runtime::mlua::prelude::*;
use agl_runtime::runtime::{Runtime, ModulePaths};
use agl_runtime::module::{Module, ModuleScope};
use agl_runtime::allow_list::AllowList;
use agl_runtime::api::ApiOptions;
use agl_runtime::api::portal_api::ToastOptions;
use agl_runtime::api::torrent_api::{TorrentServer, TorrentServerOptions};

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(author = "Nikita Podvirnyi <krypt0nn@vk.com>")]
struct Cli {
    #[arg(long, alias = "resources")]
    pub resources_folder: Option<PathBuf>,

    #[arg(long, alias = "temp")]
    pub temp_folder: Option<PathBuf>,

    #[arg(long, alias = "modules")]
    pub modules_folder: Option<PathBuf>,

    #[arg(long, alias = "persistent", alias = "persist")]
    pub persistent_folder: Option<PathBuf>,

    #[arg(long, alias = "lock-files", alias = "locks")]
    pub lock_files_folder: Option<PathBuf>,

    /// Optional proxy string. Used in all HTTP requests and, if socks5 string
    /// provided, in torrent runtime API.
    #[arg(long)]
    pub proxy: Option<String>,

    /// HTTP requests user agent string.
    #[arg(long)]
    pub user_agent: Option<String>,

    /// HTTP requests timeout in milliseconds. No timeout is used if unset.
    #[arg(long)]
    pub timeout: Option<u64>,

    #[command(subcommand)]
    pub command: CliCommands
}

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(author = "Nikita Podvirnyi <krypt0nn@vk.com>")]
enum CliCommands {
    /// Packages manager commands.
    #[command(subcommand)]
    Package(CliPackageCommands),

    /// Luau modules runtime commands.
    #[command(subcommand)]
    Module(CliModuleCommands)
}

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
enum CliPackageCommands {
    /// Download packages.
    Download {
        /// URI to the package manifest file.
        #[arg(
            long,
            alias = "src",
            alias = "from",
            alias = "uri",
            alias = "url"
        )]
        source: Vec<String>,

        /// Name of the output lock file.
        #[arg(short, long, alias = "lock", alias = "name")]
        lock_name: Option<String>
    },

    /// Run luau modules stored as outputs of the package lock file in the
    /// modules runtime.
    Run {
        /// URI to the package manifest file or a lock file.
        #[arg(
            long,
            alias = "src",
            alias = "path",
            alias = "uri",
            alias = "url"
        )]
        source: String,

        #[command(flatten)]
        scope: CliModuleScope,

        #[command(flatten)]
        torrent: TorrentOptionsCli
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
enum CliModuleCommands {
    /// Run luau module in the modules runtime.
    Run {
        /// URI to the module file.
        #[arg(
            short,
            long,
            alias = "src",
            alias = "path",
            alias = "uri",
            alias = "url"
        )]
        source: String,

        #[command(flatten)]
        scope: CliModuleScope,

        #[command(flatten)]
        torrent: TorrentOptionsCli
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
struct CliModuleScope {
    /// Allow module to access string API.
    ///
    /// This API allows module to perform conversions between different string
    /// encodings (UTF-8, ASCII, etc.) and formats (hex, base64, JSON, etc.).
    ///
    /// Default: `true`.
    #[arg(long)]
    pub string_api: Option<bool>,

    /// Allow module to access path API.
    ///
    /// This API allows module to combine different path parts, normalize and
    /// resolve them, check if files or folders exist and if they're accessible.
    ///
    /// Default: `true`.
    #[arg(long)]
    pub path_api: Option<bool>,

    /// Allow module to access task API.
    ///
    /// This API allows module to create promise (future) objects which can
    /// execute tasks in background, and poll their status.
    ///
    /// Default: `true`.
    #[arg(long)]
    pub task_api: Option<bool>,

    /// Allow module to access system API.
    ///
    /// This API allows module to request information about host system's local
    /// time, unix timestamp, read environment variables.
    ///
    /// Default: `true`.
    #[arg(long)]
    pub system_api: Option<bool>,

    /// Allow module to access filesystem API.
    ///
    /// This API allows module to perform read/write/create operations on files
    /// and folders of the host filesystem, with sandboxed access to only
    /// allowed files and folders.
    ///
    /// Default: `true`.
    #[arg(long)]
    pub filesystem_api: Option<bool>,

    /// Allow module to access HTTP API.
    ///
    /// This API allows module to perform HTTP(S) requests.
    ///
    /// Default: `true`.
    #[arg(long)]
    pub http_api: Option<bool>,

    /// Allow module to access downloader API.
    ///
    /// This API allows module to download files from HTTP servers. Similar to
    /// the Network API, except it has more user niceness in it.
    ///
    /// Default: `true`.
    #[arg(long)]
    pub downloader_api: Option<bool>,

    /// Allow module to access archive API.
    ///
    /// This API allows module to extract archives or list their info.
    ///
    /// Default: `true`.
    #[arg(long)]
    pub archive_api: Option<bool>,

    /// Allow module to access hash API.
    ///
    /// This API allows module to calculate different hashes of files or
    /// folders.
    ///
    /// Default: `true`.
    #[arg(long)]
    pub hash_api: Option<bool>,

    /// Allow module to access compression API.
    ///
    /// This API allows module to compress or decompress data with different
    /// compression algorithms.
    ///
    /// Default: `true`.
    #[arg(long)]
    pub compression_api: Option<bool>,

    /// Allow module to access sqlite API.
    ///
    /// This API allows module to work with a sqlite database.
    ///
    /// Default: `true`.
    #[arg(long)]
    pub sqlite_api: Option<bool>,

    /// Allow module to access torrent API.
    ///
    /// This API allows module to work with BitTorrent protocol, download and
    /// share files using DHT, magnet links and torrent files.
    ///
    /// Default: `false`.
    #[arg(long)]
    pub torrent_api: Option<bool>,

    /// Allow module to access portal API.
    ///
    /// This API allows module to send system/application-level notifications
    /// and open file/folder dialogs which can escape the filesystem sandbox.
    ///
    /// Default: `true`.
    #[arg(long)]
    pub portal_api: Option<bool>,

    /// Allow module to access process API.
    ///
    /// This API allows module to spawn and control new processes on the host
    /// system.
    ///
    /// > **Security warning:** This API can be used to escape the sandbox. You
    /// > must make sure that the module *really* needs this API.
    ///
    /// Default: `false`.
    #[arg(long)]
    pub process_api: Option<bool>,

    /// Paths allowed to be accessed for this module. When provided, the module
    /// can use filesystem and other APIs to read provided files or
    /// folders/subfolders.
    ///
    /// Default: none.
    #[arg(long = "sandbox-read-path", alias = "sandbox-read")]
    pub sandbox_read_paths: Vec<PathBuf>,

    /// Paths allowed to be written and read by this module. When provided, the
    /// module can use filesystem and other APIs to read and write provided
    /// files or folders/subfolders.
    ///
    /// Default: none.
    #[arg(long = "sandbox-write-path", alias = "sandbox-write")]
    pub sandbox_write_paths: Vec<PathBuf>
}

impl From<CliModuleScope> for ModuleScope {
    fn from(value: CliModuleScope) -> Self {
        Self {
            allow_string_api: value.string_api.unwrap_or(true),
            allow_path_api: value.path_api.unwrap_or(true),
            allow_task_api: value.task_api.unwrap_or(true),
            allow_system_api: value.system_api.unwrap_or(true),
            allow_filesystem_api: value.filesystem_api.unwrap_or(true),
            allow_http_api: value.http_api.unwrap_or(true),
            allow_downloader_api: value.downloader_api.unwrap_or(true),
            allow_archive_api: value.archive_api.unwrap_or(true),
            allow_hash_api: value.hash_api.unwrap_or(true),
            allow_compression_api: value.compression_api.unwrap_or(true),
            allow_sqlite_api: value.sqlite_api.unwrap_or(true),
            allow_torrent_api: value.torrent_api.unwrap_or(false),
            allow_portal_api: value.portal_api.unwrap_or(true),
            allow_process_api: value.process_api.unwrap_or(false),
            sandbox_read_paths: value.sandbox_read_paths,
            sandbox_write_paths: value.sandbox_write_paths
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
struct TorrentOptionsCli {
    /// Default path to the folder where torrents should be downloaded.
    #[arg(long)]
    pub torrent_folder: Option<PathBuf>,

    /// Enable DHT for torrent server.
    ///
    /// Default: `true`.
    #[arg(long, action = clap::ArgAction::SetTrue, default_value_t = true)]
    #[arg(long = "torrent-disable-dht", action = clap::ArgAction::SetFalse)]
    pub torrent_enable_dht: bool,

    /// Enable UPnP for torrent server.
    ///
    /// Default: `false`.
    #[arg(long, action = clap::ArgAction::SetTrue, default_value_t = true)]
    #[arg(long = "torrent-disable-upnp", action = clap::ArgAction::SetFalse)]
    pub torrent_enable_upnp: bool,

    /// URL to the torrent peers blocklist.
    #[arg(long, alias = "torrent-blocklist")]
    pub torrent_blocklist_url: Option<String>,

    /// URL to the torrent tracker.
    #[arg(long)]
    pub torrent_tracker: Vec<String>
}

fn translate(str: LocalizableString) -> String {
    str.translate(&*SYSTEM_LANG).to_string()
}

fn build_client(
    proxy: Option<String>,
    user_agent: Option<String>,
    timeout: Option<Duration>
) -> anyhow::Result<reqwest::Client> {
    let mut client = reqwest::ClientBuilder::new()
        .user_agent(user_agent.unwrap_or_else(|| format!("anirun/{APP_VERSION}")));

    if let Some(proxy) = &proxy {
        let proxy = reqwest::Proxy::all(proxy)
            .context("failed to build proxy")?;

        client = client.proxy(proxy);
    }

    if let Some(timeout) = timeout {
        client = client.connect_timeout(timeout);
    }

    client.build().context("failed to build HTTP client")
}

fn build_runtime(
    temp_folder: &Path,
    proxy: Option<String>,
    torrent: Option<TorrentOptionsCli>,
    reqwest_client: reqwest::Client
) -> anyhow::Result<Runtime> {
    Ok(Runtime::new(ApiOptions {
        lua: Lua::new(),

        reqwest_client,

        torrent_server: torrent.map(|options| {
            TorrentServer::start(TorrentServerOptions {
                default_folder: options.torrent_folder
                    .unwrap_or_else(|| temp_folder.to_path_buf()),

                socks_proxy: proxy.and_then(|proxy| {
                    proxy.starts_with("socks")
                        .then_some(proxy)
                }),

                trackers: options.torrent_tracker.into_iter().collect(),
                blocklist_url: options.torrent_blocklist_url,

                enable_dht: options.torrent_enable_dht,
                enable_upnp: options.torrent_enable_upnp
            })
        }),

        show_toast: Box::new(|options| {
            let message = match options {
                ToastOptions::Simple(message) |
                ToastOptions::Activatable { message, .. } => translate(message)
            };

            tracing::debug!("");
            tracing::debug!("toast: {message}");
            tracing::debug!("");
        }),

        show_notification: Box::new(|options| {
            let mut notification = notify_rust::Notification::new();
            let mut notification = notification.summary(&translate(options.title));

            if let Some(message) = options.message {
                notification = notification.body(&translate(message));
            }

            if let Some(icon) = options.icon {
                notification = notification.icon(&icon);
            }

            tracing::debug!(?notification, "showing notification");

            if let Err(err) = notification.show() {
                tracing::error!(?err, "failed to show system notification");
            }
        }),

        show_dialog: Box::new(|options| {
            let title = translate(options.title);
            let message = translate(options.message);

            tracing::debug!("");
            tracing::debug!("dialog:");
            tracing::debug!("");
            tracing::debug!("  {title}");

            for line in message.lines() {
                tracing::debug!("  {line}");
            }

            tracing::debug!("");
        }),

        translate
    })?)
}

fn resolve_lua_value(value: LuaValue) -> anyhow::Result<LuaValue> {
    match value {
        LuaValue::Function(ref callback) => {
            resolve_lua_value(callback.call::<LuaValue>(())?)
        }

        LuaValue::Thread(coroutine) => {
            let value = tasks::block_on(coroutine.into_async::<LuaValue>(())?)?;

            resolve_lua_value(value)
        }

        LuaValue::UserData(object) => {
            match object.type_name()?.as_deref() {
                Some("Promise") => {
                    let value = object.call_method::<LuaValue>("await", ())?;

                    resolve_lua_value(value)
                }

                Some("Bytes") => {
                    Ok(object.call_method::<LuaValue>("as_string", ())?)
                }

                _ => Ok(LuaValue::UserData(object))
            }
        }

        _ => Ok(value)
    }
}

fn main() -> anyhow::Result<()> {
    // Prepare stdout logger.
    let stdout_log = tracing_subscriber::fmt::layer()
        .with_filter({
            filter_fn(|metadata| {
                metadata.target().starts_with("anirun")
                    || metadata.target().starts_with("agl_")
            })
        })
        .with_filter(LevelFilter::TRACE);

    // Setup loggers.
    tracing_subscriber::registry()
        .with(stdout_log)
        .init();

    // Start the application.
    tracing::info!(
        anirun_version = APP_VERSION,
        core_version = agl_core::VERSION,
        packages_version = agl_packages::VERSION,
        runtime_version = agl_runtime::VERSION,
        "starting application"
    );

    // Parse CLI args and commands.
    let cli = Cli::parse();

    // Read paths or use default ones.
    let mut resources_folder = cli.resources_folder
        .unwrap_or_else(|| PathBuf::from(".anirun/resources"));

    let mut temp_folder = cli.temp_folder
        .unwrap_or_else(|| PathBuf::from(".anirun/temporary"));

    let mut modules_folder = cli.modules_folder
        .unwrap_or_else(|| PathBuf::from(".anirun/modules"));

    let mut persistent_folder = cli.persistent_folder
        .unwrap_or_else(|| PathBuf::from(".anirun/persistent"));

    let mut lock_files_folder = cli.lock_files_folder
        .unwrap_or_else(|| PathBuf::from(".anirun/locks"));

    // Create folders if they don't exist and resolve relative ones.
    for path in [
        &mut resources_folder,
        &mut temp_folder,
        &mut modules_folder,
        &mut persistent_folder,
        &mut lock_files_folder
    ] {
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }

        *path = path.canonicalize()?;
    }

    // Build reqwest client.
    let client = build_client(
        cli.proxy.clone(),
        cli.user_agent,
        cli.timeout.map(Duration::from_millis)
    )?;

    // Process the parsed command.
    match cli.command {
        CliCommands::Package(command) => match command {
            CliPackageCommands::Download { source, lock_name } => {
                let storage = Storage::open(&resources_folder)
                    .context("failed to open resources storage")?;

                let downloader = Downloader::from_client(client.clone());

                tracing::info!("downloading packages");

                let lock = tasks::block_on(storage.install_packages(&downloader, source))
                    .context("failed to install packages")?;

                let lock_name = lock_name.unwrap_or_else(|| {
                    let hash = lock.resources.keys()
                        .fold(Hash::default(), |acc, resource| acc ^ *resource)
                        .to_string();

                    format!("{hash}.json")
                });

                tracing::info!("downloading finished");

                let path = lock_files_folder.join(lock_name);

                tracing::info!(?path, "saving lock file");

                std::fs::write(
                    path,
                    serde_json::to_vec_pretty(&lock.to_json())?
                )?;

                tracing::info!("done");
            }

            CliPackageCommands::Run { source, scope, torrent } => {
                let storage = Storage::open(&resources_folder)
                    .context("failed to open resources storage")?;

                let lock = if !PathBuf::from(&source).exists() {
                    tracing::debug!(?source, "provided source is not a lock file path, attempting to download packages");

                    let downloader = Downloader::from_client(client.clone());

                    tracing::info!("downloading packages");

                    let lock = tasks::block_on(storage.install_packages(&downloader, [&source]))
                        .context("failed to install packages")?;

                    tracing::info!("downloading finished");

                    lock
                } else {
                    tracing::info!(?source, "reading lock file");

                    let lock = std::fs::read(&source)
                        .context("failed to read lock file")?;

                    let lock = serde_json::from_slice::<serde_json::Value>(&lock)
                        .context("failed to deserialize lock file")?;

                    Lock::from_json(&lock)
                        .ok_or_else(|| anyhow::anyhow!("invalid lock file format"))?
                };

                tracing::info!("preparing modules runtime");

                let runtime = build_runtime(
                    &temp_folder,
                    cli.proxy.clone(),
                    scope.torrent_api.and_then(|enabled| enabled.then_some(torrent)),
                    client
                )?;

                tracing::info!("preparing allow list");

                let mut allow_list = AllowList::default();

                for resource in lock.resources.keys().copied() {
                    allow_list.add_module_scope(
                        resource,
                        ModuleScope::from(scope.clone())
                    );
                }

                tracing::info!("loading resources from the lock file");

                let paths = ModulePaths {
                    temp_folder,
                    modules_folder,
                    persistent_folder
                };

                runtime.load_packages(
                    &lock,
                    &storage,
                    &paths,
                    &allow_list
                )?;

                for (package_hash, package) in lock.packages.iter() {
                    for (resource_name, resource) in package.outputs.iter() {
                        if resource.format == ResourceFormat::File
                            && resource.url.contains(".lua")
                            && let Some(output) = runtime.get_value::<LuaTable>(format!("{}#module", resource.hash))?
                        {
                            let output = output.get::<LuaValue>("value")?;

                            tracing::info!(
                                package_hash = package_hash.to_string(),
                                ?resource_name,
                                resource_hash = resource.hash.to_string(),
                                output = format!("{:#?}", resolve_lua_value(output)?),
                                "module output"
                            );
                        }
                    }
                }
            }
        }

        CliCommands::Module(command) => match command {
            CliModuleCommands::Run { source, scope, torrent } => {
                let mut source_path = PathBuf::from(&source);

                if !source_path.exists() {
                    let temp_path = temp_folder
                        .join(Hash::from_bytes(source.as_bytes()).to_string());

                    tracing::debug!(?source, ?temp_path, "provided module source is not a file path, attempting to download it");

                    let downloader = Downloader::from_client(client.clone());

                    tasks::block_on(downloader.download(&source, &temp_path).wait())
                        .context("failed to download module")?;

                    tracing::debug!(?source, ?temp_path, "downloaded module file");

                    source_path = temp_path;
                }

                tracing::info!("preparing modules runtime");

                let runtime = build_runtime(
                    &temp_folder,
                    cli.proxy.clone(),
                    scope.torrent_api.and_then(|enabled| enabled.then_some(torrent)),
                    client
                )?;

                let module = Module {
                    path: source_path,
                    scope: ModuleScope::from(scope)
                };

                let paths = ModulePaths {
                    temp_folder,
                    modules_folder,
                    persistent_folder
                };

                tracing::info!("loading module");

                runtime.load_module("module", module, paths)
                    .context("failed to load module")?;

                let Some(output) = runtime.get_value::<Option<LuaValue>>("module")?.flatten() else {
                    tracing::debug!("module didn't return any value");

                    return Ok(());
                };

                tracing::info!(
                    output = format!("{:#?}", resolve_lua_value(output)?),
                    "module output"
                );
            }
        }
    }

    Ok(())
}
