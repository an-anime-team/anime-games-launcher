// SPDX-License-Identifier: GPL-3.0-or-later
//
// anime-games-launcher
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

use std::sync::Arc;
use std::collections::HashMap;

use adw::prelude::*;
use relm4::prelude::*;

use serde_json::Value as Json;
use anyhow::Context;

use agl_core::network::downloader::{Downloader, DownloadOptions};
use agl_games::manifest::{GamesRegistryManifest, GameManifest};
use agl_games::engine::{GameIntegration, GameSettingsGroup};

use crate::consts;
use crate::config;
use crate::cache;
use crate::games::GameLock;
use crate::ui::dialogs;
use crate::ui::windows::game_settings::{
    GameSettingsWindow,
    GameSettingsWindowInput
};

pub mod store_page;
pub mod library_page;
// pub mod downloads_page;

use store_page::{StorePage, StorePageInput, StorePageOutput};
use library_page::{LibraryPage, LibraryPageInput, LibraryPageOutput};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum MainWindowMsg {
    SetLoadingStatus(Option<String>),

    AddStorePageGame {
        manifest_url: String,
        manifest: GameManifest
    },

    AddLibraryPageGame(GameLock),

    SetShowBackButton(bool),
    GoBackButtonClicked,

    ShowLibraryGameWithUrl(String),

    OpenGameSettingsWindow {
        layout: Box<[GameSettingsGroup]>,
        integration: Arc<GameIntegration>
    }

    // FinishLoading {
    //     generation: GenerationManifest,
    //     validator: AuthorityValidator,
    //     local_validator: LocalValidator
    // },

    // AddGame {
    //     url: String,
    //     manifest: GameManifest
    // },

    // ToggleSearching,
    // SetShowSearch(bool),
    // SetShowBack(bool),
    // GoBack,

    // ActivateStorePage,
    // ActivateLibraryPage
}

#[derive(Debug)]
pub struct MainWindow {
    store_page: AsyncController<StorePage>,
    library_page: AsyncController<LibraryPage>,
    game_settings_window: AsyncController<GameSettingsWindow>,

    window: Option<adw::ApplicationWindow>,
    view_stack: adw::ViewStack,

    loading_status: Option<String>,

    show_back_button: bool
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for MainWindow {
    type Init = ();
    type Input = MainWindowMsg;
    type Output = ();

    view! {
        #[root]
        _window = adw::ApplicationWindow {
            set_title: Some("Anime Games Launcher"),
            set_size_request: (1200, 800),

            add_css_class?: consts::APP_DEBUG.then_some("devel"),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    #[watch]
                    set_visible: model.loading_status.is_some(),

                    adw::HeaderBar {
                        add_css_class: "flat"
                    },

                    adw::StatusPage {
                        set_vexpand: true,
                        set_hexpand: true,

                        set_icon_name: Some(consts::APP_ID),

                        set_title: "Loading",

                        #[watch]
                        set_description: model.loading_status.as_deref()
                    }
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    #[watch]
                    set_visible: model.loading_status.is_none(),

                    adw::HeaderBar {
                        // pack_start = &gtk::Button {
                        //     set_icon_name: "loupe-symbolic",
                        //     add_css_class: "flat",

                        //     #[watch]
                        //     set_visible: model.show_search && !model.show_back,

                        //     connect_clicked => MainWindowMsg::ToggleSearching
                        // },

                        pack_start = &gtk::Button {
                            set_icon_name: "go-previous-symbolic",
                            add_css_class: "flat",

                            #[watch]
                            set_visible: model.show_back_button,

                            connect_clicked => MainWindowMsg::GoBackButtonClicked
                        },

                        #[wrap(Some)]
                        set_title_widget = &adw::ViewSwitcher {
                            set_policy: adw::ViewSwitcherPolicy::Wide,

                            set_stack: Some(_view_stack)
                        }
                    },

                    #[local_ref]
                    _view_stack -> adw::ViewStack {
                        add = &gtk::Box {
                            set_vexpand: true,
                            set_hexpand: true,

                            model.store_page.widget(),
                        } -> {
                            set_title: Some("Store"),
                            set_name: Some("store"),
                            set_icon_name: Some("folder-download-symbolic")
                        },

                        add = &gtk::Box {
                            set_vexpand: true,
                            set_hexpand: true,

                            model.library_page.widget(),
                        } -> {
                            set_title: Some("Library"),
                            set_name: Some("library"),
                            set_icon_name: Some("applications-games-symbolic")
                        }

                        // connect_visible_child_notify[sender] => move |stack| {
                        //     if let Some(name) = stack.visible_child_name() {
                        //         sender.input(MainWindowMsg::SetShowSearch(
                        //             ["store", "library"].contains(&name.as_str())
                        //         ));

                        //         match name.as_str() {
                        //             "store" => sender.input(MainWindowMsg::ActivateStorePage),
                        //             "library" => sender.input(MainWindowMsg::ActivateLibraryPage),

                        //             _ => ()
                        //         }
                        //     }
                        // }
                    }
                }
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>
    ) -> AsyncComponentParts<Self> {
        let mut model = Self {
            store_page: StorePage::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    StorePageOutput::SetShowBack(show)
                        => MainWindowMsg::SetShowBackButton(show),

                    StorePageOutput::AddLibraryPageGame(game)
                        => MainWindowMsg::AddLibraryPageGame(game),

                    StorePageOutput::ShowLibraryGameWithUrl(url)
                        => MainWindowMsg::ShowLibraryGameWithUrl(url)
                }),

            library_page: LibraryPage::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    LibraryPageOutput::OpenGameSettingsWindow { layout, integration }
                        => MainWindowMsg::OpenGameSettingsWindow { layout, integration }
                }),

            game_settings_window: GameSettingsWindow::builder()
                .launch(())
                .detach(),

            window: None,
            view_stack: adw::ViewStack::new(),

            loading_status: Some(String::new()),

            show_back_button: false
        };

        // Named like this to supress relm4 view macro warning.
        let _view_stack = &model.view_stack;

        let widgets = view_output!();

        model.window = Some(widgets._window.clone());

        let task = tokio::spawn(async move {
            // Create default folders.

            tracing::debug!("creating default folders");

            sender.input(MainWindowMsg::SetLoadingStatus(
                Some(String::from("Creating default folders"))
            ));

            std::fs::create_dir_all(consts::DATA_FOLDER.as_path())?;
            std::fs::create_dir_all(consts::CONFIG_FOLDER.as_path())?;
            std::fs::create_dir_all(consts::CACHE_FOLDER.as_path())?;

            std::fs::create_dir_all(&config::startup().packages_resources_path)?;
            std::fs::create_dir_all(&config::startup().packages_modules_path)?;
            std::fs::create_dir_all(&config::startup().packages_persistent_path)?;
            std::fs::create_dir_all(&config::startup().packages_temporary_path)?;
            std::fs::create_dir_all(&config::startup().games_path)?;

            // Update the config file to create it if it didn't exist before.
            // Do it after creating all the folders, including the config one.
            config::set(config::startup())?;

            // Fetch game registries.

            tracing::debug!(
                registries = ?config::startup().games_registries,
                "fetching game registries"
            );

            sender.input(MainWindowMsg::SetLoadingStatus(
                Some(String::from("Fetching game registries"))
            ));

            // Create network client from config file.
            let client = config::startup()
                .client_builder()
                .context("failed to create network client from the config values")?
                .build()
                .context("failed to build network client")?;

            // Prepare downloader and files cache.
            let downloader = Downloader::from_client(client);

            let mut tasks = Vec::with_capacity(config::startup().games_registries.len());
            let mut paths = Vec::with_capacity(tasks.capacity());

            // Either fetch game registry manifest or use cached one.
            for url in &config::startup().games_registries {
                let cache_path = cache::get_path(url);

                tracing::trace!(?url, ?cache_path, "fetching game registry");

                // If cache for this registry is expired - request the registry
                // value again.
                if cache::is_expired(url, cache::DEFAULT_TTL)? {
                    tracing::trace!(?url, ?cache_path, "game registry cache is expired");

                    let task = downloader.download_with_options(
                        url,
                        &cache_path,
                        DownloadOptions {
                            continue_download: false,
                            on_update: None,
                            on_finish: None
                        }
                    );

                    tasks.push((url, cache_path.clone(), task));
                }

                paths.push(cache_path);
            }

            // Wait for all the game registries to be downloaded.
            for (url, path, task) in tasks {
                tracing::trace!(?url, ?path, "awaiting game registry downloading");

                let result = task.wait().await
                    .context("failed to await game registry fetching");

                if let Err(err) = result {
                    // Remove half-downloaded/broken file.
                    let _ = std::fs::remove_file(path);

                    return Err(err);
                }
            }

            let mut games_manifests = HashMap::<String, bool>::new();

            for path in paths {
                tracing::trace!(?path, "reading game registry");

                let registry = std::fs::read(path)?;
                let registry = serde_json::from_slice::<Json>(&registry)?;

                let registry = GamesRegistryManifest::from_json(&registry)
                    .context("failed to deserialize games registry")?;

                // List all the games manifests' URLs and whether they're
                // featured.
                for game in registry.games {
                    *games_manifests.entry(game.url)
                        .or_default() |= game.featured;
                }
            }

            // Fetch game manifests.

            tracing::debug!(
                urls = ?games_manifests.keys()
                    .collect::<Vec<_>>(),
                "fetching games manifests"
            );

            sender.input(MainWindowMsg::SetLoadingStatus(
                Some(String::from("Fetching games manifests"))
            ));

            let mut tasks = Vec::with_capacity(games_manifests.len());
            let mut paths = Vec::with_capacity(games_manifests.len());

            // Iterate over the list of game manifests URLs.
            for (url, is_featured) in games_manifests {
                let cache_path = cache::get_path(&url);

                tracing::trace!(?url, ?cache_path, "fetching game manifest");

                // If cache for this game manifest is expired - request the
                // manifest again.
                if cache::is_expired(&url, cache::DEFAULT_TTL)? {
                    tracing::trace!(?url, ?cache_path, "game manifest cache is expired");

                    let task = downloader.download_with_options(
                        &url,
                        &cache_path,
                        DownloadOptions {
                            continue_download: false,
                            on_update: None,
                            on_finish: None
                        }
                    );

                    tasks.push((url.clone(), cache_path.clone(), task));
                }

                paths.push((url, cache_path, is_featured));
            }

            // Wait for all the game manifests to be downloaded.
            for (url, path, task) in tasks {
                tracing::trace!(?url, ?path, "awaiting game manifest downloading");

                let result = task.wait().await
                    .context("failed to await game manifest fetching");

                if let Err(err) = result {
                    // Remove half-downloaded/broken file.
                    let _ = std::fs::remove_file(path);

                    return Err(err);
                }
            }

            // Load added game packages locks.

            tracing::debug!("loading added game packages locks");

            sender.input(MainWindowMsg::SetLoadingStatus(
                Some(String::from("Loading added games"))
            ));

            for entry in config::startup().games_path.read_dir()? {
                let entry = entry?;

                tracing::trace!(
                    path = ?entry.path(),
                    "loading added game package lock"
                );

                // TODO: update the lock before loading it.

                let lock = std::fs::read(entry.path())?;
                let lock = serde_json::from_slice::<Json>(&lock)?;

                let lock = GameLock::from_json(&lock)
                    .context("failed to deserialize game package lock")?;

                sender.input(MainWindowMsg::AddLibraryPageGame(lock));
            }

            // Add store page games.

            tracing::debug!(?paths, "adding store page games");

            sender.input(MainWindowMsg::SetLoadingStatus(
                Some(String::from("Adding store page games"))
            ));

            for (url, path, _is_featured) in paths {
                tracing::trace!(?url, ?path, "reading game manifest");

                let manifest = std::fs::read(path)?;
                let manifest = serde_json::from_slice::<Json>(&manifest)?;

                let manifest = GameManifest::from_json(&manifest)
                    .context("failed to deserialize game manifest")?;

                sender.input(MainWindowMsg::AddStorePageGame {
                    manifest_url: url,
                    manifest
                });
            }

            // Finish loading.

            tracing::info!("loading finished");

            sender.input(MainWindowMsg::SetLoadingStatus(None));

            Ok::<_, anyhow::Error>(())
        });

        // Handle error from the above task.
        tokio::spawn(async move {
            match task.await {
                Ok(Ok(())) => (),

                Ok(Err(err)) => {
                    tracing::error!(?err, "failed to execute startup task");

                    dialogs::critical_error("failed to execute startup task", err);
                }

                Err(err) => {
                    tracing::error!(?err, "failed to execute startup task");

                    dialogs::critical_error("failed to execute startup task", err);
                }
            }
        });

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        _sender: AsyncComponentSender<Self>
    ) {
        match message {
            MainWindowMsg::SetLoadingStatus(status) => self.loading_status = status,

            MainWindowMsg::AddStorePageGame { manifest_url, manifest } => {
                self.store_page.emit(StorePageInput::AddGame {
                    manifest_url,
                    manifest
                });
            }

            MainWindowMsg::AddLibraryPageGame(game) => {
                self.library_page.emit(LibraryPageInput::AddGame(game));
            }

            MainWindowMsg::SetShowBackButton(show) => self.show_back_button = show,

            MainWindowMsg::GoBackButtonClicked => {
                self.store_page.emit(StorePageInput::CloseGameDetails);
            }

            MainWindowMsg::ShowLibraryGameWithUrl(url) => {
                self.view_stack.set_visible_child_name("library");

                self.library_page.emit(LibraryPageInput::SelectGameWithUrl(url));
            }

            MainWindowMsg::OpenGameSettingsWindow { layout, integration } => {
                if let Some(window) = &self.window {
                    self.game_settings_window.emit(GameSettingsWindowInput::SetGame {
                        layout,
                        integration
                    });

                    self.game_settings_window.widget()
                        .present(Some(window));
                }
            }

            // MainWindowMsg::FinishLoading { generation, validator, local_validator } => {
            //     if let Some(library_page) = self.library_page.as_ref() {
            //         library_page.emit(LibraryPageInput::SpawnLuauEngine {
            //             generation,
            //             validator,
            //             local_validator
            //         });
            //     }

            //     self.is_loading = false;
            // }

            // MainWindowMsg::AddGame { url, manifest } => {
            //     let manifest = Arc::new(manifest);

            //     self.games.insert(url.clone(), manifest.clone());

            //     self.store_page.emit(StorePageInput::AddGame {
            //         url,
            //         manifest: manifest.clone()
            //     });
            // }

            // MainWindowMsg::ToggleSearching => {
            //     self.store_page.emit(StorePageInput::ToggleSearching);

            //     self.searching = !self.searching;
            // }

            // MainWindowMsg::SetShowSearch(state) => {
            //     self.show_search = state;
            // }

            // MainWindowMsg::SetShowBack(state) => {
            //     self.show_back = state;
            // }

            // MainWindowMsg::GoBack => {
            //     self.show_back = false;

            //     // Navigate back only on the visible page
            //     if let Some(name) = self.view_stack.visible_child_name() {
            //         match name.as_str() {
            //             "store"   => self.store_page.emit(StorePageInput::HideGamePage),
            //             // "library" => self.library_page.emit(LibraryPageInput::ToggleDownloadsPage),

            //             _ => ()
            //         }
            //     }
            // }

            // MainWindowMsg::ActivateStorePage => {
            //     self.store_page.emit(StorePageInput::Activate);
            // }

            // MainWindowMsg::ActivateLibraryPage => {
            //     if let Some(library_page) = self.library_page.as_ref() {
            //         library_page.emit(LibraryPageInput::Activate);
            //     }
            // }
        }
    }
}
