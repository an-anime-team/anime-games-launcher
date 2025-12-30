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

use std::collections::HashSet;
use std::path::PathBuf;
use std::borrow::Cow;
use std::sync::mpsc::Sender;

use agl_core::export::compression::zstd::zstd_safe::WriteBuf;
use mlua::prelude::*;

use librqbit::{
    Session as TorrentSession,
    SessionOptions as TorrentSessionOptions,
    AddTorrent as AddTorrentInfo,
    AddTorrentOptions,
    AddTorrentResponse,
    CreateTorrentOptions,
    Magnet
};

use librqbit::api::TorrentIdOrHash;

use agl_core::tasks;

use super::task_api::{Promise, PromiseValue, TaskOutput, task_output};
use super::*;

#[derive(Debug, thiserror::Error)]
pub enum TorrentServerError {
    #[error("torrent server is offline")]
    ServerIsOffline,

    #[error("failed to read torrent file: {0}")]
    TorrentReadError(#[from] std::io::Error),

    #[error("failed to add torrent: {0}")]
    AddTorrent(#[source] Box<dyn std::error::Error + Send + 'static>),

    #[error("failed to delete torrent: {0}")]
    DeleteTorrent(#[source] Box<dyn std::error::Error + Send + 'static>),

    #[error("invalid torrent info hash format: {0}")]
    InvalidInfoHash(#[source] Box<dyn std::error::Error + Send + 'static>),

    #[error("failed to read torrent metadata: {0}")]
    ReadMetadata(#[source] Box<dyn std::error::Error + Send + 'static>),

    #[error("failed to pause or resume a torrent: {0}")]
    PauseOrResume(#[source] Box<dyn std::error::Error + Send + 'static>)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TorrentServerOptions {
    /// Default torrents downloading folder.
    pub default_folder: PathBuf,

    /// Optional socks proxy URL.
    pub socks_proxy: Option<String>,

    /// List of torrent tracker URIs.
    pub trackers: HashSet<String>,

    /// Optional URL to the peers blocklist.
    pub blocklist_url: Option<String>,

    /// Enable DHT.
    pub enable_dht: bool,

    /// Enable UPnP.
    pub enable_upnp: bool
}

impl Default for TorrentServerOptions {
    fn default() -> Self {
        Self {
            default_folder: std::env::temp_dir(),
            socks_proxy: None,
            trackers: HashSet::new(),
            blocklist_url: None,
            enable_dht: true,
            enable_upnp: true
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TorrentListInfo {
    /// Name of the torrent.
    pub name: Option<String>,

    /// Info hash of the torrent.
    pub info_hash: String,

    /// Torrent stats.
    pub stats: TorrentStats,

    /// Whether the torrent is paused.
    pub paused: bool,

    /// Whether the torrent downloading is finished.
    pub finished: bool
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TorrentPeerInfo {
    /// Address of the peer.
    pub address: String,

    /// Amount of bytes downloaded from this peer.
    pub downloaded: u64
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TorrentFileInfo {
    /// Relative path to the file within a torrent.
    pub path: PathBuf,

    /// Total size of the file.
    pub size: u64
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TorrentStats {
    /// Amount of downloaded (available) bytes.
    pub current: u64,

    /// Total amount of bytes to download.
    pub total: u64,

    /// Amount of bytes uploaded in the current session.
    pub uploaded: u64
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TorrentInfo {
    /// Name of the torrent.
    pub name: Option<String>,

    /// List of torrent trackers.
    pub trackers: Box<[String]>,

    /// List of torrent peers.
    pub peers: Box<[TorrentPeerInfo]>,

    /// List of files within the torrent.
    pub files: Box<[TorrentFileInfo]>,

    /// Stats of the torrent.
    pub stats: TorrentStats,

    /// Whether the torrent is paused.
    pub paused: bool,

    /// Whether the torrent downloading is finished.
    pub finished: bool
}

#[derive(Debug, Clone)]
enum TorrentServerMsg {
    Add {
        torrent: Box<[u8]>,
        output_folder: PathBuf,
        trackers: Option<Vec<String>>,
        paused: bool,
        restart: bool,
        sender: Sender<Result<String, TorrentServerError>>
    },

    List {
        sender: Sender<Box<[TorrentListInfo]>>
    },

    GetInfo {
        info_hash: String,
        sender: Sender<Result<Option<TorrentInfo>, TorrentServerError>>
    },

    PauseOrResume {
        info_hash: String,
        pause: bool,
        sender: Sender<Result<(), TorrentServerError>>
    }
}

#[derive(Debug, Clone)]
pub struct TorrentServer(Sender<TorrentServerMsg>);

impl TorrentServer {
    /// Start torrent server with provided default output folder.
    pub fn start(options: TorrentServerOptions) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();

        tasks::spawn(async move {
            let session = TorrentSession::new_with_opts(
                options.default_folder,
                TorrentSessionOptions {
                    persistence: None,

                    socks_proxy_url: options.socks_proxy,

                    trackers: options.trackers.iter()
                        .flat_map(|url| url.parse())
                        .collect(),

                    blocklist_url: options.blocklist_url,
                    disable_dht: !options.enable_dht,
                    enable_upnp_port_forwarding: options.enable_upnp,

                    ..TorrentSessionOptions::default()
                }
            ).await;

            let session = match session {
                Ok(session) => session,

                Err(err) => {
                    #[cfg(feature = "tracing")]
                    tracing::error!(?err, "failed to start torrent server");

                    return;
                }
            };

            while let Ok(msg) = receiver.recv() {
                match msg {
                    TorrentServerMsg::Add {
                        torrent,
                        output_folder,
                        trackers,
                        paused,
                        restart,
                        sender
                    } => {
                        fn get_torrent_info<'a>(
                            torrent: Box<[u8]>
                        ) -> std::io::Result<AddTorrentInfo<'a>> {
                            let torrent_str = String::from_utf8_lossy(torrent.as_slice())
                                .to_string();

                            let mut info = AddTorrentInfo::TorrentFileBytes(torrent.into());

                            if torrent_str.starts_with("magnet:")
                                || torrent_str.starts_with("http")
                                || matches!(TorrentIdOrHash::parse(&torrent_str), Ok(TorrentIdOrHash::Hash(_)))
                            {
                                info = AddTorrentInfo::Url(Cow::Owned(torrent_str));
                            }

                            else if PathBuf::from(&torrent_str).is_file() {
                                // FIXME: check read permissions in torrent API
                                info = AddTorrentInfo::TorrentFileBytes(std::fs::read(&torrent_str)?.into());
                            }

                            Ok(info)
                        }

                        let info = match get_torrent_info(torrent.clone()) {
                            Ok(info) => info,
                            Err(err) => {
                                #[cfg(feature = "tracing")]
                                tracing::error!(?err, "failed to read torrent file");

                                let _ = sender.send(Err(TorrentServerError::TorrentReadError(err)));

                                continue;
                            }
                        };

                        let trackers = trackers
                            .map(|mut trackers| {
                                trackers.extend(options.trackers.clone());

                                trackers
                            })
                            .unwrap_or_else(|| {
                                options.trackers.iter()
                                    .cloned()
                                    .collect()
                            });

                        let options = AddTorrentOptions {
                            output_folder: Some(output_folder.to_string_lossy().to_string()),
                            overwrite: true,
                            defer_writes: Some(false),
                            trackers: Some(trackers.clone()),
                            paused,

                            ..AddTorrentOptions::default()
                        };

                        let mut torrent_handle = match session.add_torrent(info, Some(options)).await {
                            Ok(torrent) => torrent,
                            Err(err) => {
                                #[cfg(feature = "tracing")]
                                tracing::error!(?err, "failed to add torrent");

                                let _ = sender.send(Err(TorrentServerError::AddTorrent(err.into())));

                                continue;
                            }
                        };

                        if let AddTorrentResponse::AlreadyManaged(id, _) = torrent_handle && restart {
                            if let Err(err) = session.delete(id.into(), false).await {
                                #[cfg(feature = "tracing")]
                                tracing::error!(?err, "failed to delete added torrent");

                                let _ = sender.send(Err(TorrentServerError::DeleteTorrent(err.into())));

                                continue;
                            }

                            let info = match get_torrent_info(torrent) {
                                Ok(info) => info,
                                Err(err) => {
                                    #[cfg(feature = "tracing")]
                                    tracing::error!(?err, "failed to read torrent file");

                                    let _ = sender.send(Err(TorrentServerError::TorrentReadError(err)));

                                    continue;
                                }
                            };

                            let options = AddTorrentOptions {
                                output_folder: Some(output_folder.to_string_lossy().to_string()),
                                overwrite: true,
                                defer_writes: Some(false),
                                trackers: Some(trackers),
                                paused,

                                ..AddTorrentOptions::default()
                            };

                            torrent_handle = match session.add_torrent(info, Some(options)).await {
                                Ok(torrent_handle) => torrent_handle,
                                Err(err) => {
                                    #[cfg(feature = "tracing")]
                                    tracing::error!(?err, "failed to add torrent");

                                    let _ = sender.send(Err(TorrentServerError::AddTorrent(err.into())));

                                    continue;
                                }
                            };
                        }

                        let info_hash = match torrent_handle {
                            AddTorrentResponse::Added(_, handle) |
                            AddTorrentResponse::AlreadyManaged(_, handle) => handle.info_hash(),
                            AddTorrentResponse::ListOnly(info) => info.info_hash
                        };

                        let _ = sender.send(Ok(info_hash.as_string()));
                    }

                    TorrentServerMsg::List { sender } => {
                        let torrents = session.with_torrents(|torrents| {
                            torrents.into_iter()
                                .map(|(_, info)| {
                                    let stats = info.stats();

                                    TorrentListInfo {
                                        name: info.name(),
                                        info_hash: info.info_hash().as_string(),
                                        stats: TorrentStats {
                                            current: stats.progress_bytes,
                                            total: stats.total_bytes,
                                            uploaded: stats.uploaded_bytes
                                        },
                                        paused: info.is_paused(),
                                        finished: stats.finished
                                    }
                                })
                                .collect::<Box<[_]>>()
                        });

                        let _ = sender.send(torrents);
                    }

                    TorrentServerMsg::GetInfo { info_hash, sender } => {
                        let info_hash = match TorrentIdOrHash::parse(&info_hash) {
                            Ok(info_hash) => info_hash,

                            Err(err) => {
                                #[cfg(feature = "tracing")]
                                tracing::error!(?err, "failed to parse torrent info hash");

                                let _ = sender.send(Err(TorrentServerError::InvalidInfoHash(err.into())));

                                continue;
                            }
                        };

                        let Some(info) = session.get(info_hash) else {
                            let _ = sender.send(Ok(None));

                            continue;
                        };

                        let mut peers = Vec::new();

                        if let Some(live_info) = info.live() {
                            let peers_info = live_info.per_peer_stats_snapshot(Default::default());

                            for (address, stats) in peers_info.peers {
                                peers.push(TorrentPeerInfo {
                                    address,
                                    downloaded: stats.counters.fetched_bytes
                                });
                            }
                        }

                        let mut files = Vec::new();

                        let result = info.with_metadata(|metadata| {
                            for file in &metadata.file_infos {
                                files.push(TorrentFileInfo {
                                    path: file.relative_filename.clone(),
                                    size: file.len
                                });
                            }
                        });

                        if let Err(err) = result {
                            #[cfg(feature = "tracing")]
                            tracing::error!(?err, ?info_hash, "failed to read torrent metadata");

                            let _ = sender.send(Err(TorrentServerError::ReadMetadata(err.into())));

                            continue;
                        }

                        let stats = info.stats();

                        let _ = sender.send(Ok(Some(TorrentInfo {
                            name: info.name(),
                            trackers: info.shared().trackers.iter()
                                .map(|url| url.to_string())
                                .collect(),
                            peers: peers.into_boxed_slice(),
                            files: files.into_boxed_slice(),
                            stats: TorrentStats {
                                current: stats.progress_bytes,
                                total: stats.total_bytes,
                                uploaded: stats.uploaded_bytes
                            },
                            paused: info.is_paused(),
                            finished: stats.finished
                        })));
                    }

                    TorrentServerMsg::PauseOrResume {
                        info_hash,
                        pause,
                        sender
                    } => {
                        let info_hash = match TorrentIdOrHash::parse(&info_hash) {
                            Ok(info_hash) => info_hash,

                            Err(err) => {
                                #[cfg(feature = "tracing")]
                                tracing::error!(?err, "failed to parse torrent info hash");

                                let _ = sender.send(Err(TorrentServerError::InvalidInfoHash(err.into())));

                                continue;
                            }
                        };

                        let Some(info) = session.get(info_hash) else {
                            let _ = sender.send(Ok(()));

                            continue;
                        };

                        let result = if pause {
                            session.pause(&info).await
                        } else {
                            session.unpause(&info).await
                        };

                        if let Err(err) = result {
                            #[cfg(feature = "tracing")]
                            tracing::error!(?err, ?info_hash, ?pause, "failed to pause or resume a torrent");

                            let _ = sender.send(Err(TorrentServerError::PauseOrResume(err.into())));

                            continue;
                        };

                        let _ = sender.send(Ok(()));
                    }
                }
            }
        });

        Self(sender)
    }

    /// Try to add torrent file, magnet link or info hash to the downloading
    /// queue. If succeeded - return info hash string of the added torrent.
    pub fn add_torrent(
        &self,
        torrent: Box<[u8]>,
        output_folder: PathBuf,
        trackers: Option<Vec<String>>,
        paused: bool,
        restart: bool
    ) -> Result<String, TorrentServerError> {
        let (sender, receiver) = std::sync::mpsc::channel();

        let result = self.0.send(TorrentServerMsg::Add {
            torrent,
            output_folder,
            trackers,
            paused,
            restart,
            sender
        });

        if result.is_err() {
            return Err(TorrentServerError::ServerIsOffline);
        }

        receiver.recv()
            .map_err(|_| TorrentServerError::ServerIsOffline)?
    }

    /// Try to get list of info hashes of all the added torrents.
    pub fn list(&self) -> Result<Box<[TorrentListInfo]>, TorrentServerError> {
        let (sender, receiver) = std::sync::mpsc::channel();

        let result = self.0.send(TorrentServerMsg::List {
            sender
        });

        if result.is_err() {
            return Err(TorrentServerError::ServerIsOffline);
        }

        receiver.recv()
            .map_err(|_| TorrentServerError::ServerIsOffline)
    }

    /// Try to get information about added torrent file with provided info hash.
    /// Return `Ok(None)` if there's no torrent with provided info hash.
    pub fn get_info(
        &self,
        info_hash: impl ToString
    ) -> Result<Option<TorrentInfo>, TorrentServerError> {
        let (sender, receiver) = std::sync::mpsc::channel();

        let result = self.0.send(TorrentServerMsg::GetInfo {
            info_hash: info_hash.to_string(),
            sender
        });

        if result.is_err() {
            return Err(TorrentServerError::ServerIsOffline);
        }

        receiver.recv()
            .map_err(|_| TorrentServerError::ServerIsOffline)?
    }

    /// Try to pause or resume torrent downloading and seeding.
    pub fn pause_or_resume(
        &self,
        info_hash: impl ToString,
        pause: bool
    ) -> Result<(), TorrentServerError> {
        let (sender, receiver) = std::sync::mpsc::channel();

        let result = self.0.send(TorrentServerMsg::PauseOrResume {
            info_hash: info_hash.to_string(),
            pause,
            sender
        });

        if result.is_err() {
            return Err(TorrentServerError::ServerIsOffline);
        }

        receiver.recv()
            .map_err(|_| TorrentServerError::ServerIsOffline)?
    }
}

pub struct TorrentApi {
    lua: Lua,

    torrent_create: LuaFunctionBuilder,
    torrent_add: LuaFunctionBuilder,
    torrent_list: LuaFunction,
    torrent_info: LuaFunction,
    torrent_pause: LuaFunction,
    torrent_resume: LuaFunction
}

impl TorrentApi {
    pub fn new(
        lua: Lua,
        server: TorrentServer
    ) -> Result<Self, LuaError> {
        Ok(Self {
            torrent_create: Box::new(|lua, context| {
                let context = context.clone();

                lua.create_function(move |lua: &Lua, (mut path, options): (PathBuf, Option<LuaTable>)| {
                    let mut name = None;
                    let mut piece_size = None;
                    let mut trackers = vec![];

                    if let Some(options) = options {
                        name = options.get::<Option<String>>("name")?;
                        piece_size = options.get::<Option<u32>>("piece_size")?;

                        if let Some(opt_trackers) = options.get::<Option<Vec<String>>>("trackers")? {
                            trackers = opt_trackers;
                        }
                    }

                    if path.is_relative() {
                        path = context.module_folder.join(path);
                    }

                    path = normalize_path(path, true)
                        .map_err(|err| {
                            LuaError::external(format!("failed to normalize path: {err}"))
                        })?;

                    if !context.can_read_path(&path)? {
                        return Err(LuaError::external("no path read permissions"));
                    }

                    let value = PromiseValue::from_future(async move {
                        let result = librqbit::create_torrent(
                            &path,
                            CreateTorrentOptions {
                                name: name.as_deref(),
                                piece_length: piece_size
                            }
                        ).await;

                        let torrent = result.map_err(|err| {
                            LuaError::external(format!("failed to create torrent: {err}"))
                        })?;

                        let mut magnet = Magnet::from_id20(
                            torrent.info_hash(),
                            trackers,
                            None
                        );

                        magnet.name = name.clone();

                        let content = torrent.as_bytes()
                            .map_err(|err| {
                                LuaError::external(format!("failed to get torrent file content: {err}"))
                            })?;

                        Ok(Box::new(move |lua: &Lua| {
                            let result = lua.create_table_with_capacity(0, 3)?;

                            result.raw_set("info_hash", torrent.info_hash().as_string())?;
                            result.raw_set("magnet", magnet.to_string())?;
                            result.raw_set("content", content.to_vec())?;

                            Ok(LuaValue::Table(result))
                        }) as TaskOutput)
                    });

                    Promise::new(value)
                        .into_lua(lua)
                })
            }),

            torrent_add: {
                let torrent_server = server.clone();

                Box::new(move |lua, context| {
                    let torrent_server = torrent_server.clone();
                    let context = context.clone();

                    lua.create_function(move |lua: &Lua, (torrent, options): (LuaValue, Option<LuaTable>)| {
                        let mut output_folder = context.temp_folder.clone();
                        let mut trackers = None;
                        let mut paused = false;
                        let mut restart = true;

                        if let Some(options) = options {
                            if let Some(opt_output_folder) = options.get::<Option<String>>("output_folder")? {
                                output_folder = PathBuf::from(opt_output_folder);
                            }

                            trackers = options.get::<Option<Vec<String>>>("trackers")?;

                            if let Some(opt_paused) = options.get::<Option<bool>>("paused")? {
                                paused = opt_paused;
                            }

                            if let Some(opt_restart) = options.get::<Option<bool>>("restart")? {
                                restart = opt_restart;
                            }
                        }

                        if output_folder.is_relative() {
                            output_folder = context.module_folder.join(output_folder);
                        }

                        output_folder = normalize_path(output_folder, true)
                            .map_err(|err| {
                                LuaError::external(format!("failed to normalize output folder path: {err}"))
                            })?;

                        if !context.can_write_path(&output_folder)? {
                            return Err(LuaError::external("no output folder write permissions"));
                        }

                        let torrent = lua_value_to_bytes(torrent)?
                            .into_boxed_slice();

                        let torrent_server = torrent_server.clone();

                        let value = PromiseValue::from_blocking(move || {
                            let result = torrent_server.add_torrent(
                                torrent,
                                output_folder,
                                trackers,
                                paused,
                                restart
                            );

                            let result = result.map_err(|err| {
                                LuaError::external(err.to_string())
                            })?;

                            Ok(Box::new(move |lua: &Lua| {
                                lua.create_string(result.as_bytes())
                                    .map(LuaValue::String)
                            }) as TaskOutput)
                        });

                        Promise::new(value)
                            .into_lua(lua)
                    })
                })
            },

            torrent_list: {
                let torrent_server = server.clone();

                lua.create_function(move |lua: &Lua, _: ()| {
                    let torrent_server = torrent_server.clone();

                    let value = PromiseValue::from_blocking(move || {
                        let torrents = torrent_server.list()
                            .map_err(|err| LuaError::external(err.to_string()))?;

                        Ok(Box::new(move |lua: &Lua| {
                            let result = lua.create_table_with_capacity(torrents.len(), 0)?;

                            for torrent in torrents {
                                let stats = lua.create_table_with_capacity(0, 3)?;

                                stats.raw_set("current", torrent.stats.current)?;
                                stats.raw_set("total", torrent.stats.total)?;
                                stats.raw_set("uploaded", torrent.stats.uploaded)?;

                                let torrent_info = lua.create_table_with_capacity(0, 5)?;

                                torrent_info.raw_set("name", torrent.name)?;
                                torrent_info.raw_set("info_hash", torrent.info_hash)?;
                                torrent_info.raw_set("stats", stats)?;
                                torrent_info.raw_set("paused", torrent.paused)?;
                                torrent_info.raw_set("finished", torrent.finished)?;

                                result.raw_push(torrent_info)?;
                            }

                            Ok(LuaValue::Table(result))
                        }) as TaskOutput)
                    });

                    Promise::new(value)
                        .into_lua(lua)
                })?
            },

            torrent_info: {
                let torrent_server = server.clone();

                lua.create_function(move |lua: &Lua, info_hash: String| {
                    let torrent_server = torrent_server.clone();

                    let value = PromiseValue::from_blocking(move || {
                        let info = torrent_server.get_info(info_hash)
                            .map_err(|err| LuaError::external(err.to_string()))?;

                        Ok(Box::new(move |lua: &Lua| {
                            let Some(info) = info else {
                                return Ok(LuaValue::Nil);
                            };

                            let peers = lua.create_table_with_capacity(info.peers.len(), 0)?;

                            for peer in info.peers {
                                let peer_info = lua.create_table_with_capacity(0, 2)?;

                                peer_info.raw_set("address", peer.address)?;
                                peer_info.raw_set("downloaded", peer.downloaded)?;

                                peers.raw_push(peer_info)?;
                            }

                            let files = lua.create_table_with_capacity(info.files.len(), 0)?;

                            for file in info.files {
                                let file_info = lua.create_table_with_capacity(0, 2)?;

                                file_info.raw_set("path", file.path)?;
                                file_info.raw_set("size", file.size)?;

                                files.raw_push(file_info)?;
                            }

                            let stats = lua.create_table_with_capacity(0, 3)?;

                            stats.raw_set("current", info.stats.current)?;
                            stats.raw_set("total", info.stats.total)?;
                            stats.raw_set("uploaded", info.stats.uploaded)?;

                            let result = lua.create_table_with_capacity(0, 7)?;

                            result.raw_set("name", info.name)?;
                            result.raw_set("trackers", info.trackers)?;
                            result.raw_set("peers", peers)?;
                            result.raw_set("files", files)?;
                            result.raw_set("stats", stats)?;
                            result.raw_set("paused", info.paused)?;
                            result.raw_set("finished", info.finished)?;

                            Ok(LuaValue::Table(result))
                        }) as TaskOutput)
                    });

                    Promise::new(value)
                        .into_lua(lua)
                })?
            },

            torrent_pause: {
                let torrent_server = server.clone();

                lua.create_function(move |lua: &Lua, info_hash: String| {
                    let torrent_server = torrent_server.clone();

                    let value = PromiseValue::from_blocking(move || {
                        torrent_server.pause_or_resume(info_hash, true)
                            .map_err(|err| LuaError::external(err.to_string()))?;

                        Ok(task_output(Ok(LuaValue::Nil)))
                    });

                    Promise::new(value)
                        .into_lua(lua)
                })?
            },

            torrent_resume: {
                let torrent_server = server.clone();

                lua.create_function(move |lua: &Lua, info_hash: String| {
                    let torrent_server = torrent_server.clone();

                    let value = PromiseValue::from_blocking(move || {
                        torrent_server.pause_or_resume(info_hash, false)
                            .map_err(|err| LuaError::external(err.to_string()))?;

                        Ok(task_output(Ok(LuaValue::Nil)))
                    });

                    Promise::new(value)
                        .into_lua(lua)
                })?
            },

            lua
        })
    }

    /// Create new lua table with API functions.
    pub fn create_env(&self, context: &Context) -> Result<LuaTable, LuaError> {
        let env = self.lua.create_table_with_capacity(0, 6)?;

        env.raw_set("create", (self.torrent_create)(&self.lua, context)?)?;
        env.raw_set("add", (self.torrent_add)(&self.lua, context)?)?;
        env.raw_set("list", &self.torrent_list)?;
        env.raw_set("info", &self.torrent_info)?;
        env.raw_set("pause", &self.torrent_pause)?;
        env.raw_set("resume", &self.torrent_resume)?;

        Ok(env)
    }
}
