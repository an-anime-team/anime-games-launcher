use std::collections::HashMap;
use std::sync::Arc;

use adw::prelude::*;
use relm4::prelude::*;

use crate::prelude::*;

pub mod actions;

pub use actions::prelude::*;

// pub mod downloads_page;
pub mod library_page;
pub mod store_page;

// use downloads_page::*;
use library_page::*;
use store_page::*;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum MainWindowMsg {
    SetLoadingAction(String),
    FinishLoading(GenerationManifest),

    AddGame {
        url: String,
        manifest: GameManifest
    },

    ToggleSearching,
    SetShowSearch(bool),
    SetShowBack(bool),
    GoBack,

    ActivateStorePage,
    ActivateLibraryPage
}

pub struct MainWindow {
    store_page: AsyncController<StorePage>,
    library_page: Option<AsyncController<LibraryPage>>,

    window: Option<adw::ApplicationWindow>,
    view_stack: adw::ViewStack,

    games: HashMap<String, Arc<GameManifest>>,

    is_loading: bool,
    loading_action: Option<String>,

    show_search: bool,
    searching: bool,

    show_back: bool
}

impl MainWindow {
    pub fn window(&self) -> &adw::ApplicationWindow {
        self.window.as_ref().expect("Failed to load application window")
    }
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for MainWindow {
    type Init = ();
    type Input = MainWindowMsg;
    type Output = ();

    view! {
        #[root]
        window = adw::ApplicationWindow {
            set_title: Some("Anime Games Launcher"),

            set_size_request: (1200, 800),
            set_hide_on_close: false,

            add_css_class?: APP_DEBUG.then_some("devel"),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    #[watch]
                    set_visible: model.is_loading,

                    adw::HeaderBar {
                        add_css_class: "flat"
                    },

                    adw::StatusPage {
                        set_vexpand: true,
                        set_hexpand: true,

                        set_icon_name: Some(APP_ID),

                        set_title: "Loading",

                        #[watch]
                        set_description: model.loading_action.as_deref()
                    }
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    #[watch]
                    set_visible: !model.is_loading,

                    adw::HeaderBar {
                        pack_start = &gtk::Button {
                            set_icon_name: "loupe-symbolic",
                            add_css_class: "flat",

                            #[watch]
                            set_visible: model.show_search && !model.show_back,

                            connect_clicked => MainWindowMsg::ToggleSearching
                        },

                        pack_start = &gtk::Button {
                            set_icon_name: "go-previous-symbolic",
                            add_css_class: "flat",

                            #[watch]
                            set_visible: model.show_back,

                            connect_clicked => MainWindowMsg::GoBack
                        },

                        #[wrap(Some)]
                        set_title_widget = &adw::ViewSwitcher {
                            set_policy: adw::ViewSwitcherPolicy::Wide,

                            set_stack: Some(view_stack)
                        }
                    },

                    #[local_ref]
                    view_stack -> adw::ViewStack {
                        add = &gtk::Box {
                            set_vexpand: true,
                            set_hexpand: true,

                            model.store_page.widget(),
                        } -> {
                            set_title: Some("Store"),
                            set_name: Some("store"),
                            set_icon_name: Some("folder-download-symbolic")
                        },

                        #[name(library_page_box)]
                        add = &gtk::Box {
                            set_vexpand: true,
                            set_hexpand: true
                        } -> {
                            set_title: Some("Library"),
                            set_name: Some("library"),
                            set_icon_name: Some("applications-games-symbolic")
                        },

                        connect_visible_child_notify[sender] => move |stack| {
                            if let Some(name) = stack.visible_child_name() {
                                sender.input(MainWindowMsg::SetShowSearch(
                                    ["store", "library"].contains(&name.as_str())
                                ));

                                match name.as_str() {
                                    "store" => sender.input(MainWindowMsg::ActivateStorePage),
                                    "library" => sender.input(MainWindowMsg::ActivateLibraryPage),

                                    _ => ()
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    async fn init(_init: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let mut model = Self {
            store_page: StorePage::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    StorePageOutput::SetShowBack(s) => MainWindowMsg::SetShowBack(s)
                }),

            library_page: None,

            window: None,
            view_stack: adw::ViewStack::new(),

            is_loading: true,
            loading_action: None,

            games: HashMap::new(),

            show_search: true,
            searching: false,

            show_back: false,
        };

        let view_stack = &model.view_stack;

        let widgets = view_output!();

        let library_page = LibraryPage::builder()
            .launch(widgets.window.clone())
            .forward(sender.input_sender(), |msg| match msg {
                LibraryPageOutput::SetShowBack(s) => MainWindowMsg::SetShowBack(s)
            });

        widgets.library_page_box.append(library_page.widget());

        model.window = Some(widgets.window.clone());
        model.library_page = Some(library_page);

        let task = tokio::spawn(async move {
            // Create default folders.
            tracing::debug!("Creating default folders");

            sender.input(MainWindowMsg::SetLoadingAction(String::from("Creating default folders")));

            tokio::try_join!(
                tokio::fs::create_dir_all(DATA_FOLDER.as_path()),
                tokio::fs::create_dir_all(CONFIG_FOLDER.as_path()),
                tokio::fs::create_dir_all(CACHE_FOLDER.as_path()),

                tokio::fs::create_dir_all(&STARTUP_CONFIG.packages.resources_store.path),
                tokio::fs::create_dir_all(&STARTUP_CONFIG.packages.modules_store.path),
                tokio::fs::create_dir_all(&STARTUP_CONFIG.packages.persist_store.path),
                tokio::fs::create_dir_all(&STARTUP_CONFIG.packages.temp_store.path),
                tokio::fs::create_dir_all(&STARTUP_CONFIG.generations.store.path)
            )?;

            // Update the config file to create it
            // if it didn't exist before.
            config::update(&STARTUP_CONFIG)?;

            // Start fetching games manifests for the store page.
            {
                let sender = sender.clone();

                tokio::spawn(fetch_games(move |url, manifest| {
                    sender.input(MainWindowMsg::AddGame { url, manifest });
                }));
            }

            // Open generations and packages stores.
            tracing::debug!(
                generations_store = ?STARTUP_CONFIG.generations.store.path,
                packages_store = ?STARTUP_CONFIG.packages.resources_store.path,
                "Opening generations and packages stores"
            );

            sender.input(MainWindowMsg::SetLoadingAction(String::from("Opening generations and packages stores")));

            let generations_store = GenerationsStore::new(&STARTUP_CONFIG.generations.store.path);
            let packages_store = PackagesStore::new(&STARTUP_CONFIG.packages.resources_store.path);

            // List all available generations.
            tracing::debug!("Listing available generations");

            sender.input(MainWindowMsg::SetLoadingAction(String::from("Listing available generations")));

            let mut generations = generations_store.list()?.unwrap_or_default();

            let mut games = None;
            let mut valid_generation = None;

            // Iterate over available generations, from newest to oldest,
            // and try to load them.
            while let Some(generation) = generations.pop() {
                tracing::debug!(hash = generation.to_base32(), "Trying to load generation");

                sender.input(MainWindowMsg::SetLoadingAction(String::from("Loading generation")));

                // Try to load the generation file.
                let Some(generation) = generations_store.load(&generation)? else {
                    tracing::warn!("Generation is missing");

                    continue;
                };

                // Save the added games.
                if games.is_none() {
                    games = Some(generation.games.iter()
                        .map(|game| game.url.clone())
                        .collect::<Vec<_>>());
                }

                // Validate the generation.
                tracing::debug!("Validating generation resources");

                sender.input(MainWindowMsg::SetLoadingAction(String::from("Validating generation")));

                if !packages_store.validate(&generation.lock_file)? {
                    tracing::warn!("Generation is invalid");

                    continue;
                }

                // Store the valid generation for future use.
                valid_generation = Some(generation);

                break;
            }

            // Start building the new generation with potentially updated games info.
            let new_generation_task = tokio::spawn(async move {
                let generation = games.map(Generation::new).unwrap_or_default();

                tracing::debug!(?generation, "Building new generation");

                let generation = generation.build(&packages_store, &generations_store).await
                    .map_err(|err| anyhow::anyhow!(err.to_string()));

                match generation {
                    Ok(generation) => {
                        tracing::debug!("Indexing new generation in the store");

                        generations_store.insert(&generation)?;

                        Ok::<_, anyhow::Error>(generation)
                    }

                    // We should not spawn any GUI handler here because it's done in the other place.
                    // There's a setting called lazy_load, and if we couldn't make a new generation
                    // in *background* thread - we shouldn't directly notify the user about it (or should we?).
                    // If the task is in *foreground* thread, however (when there's no generations or lazy_load = false)
                    // then this error will be handled later and displayed in GUI dialog.
                    Err(err) => {
                        tracing::error!(?err, "Failed to build new generation");

                        Err(err)
                    }
                }
            });

            // Resolve the generation.
            let valid_generation = match valid_generation {
                Some(generation) if STARTUP_CONFIG.generations.lazy_load => generation,

                // Make a new generation if no valid one was found
                // or lazy loading is disabled.
                _ => {
                    if STARTUP_CONFIG.generations.lazy_load {
                        tracing::debug!("No valid generation found, awaiting the new one");
                    } else {
                        tracing::debug!("Generations lazy loading is disabled. Awaiting new generation to build");
                    }

                    sender.input(MainWindowMsg::SetLoadingAction(String::from("Building new generation")));

                    new_generation_task.await??
                }
            };

            // Load the main window.
            tracing::debug!("Load main window");

            sender.input(MainWindowMsg::SetLoadingAction(String::from("Almost done")));
            sender.input(MainWindowMsg::FinishLoading(valid_generation));

            Ok::<_, anyhow::Error>(())
        });

        // Handle error from the above task.
        tokio::spawn(async move {
            match task.await {
                Ok(Ok(())) => (),

                Ok(Err(err)) => {
                    tracing::error!(?err, "Failed to execute startup task");

                    critical_error("Failed to execute startup task", err);
                }

                Err(err) => {
                    tracing::error!(?err, "Failed to execute startup task");

                    critical_error("Failed to execute startup task", err);
                }
            }
        });

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, _sender: AsyncComponentSender<Self>) {
        match message {
            MainWindowMsg::SetLoadingAction(action) => self.loading_action = Some(action),

            MainWindowMsg::FinishLoading(generation) => {
                if let Some(library_page) = self.library_page.as_ref() {
                    library_page.emit(LibraryPageInput::SetGeneration(generation));
                }

                self.is_loading = false;
            }

            MainWindowMsg::AddGame { url, manifest } => {
                let manifest = Arc::new(manifest);

                self.games.insert(url.clone(), manifest.clone());

                self.store_page.emit(StorePageInput::AddGame {
                    url,
                    manifest: manifest.clone()
                });
            }

            MainWindowMsg::ToggleSearching => {
                self.store_page.emit(StorePageInput::ToggleSearching);

                self.searching = !self.searching;
            }

            MainWindowMsg::SetShowSearch(state) => {
                self.show_search = state;
            }

            MainWindowMsg::SetShowBack(state) => {
                self.show_back = state;
            }

            MainWindowMsg::GoBack => {
                self.show_back = false;

                // Navigate back only on the visible page
                if let Some(name) = self.view_stack.visible_child_name() {
                    match name.as_str() {
                        "store"   => self.store_page.emit(StorePageInput::HideGamePage),
                        // "library" => self.library_page.emit(LibraryPageInput::ToggleDownloadsPage),

                        _ => ()
                    }
                }
            }

            MainWindowMsg::ActivateStorePage => {
                self.store_page.emit(StorePageInput::Activate);
            }

            MainWindowMsg::ActivateLibraryPage => {
                if let Some(library_page) = self.library_page.as_ref() {
                    library_page.emit(LibraryPageInput::Activate);
                }
            }
        }
    }
}
