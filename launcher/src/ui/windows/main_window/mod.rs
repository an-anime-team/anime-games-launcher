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
use std::process::Command;

use adw::prelude::*;

use relm4::prelude::*;
use relm4::actions::*;

use serde_json::Value as Json;
use anyhow::Context;

use agl_core::tasks;
use agl_core::network::downloader::{Downloader, DownloadOptions};
use agl_locale::LocalizableString;
use agl_packages::storage::Storage;
use agl_runtime::mlua::prelude::*;
use agl_runtime::allow_list::AllowList;
use agl_runtime::api::ApiOptions;
use agl_runtime::api::portal_api::{
    ToastOptions, NotificationOptions, DialogOptions, DialogButtonStatus
};
use agl_runtime::runtime::{Runtime, ModulePaths};
use agl_games::manifest::{GamesRegistryManifest, GameManifest};
use agl_games::engine::{
    GameVariant,
    GameIntegration,
    ActionsPipeline,
    GameLaunchInfo,
    GameSettingsGroup
};

use crate::consts;
use crate::config;
use crate::cache;
use crate::games::GameLock;
use crate::ui::dialogs;
use crate::ui::windows::about::AboutWindow;
use crate::ui::windows::game_settings::{
    GameSettingsWindow,
    GameSettingsWindowInput,
    GameSettingsWindowOutput
};
use crate::ui::windows::pipeline_actions::{
    PipelineActionsWindow,
    PipelineActionsWindowInput,
    PipelineActionsWindowOutput
};
use crate::ui::windows::game_running::{GameRunningWindow, GameRunningWindowMsg};

pub mod store_page;
pub mod library_page;

use store_page::{StorePage, StorePageInput, StorePageOutput};
use library_page::{LibraryPage, LibraryPageInput, LibraryPageOutput};

relm4::new_action_group!(WindowActionGroup, "win");

// relm4::new_stateless_action!(LauncherFolder, WindowActionGroup, "launcher_folder");
// relm4::new_stateless_action!(GameFolder, WindowActionGroup, "game_folder");
// relm4::new_stateless_action!(ConfigFile, WindowActionGroup, "config_file");
// relm4::new_stateless_action!(DebugFile, WindowActionGroup, "debug_file");

relm4::new_stateless_action!(About, WindowActionGroup, "about");

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum MainWindowMsg {
    OpenAboutWindow,

    SetLoadingStatus(Option<String>),

    AddAllowList(AllowList),

    AddStorePageGame {
        manifest_url: String,
        manifest: GameManifest
    },

    AddLibraryPageGame(GameLock),

    ShowToast(ToastOptions),
    ShowNotification(NotificationOptions),
    ShowDialog(DialogOptions),

    SetShowBackButton(bool),
    GoBackButtonClicked,

    ShowLibraryGameWithUrl(String),

    ScheduleGameActionsPipeline {
        game_index: usize,
        game_title: String,
        actions_pipeline: Arc<ActionsPipeline>
    },

    OpenGameSettingsWindow {
        variant: GameVariant,
        integration: Arc<GameIntegration>,
        layout: Box<[GameSettingsGroup]>
    },

    LaunchGame {
        game_title: String,
        game_launch_info: GameLaunchInfo
    },

    ReloadSelectedLibraryGameInfo
}

pub struct MainWindow {
    about_window: AsyncController<AboutWindow>,
    store_page: AsyncController<StorePage>,
    library_page: AsyncController<LibraryPage>,
    game_settings_window: AsyncController<GameSettingsWindow>,
    pipeline_actions_window: AsyncController<PipelineActionsWindow>,
    game_running_window: AsyncController<GameRunningWindow>,

    window: Option<adw::ApplicationWindow>,
    toast_overlay: adw::ToastOverlay,
    view_stack: adw::ViewStack,

    loading_status: Option<String>,

    show_back_button: bool,

    storage: Storage,
    runtime: Runtime,
    allow_list: AllowList
}

impl std::fmt::Debug for MainWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LibraryPage")
            .field("about_window", &self.about_window)
            .field("store_page", &self.store_page)
            .field("library_page", &self.library_page)
            .field("game_settings_window", &self.game_settings_window)
            .field("pipeline_actions_window", &self.pipeline_actions_window)
            .field("game_running_window", &self.game_running_window)
            .field("window", &self.window)
            .field("toast_overlay", &self.toast_overlay)
            .field("view_stack", &self.view_stack)
            .field("loading_status", &self.loading_status)
            .field("show_back_button", &self.show_back_button)
            .field("storage", &self.storage)
            .field("runtime", &"Runtime")
            .field("allow_list", &self.allow_list)
            .finish()
    }
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for MainWindow {
    type Init = ();
    type Input = MainWindowMsg;
    type Output = ();

    menu! {
        main_menu: {
            section! {
                "About" => About
            }
        }
    }

    view! {
        #[root]
        _window = adw::ApplicationWindow {
            set_title: Some("Anime Games Launcher"),
            set_size_request: (1200, 800),

            add_css_class?: consts::APP_DEBUG.then_some("devel"),

            #[local_ref]
            _toast_overlay -> adw::ToastOverlay {
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
                            pack_start = &gtk::Button {
                                set_icon_name: "go-previous-symbolic",
                                add_css_class: "flat",

                                #[watch]
                                set_visible: model.show_back_button,

                                connect_clicked => MainWindowMsg::GoBackButtonClicked
                            },

                            pack_end = &gtk::MenuButton {
                                set_icon_name: "open-menu-symbolic",
                                set_menu_model: Some(&main_menu)
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
                                set_icon_name: Some("shopping-cart-symbolic")
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
                        }
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
        let config = config::startup();

        let lua = Lua::new();

        // Set runtime memory limit.
        if config.runtime_memory_limit > 0 {
            lua.set_memory_limit(config.runtime_memory_limit)
                .expect("failed to set packages runtime memory limit");
        }

        let client = config.client_builder()
            .and_then(|client| {
                client.build()
                    .map_err(|err| anyhow::anyhow!(err))
            })
            .expect("failed to build network client");

        let storage = Storage::open(&config.packages_resources_path)
            .expect("failed to open packages storage");

        fn translate(str: LocalizableString) -> String {
            let config = config::get();

            let str = match config.language() {
                Ok(lang) => str.translate(&lang),
                Err(_) => str.default_translation()
            };

            str.to_string()
        }

        let runtime = Runtime::new(ApiOptions {
            lua,
            client,
            translate,

            show_toast: {
                let sender = sender.clone();

                Box::new(move |options| {
                    sender.input(MainWindowMsg::ShowToast(options));
                })
            },

            show_notification: {
                let sender = sender.clone();

                Box::new(move |options| {
                    sender.input(MainWindowMsg::ShowNotification(options));
                })
            },

            show_dialog: {
                let sender = sender.clone();

                Box::new(move |options| {
                    sender.input(MainWindowMsg::ShowDialog(options));

                    // FIXME
                    None
                })
            },
        }).expect("failed to initialize packages runtime");

        let mut model = Self {
            about_window: AboutWindow::builder()
                .launch(())
                .detach(),

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
                    LibraryPageOutput::ScheduleGameActionsPipeline { game_index, game_title, actions_pipeline }
                        => MainWindowMsg::ScheduleGameActionsPipeline { game_index, game_title, actions_pipeline },

                    LibraryPageOutput::OpenGameSettingsWindow { variant, integration, layout }
                        => MainWindowMsg::OpenGameSettingsWindow { variant, integration, layout },

                    LibraryPageOutput::LaunchGame { game_title, game_launch_info }
                        => MainWindowMsg::LaunchGame { game_title, game_launch_info }
                }),

            game_settings_window: GameSettingsWindow::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    GameSettingsWindowOutput::ReloadGameInfo
                        => MainWindowMsg::ReloadSelectedLibraryGameInfo
                }),

            pipeline_actions_window: PipelineActionsWindow::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    PipelineActionsWindowOutput::UpdateGameInfo(_)
                        => MainWindowMsg::ReloadSelectedLibraryGameInfo
                }),

            game_running_window: GameRunningWindow::builder()
                .launch(())
                .detach(),

            window: None,
            toast_overlay: adw::ToastOverlay::new(),
            view_stack: adw::ViewStack::new(),

            loading_status: Some(String::new()),

            show_back_button: false,

            storage,
            runtime,
            allow_list: AllowList::default()
        };

        // Named like this to supress relm4 view macro warning.
        let _toast_overlay = &model.toast_overlay;
        let _view_stack = &model.view_stack;

        let widgets = view_output!();

        model.window = Some(widgets._window.clone());

        // Connect hamburger menu buttons.
        let mut group = RelmActionGroup::<WindowActionGroup>::new();

        {
            let sender = sender.clone();

            group.add_action::<About>(RelmAction::new_stateless(move |_| {
                sender.input(MainWindowMsg::OpenAboutWindow);
            }));
        }

        widgets._window.insert_action_group(
            "win",
            Some(&group.into_action_group())
        );

        // Spawn startup task.
        let task = tasks::spawn(async move {
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

            let config = config::startup();
            let lang = config.language();

            // Create network client from config file.
            let client = config::startup()
                .client_builder()
                .context("failed to create network client from the config values")?
                .build()
                .context("failed to build network client")?;

            // Prepare downloader and files cache.
            let downloader = Downloader::from_client(client);

            // Fetch packages allow lists.

            tracing::debug!(
                registries = ?config.games_registries,
                "fetching allow lists"
            );

            sender.input(MainWindowMsg::SetLoadingStatus(
                Some(String::from("Fetching allow lists"))
            ));

            let mut tasks = Vec::with_capacity(config.packages_allow_lists.len());
            let mut paths = Vec::with_capacity(tasks.capacity());

            // Either fetch package allow list or use cached one.
            for url in &config.packages_allow_lists {
                let cache_path = cache::get_path(url);

                tracing::trace!(?url, ?cache_path, "fetching packages allow list");

                // If cache for this allow list is expired - request the list
                // again.
                let is_expired = cache::is_expired(
                    &cache_path,
                    config.cache_packages_allow_lists_duration
                )?;

                if is_expired {
                    tracing::trace!(?url, ?cache_path, "packages allow list cache is expired");

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

            // Wait for all the allow lists to be downloaded.
            for (url, path, task) in tasks {
                tracing::trace!(?url, ?path, "awaiting packages allow list downloading");

                let result = task.wait().await
                    .context("failed to await packages allow list fetching");

                if let Err(err) = result {
                    // Remove half-downloaded/broken file.
                    let _ = std::fs::remove_file(path);

                    return Err(err);
                }
            }

            for path in paths {
                tracing::trace!(?path, "reading packages allow list");

                let allow_list = std::fs::read(path)?;
                let allow_list = serde_json::from_slice::<Json>(&allow_list)?;

                let allow_list = AllowList::from_json(&allow_list)
                    .context("failed to deserialize packages allow list")?;

                sender.input(MainWindowMsg::AddAllowList(allow_list));
            }

            // Fetch game registries.

            tracing::debug!(
                registries = ?config::startup().games_registries,
                "fetching game registries"
            );

            sender.input(MainWindowMsg::SetLoadingStatus(
                Some(String::from("Fetching game registries"))
            ));

            let mut tasks = Vec::with_capacity(config.games_registries.len());
            let mut paths = Vec::with_capacity(tasks.capacity());

            // Either fetch game registry manifest or use cached one.
            for url in &config.games_registries {
                let cache_path = cache::get_path(url);

                tracing::trace!(?url, ?cache_path, "fetching game registry");

                // If cache for this registry is expired - request the registry
                // value again.
                let is_expired = cache::is_expired(
                    &cache_path,
                    config.cache_game_registries_duration
                )?;

                if is_expired {
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
                let is_expired = cache::is_expired(
                    &cache_path,
                    config.cache_game_manifests_duration
                )?;

                if is_expired {
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

            let storage = Storage::open(&config::startup().packages_resources_path)
                .context("failed to open packages storage")?;

            for entry in config.games_path.read_dir()? {
                let entry = entry?;

                tracing::trace!(
                    path = ?entry.path(),
                    "loading added game package lock"
                );

                let lock = std::fs::read(entry.path())?;
                let lock = serde_json::from_slice::<Json>(&lock)?;

                let mut lock = GameLock::from_json(&lock)
                    .context("failed to deserialize game package lock")?;

                let title = match &lang {
                    Ok(lang) => lock.manifest.game.title.translate(lang),
                    Err(_) => lock.manifest.game.title.default_translation()
                };

                sender.input(MainWindowMsg::SetLoadingStatus(
                    Some(format!("Loading {title} game package"))
                ));

                let is_expired = cache::is_expired(
                    entry.path(),
                    config.cache_game_packages_duration
                )?;

                if is_expired {
                    tracing::trace!(
                        path = ?entry.path(),
                        ?title,
                        "updating added game package lock, cache is expired"
                    );

                    sender.input(MainWindowMsg::SetLoadingStatus(
                        Some(format!("Updating {title} game package"))
                    ));

                    let prev_scope = lock.scope;

                    lock = GameLock::download(&lock.url, &storage).await
                        .context("failed to update game package lock")?;

                    lock.scope = prev_scope;

                    std::fs::write(
                        entry.path(),
                        serde_json::to_vec_pretty(&lock.to_json())?
                    )?;
                }

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
        tasks::spawn(async move {
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
            MainWindowMsg::OpenAboutWindow => {
                if let Some(window) = &self.window {
                    self.about_window.widget().present(Some(window));
                }
            }

            MainWindowMsg::SetLoadingStatus(status) => self.loading_status = status,

            MainWindowMsg::AddAllowList(allow_list) => {
                self.allow_list.merge_with(allow_list);
            }

            MainWindowMsg::AddStorePageGame { manifest_url, manifest } => {
                self.store_page.emit(StorePageInput::AddGame {
                    manifest_url,
                    manifest
                });
            }

            MainWindowMsg::AddLibraryPageGame(game) => {
                let config = config::get();

                let lang = config.language();

                let title = match &lang {
                    Ok(lang) => game.manifest.game.title.translate(lang),
                    Err(_) => game.manifest.game.title.default_translation()
                };

                let paths = ModulePaths {
                    temp_folder: config.packages_temporary_path.clone(),
                    modules_folder: config.packages_modules_path.clone(),
                    persistent_folder: config.packages_persistent_path.clone()
                };

                // Add game's scope to all the game integration resources.
                if let Some(scope) = &game.scope {
                    for hash in game.lock.resources.keys() {
                        self.allow_list.add_module_scope(*hash, scope.clone());
                    }
                }

                // Load the game integration.
                let result = self.runtime.load_packages(
                    &game.lock,
                    &self.storage,
                    &paths,
                    &self.allow_list
                );

                if let Err(err) = result {
                    tracing::error!(
                        ?err,
                        url = game.url,
                        title = game.manifest.game.title.default_translation(),
                        "failed to load game package"
                    );

                    dialogs::error(
                        format!("Failed to load {title} game package"),
                        err.to_string()
                    );

                    return;
                }

                fn find_module_key(lock: &GameLock) -> Option<String> {
                    for hash in &lock.lock.root {
                        #[allow(clippy::collapsible_if)]
                        if let Some(package) = lock.lock.packages.get(hash) {
                            if let Some(output) = package.outputs.get(&lock.manifest.package.output) {
                                // TODO: can change in future. Better make some
                                //       universal solution.
                                let module_key = format!("{}#module", output.hash.to_base32());

                                return Some(module_key);
                            }
                        }
                    }

                    None
                }

                let Some(module_key) = find_module_key(&game) else {
                    tracing::error!(
                        url = game.url,
                        title = game.manifest.game.title.default_translation(),
                        "failed to find game integration module in package lock"
                    );

                    dialogs::error(
                        "Failed to find game integration module in package lock",
                        format!("Attempted to find {title} game integration module, but it's missing in the package lock. Perhaps the lock file is broken")
                    );

                    return;
                };

                let game_integration = self.runtime.get_value::<LuaTable>(module_key)
                    .transpose()
                    .map(|game_integration| {
                        game_integration.and_then(|game_integration| {
                            game_integration.raw_get::<LuaTable>("value")
                        })
                    });

                let game_integration = match game_integration {
                    Some(Ok(game_integration)) => game_integration,

                    Some(Err(err)) => {
                        tracing::error!(
                            ?err,
                            url = game.url,
                            title = game.manifest.game.title.default_translation(),
                            "failed to read game integration from the runtime"
                        );

                        dialogs::error(
                            format!("Failed to read {title} game integration from the runtime"),
                            err.to_string()
                        );

                        return;
                    }

                    None => {
                        tracing::error!(
                            url = game.url,
                            title = game.manifest.game.title.default_translation(),
                            "game integration module is missing in the runtime"
                        );

                        dialogs::error(
                            "Game integration module is missing in the runtime",
                            format!("Attempted to load {title} game integration, but integration module is missing in the packages runtime")
                        );

                        return;
                    }
                };

                let game_integration = GameIntegration::from_lua(
                    self.runtime.lua().clone(),
                    &game_integration
                );

                let game_integration = match game_integration {
                    Ok(game_integration) => Arc::new(game_integration),

                    Err(err) => {
                        tracing::error!(
                            ?err,
                            url = game.url,
                            title = game.manifest.game.title.default_translation(),
                            "failed to build game integration"
                        );

                        dialogs::error(
                            format!("Failed to build {title} game integration"),
                            err.to_string()
                        );

                        return;
                    }
                };

                self.library_page.emit(LibraryPageInput::AddGame {
                    package: game,
                    integration: game_integration
                });
            }

            MainWindowMsg::ShowToast(options) => {
                let lang = config::get().language();

                let title = match &options {
                    ToastOptions::Simple(title) => title,
                    ToastOptions::Activatable { message, .. } => message
                };

                let title = match &lang {
                    Ok(lang) => title.translate(lang),
                    Err(_) => title.default_translation()
                };

                let toast = adw::Toast::new(title);

                if let ToastOptions::Activatable { label, callback, .. } = options {
                    let label = match &lang {
                        Ok(lang) => label.translate(lang),
                        Err(_) => label.default_translation()
                    };

                    toast.set_button_label(Some(label));

                    toast.connect_button_clicked(move |_| {
                        if let Err(err) = callback.call::<()>(()) {
                            tracing::error!(?err, "failed to execute toast action");

                            dialogs::error("Failed to execute toast action", err.to_string());
                        }
                    });
                }

                self.toast_overlay.add_toast(toast);
            }

            MainWindowMsg::ShowNotification(options) => {
                let lang = config::get().language();

                let title = match &lang {
                    Ok(lang) => options.title.translate(lang),
                    Err(_) => options.title.default_translation()
                };

                let mut notification = notify_rust::Notification::new();
                let mut notification = notification.summary(title);

                if let Some(message) = options.message {
                    let message = match &lang {
                        Ok(lang) => message.translate(lang),
                        Err(_) => message.default_translation()
                    };

                    notification = notification.body(message);
                }

                if let Some(icon) = options.icon {
                    notification = notification.icon(&icon);
                }

                if let Err(err) = notification.show() {
                    tracing::error!(?err, "failed to show system notification");

                    dialogs::error("Failed to show system notification", err.to_string());
                }
            }

            MainWindowMsg::ShowDialog(options) => {
                let lang = config::get().language();

                let title = match &lang {
                    Ok(lang) => options.title.translate(lang),
                    Err(_) => options.title.default_translation()
                };

                let message = match &lang {
                    Ok(lang) => options.message.translate(lang),
                    Err(_) => options.message.default_translation()
                };

                let dialog = adw::AlertDialog::new(
                    Some(title),
                    Some(message)
                );

                if options.can_close || options.buttons.is_empty() {
                    dialog.add_response("close", "Close");

                    dialog.connect_response(Some("close"), |dialog, _| {
                        dialog.close();
                    });
                }

                for (i, button) in options.buttons.into_iter().enumerate() {
                    let name = format!("response_{i}");

                    let label = match &lang {
                        Ok(lang) => button.label.translate(lang),
                        Err(_) => button.label.default_translation()
                    };

                    dialog.connect_response(Some(&name), move |dialog, _| {
                        if let Err(err) = button.callback.call::<()>(()) {
                            tracing::error!(?err, "failed to execute dialog action");

                            dialogs::error("Failed to execute dialog action", err.to_string());
                        }

                        dialog.close();
                    });

                    dialog.add_response(&name, label);

                    dialog.set_response_appearance(&name, match button.status {
                        DialogButtonStatus::Normal    => adw::ResponseAppearance::Default,
                        DialogButtonStatus::Suggested => adw::ResponseAppearance::Suggested,
                        DialogButtonStatus::Dangerous => adw::ResponseAppearance::Destructive
                    });
                }

                dialog.present(self.window.as_ref());
            }

            MainWindowMsg::SetShowBackButton(show) => self.show_back_button = show,

            MainWindowMsg::GoBackButtonClicked => {
                self.store_page.emit(StorePageInput::CloseGameDetails);
            }

            MainWindowMsg::ShowLibraryGameWithUrl(url) => {
                self.view_stack.set_visible_child_name("library");

                self.library_page.emit(LibraryPageInput::SelectGameWithUrl(url));
            }

            MainWindowMsg::ScheduleGameActionsPipeline {
                game_index,
                game_title,
                actions_pipeline
            } => {
                if let Some(window) = &self.window {
                    self.pipeline_actions_window.emit(PipelineActionsWindowInput::SetActionsPipeline {
                        game_index,
                        game_title,
                        actions_pipeline
                    });

                    self.pipeline_actions_window.widget()
                        .present(Some(window));
                }
            }

            MainWindowMsg::OpenGameSettingsWindow {
                variant,
                integration,
                layout
            } => {
                if let Some(window) = &self.window {
                    self.game_settings_window.emit(GameSettingsWindowInput::SetGame {
                        variant,
                        integration,
                        layout
                    });

                    self.game_settings_window.widget()
                        .present(Some(window));
                }
            }

            MainWindowMsg::LaunchGame { game_title, game_launch_info } => {
                if let Some(window) = &self.window {
                    let mut command = &mut Command::new(&game_launch_info.binary);

                    if let Some(args) = &game_launch_info.args {
                        command = command.args(args);
                    }

                    if let Some(env) = &game_launch_info.env {
                        command = command.envs(env);
                    }

                    // TODO: pipe stdout/stderr to a log file.

                    tracing::info!(?command, "launching game");

                    match command.spawn() {
                        Ok(child) => {
                            self.game_running_window.emit(GameRunningWindowMsg::SetChild {
                                game_title,
                                child
                            });

                            self.game_running_window.widget()
                                .present(Some(window));
                        }

                        Err(err) => tracing::error!(?err, "failed to launch game")
                    }
                }
            }

            MainWindowMsg::ReloadSelectedLibraryGameInfo => {
                self.library_page.emit(LibraryPageInput::UpdateSelectedGameInfo);
            }
        }
    }
}
