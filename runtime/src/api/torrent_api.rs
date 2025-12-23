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
use std::sync::mpsc::Sender;

use mlua::prelude::*;
use librqbit::{
    Session as TorrentSession,
    SessionOptions as TorrentSessionOptions,
    AddTorrent as AddTorrentInfo,
    AddTorrentOptions
};

use agl_core::tasks;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TorrentServerOptions {
    /// Default torrents downloading folder.
    pub default_folder: PathBuf,

    /// Optional socks proxy URL.
    pub socks_proxy: Option<String>,

    /// List of torrent tracker URIs.
    pub trackers: HashSet<String>,

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
            enable_dht: true,
            enable_upnp: true
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TorrentServerMsg {
    AddTorrent {
        uri: String,
        output_folder: PathBuf
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
                    socks_proxy_url: options.socks_proxy,

                    trackers: options.trackers.iter()
                        .flat_map(|url| url.parse())
                        .collect(),

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
                    TorrentServerMsg::AddTorrent { uri, output_folder } => {
                        let mut info = AddTorrentInfo::Url(uri.clone().into());

                        if PathBuf::from(&uri).is_file() {
                            info = match std::fs::read(&uri) {
                                Ok(content) => AddTorrentInfo::TorrentFileBytes(content.into()),
                                Err(err) => {
                                    #[cfg(feature = "tracing")]
                                    tracing::error!(?err, "failed to read torrent file");

                                    continue;
                                }
                            };
                        }

                        let options = AddTorrentOptions {
                            output_folder: Some(output_folder.to_string_lossy().to_string()),
                            overwrite: true,
                            defer_writes: Some(false),

                            ..AddTorrentOptions::default()
                        };

                        if let Err(err) = session.add_torrent(info, Some(options)).await {
                            #[cfg(feature = "tracing")]
                            tracing::error!(?err, "failed to add torrent");
                        }
                    }
                }
            }
        });

        Self(sender)
    }

    pub fn add_torrent(
        &self,
        uri: impl ToString,
        output_folder: impl Into<PathBuf>
    ) {
        let _ = self.0.send(TorrentServerMsg::AddTorrent {
            uri: uri.to_string(),
            output_folder: output_folder.into()
        });
    }
}

pub struct TorrentApi {
    lua: Lua,

    torrent_add: LuaFunctionBuilder
}

impl TorrentApi {
    pub fn new(
        lua: Lua,
        server: TorrentServer
    ) -> Result<Self, LuaError> {
        Ok(Self {
            torrent_add: {
                let torrent_server = server.clone();

                Box::new(move |lua, context| {
                    let torrent_server = torrent_server.clone();
                    let context = context.clone();

                    lua.create_function(move |_, (uri, options): (String, Option<LuaTable>)| {
                        let mut output_folder = context.temp_folder.clone();

                        #[allow(clippy::collapsible_if)]
                        if let Some(options) = options {
                            if let Some(opt_output_folder) = options.get::<Option<String>>("output_folder")? {
                                output_folder = PathBuf::from(opt_output_folder);
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

                        torrent_server.add_torrent(uri, output_folder);

                        Ok(())
                    })
                })
            },

            lua
        })
    }

    /// Create new lua table with API functions.
    pub fn create_env(&self, context: &Context) -> Result<LuaTable, LuaError> {
        let env = self.lua.create_table_with_capacity(0, 1)?;

        env.raw_set("add", (self.torrent_add)(&self.lua, context)?)?;

        Ok(env)
    }
}
