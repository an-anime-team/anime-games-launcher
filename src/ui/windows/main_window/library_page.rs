use std::sync::Arc;

use adw::prelude::*;
use relm4::prelude::*;

use mlua::prelude::*;

use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::oneshot::Sender as OneshotSender;

use tokio::runtime::{
    Runtime,
    Builder as RuntimeBuilder
};

use unic_langid::LanguageIdentifier;

use crate::prelude::*;
use crate::ui::components::*;

use super::DownloadsPageApp;

lazy_static::lazy_static! {
    static ref RUNTIME: Runtime = RuntimeBuilder::new_current_thread()
        .thread_name("games-daemon")
        .enable_all()
        .build()
        .expect("Failed to create games integrations daemon");
}

#[derive(Debug)]
pub enum SyncGameCommand {
    GetEditions(OneshotSender<Vec<GameEdition>>),
    // GetComponents(OneshotSender<Vec<GameComponent<'lua>>>)
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum LibraryPageInput {
    SetGeneration(GenerationManifest),

    AddGameFromGeneration {
        url: String,
        manifest: GameManifest,
        listener: UnboundedSender<SyncGameCommand>
    },

    Activate,
    ShowGameDetails(DynamicIndex),
    ToggleDownloadsPage
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryPageOutput {
    SetShowBack(bool)
}

pub struct LibraryPage {
    cards_list: AsyncFactoryVecDeque<CardsList>,
    game_details: AsyncController<GameDetails>,
    active_download: AsyncController<DownloadsRow>,
    downloads_page: AsyncController<DownloadsPageApp>,

    games: Vec<(String, Arc<GameManifest>, UnboundedSender<SyncGameCommand>)>,

    show_downloads: bool
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for LibraryPage {
    type Init = ();
    type Input = LibraryPageInput;
    type Output = LibraryPageOutput;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            #[transition(SlideLeftRight)]
            append = if !model.show_downloads {
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    adw::NavigationSplitView {
                        set_vexpand: true,
                        set_hexpand: true,

                        #[wrap(Some)]
                        set_sidebar = &adw::NavigationPage {
                            // Supress Adwaita-WARNING **: AdwNavigationPage is missing a title
                            set_title: "Games",

                            #[wrap(Some)]
                            set_child = model.cards_list.widget() {
                                add_css_class: "navigation-sidebar"
                            }
                        },

                        #[wrap(Some)]
                        set_content = &adw::NavigationPage {
                            set_hexpand: true,

                            // Supress Adwaita-WARNING **: AdwNavigationPage is missing a title
                            set_title: "Details",

                            #[wrap(Some)]
                            set_child = model.game_details.widget(),
                        }
                    },

                    adw::PreferencesPage {
                        adw::PreferencesGroup {
                            model.active_download.widget() {
                                set_width_request: 1000,

                                set_activatable: true,

                                connect_activated => LibraryPageInput::ToggleDownloadsPage
                            }
                        }
                    }
                }
            } else {
                gtk::Box {
                    model.downloads_page.widget(),
                }
            }
        }
    }

    async fn init(_init: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let model = Self {
            cards_list: AsyncFactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |msg| match msg {
                    CardsListOutput::Selected(index) => LibraryPageInput::ShowGameDetails(index)
                }),

            game_details: GameDetails::builder()
                .launch(())
                .detach(),

            active_download: DownloadsRow::builder()
                .launch(DownloadsRowInit::new(
                    "123",
                    String::from("Punishing: Gray Raven"),
                    String::from("69.42.0"),
                    String::from("Global"),
                    696969696969,
                    true,
                ))
                .detach(),

            downloads_page: DownloadsPageApp::builder()
                .launch(())
                .detach(),

            games: Vec::new(),

            show_downloads: false
        };

        model.cards_list.widget().connect_row_selected(|_, row| {
            if let Some(row) = row {
                row.emit_activate();
            }
        });

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            LibraryPageInput::SetGeneration(generation) => {
                let config = config::get();

                let packages_store = PackagesStore::new(config.packages.resources_store.path);

                std::thread::spawn(move || {
                    let lua = Lua::new();

                    // Iterate through locked resources and find manifests
                    // for appropriate games packages.
                    let mut games_resources = Vec::with_capacity(generation.games.len());

                    for game in generation.games {
                        let resource = generation.lock_file.resources.iter()
                            .find(|resource| game.manifest.package.url == resource.url);

                        if let Some(resource) = resource {
                            games_resources.push((game, resource.clone()));
                        }
                    }

                    // Load generation's lock file into the packages engine.
                    let engine = match PackagesEngine::create(&lua, &packages_store, generation.lock_file) {
                        Ok(engine) => engine,
                        Err(err) => {
                            tracing::error!(?err, "Failed to load locked packages to the lua engine");

                            return;
                        }
                    };

                    // Prepare games engines for locked games.
                    let mut games = Vec::with_capacity(games_resources.len());

                    for (game, resource) in games_resources {
                        tracing::trace!(
                            game = game.manifest.game.title.default_translation(),
                            manifest = resource.url,
                            "Trying to load the game engine"
                        );

                        let Some(integration_resource) = resource.outputs.and_then(|outputs| outputs.get(&game.manifest.package.output).copied()) else {
                            tracing::error!(
                                game = game.manifest.game.title.default_translation(),
                                manifest = resource.url,
                                output = game.manifest.package.output,
                                "Game package doesn't have requested output"
                            );

                            continue;
                        };

                        let module = match engine.load_resource(integration_resource) {
                            Ok(Some(module)) => match module.get::<_, LuaTable>("value") {
                                Ok(module) => module,
                                Err(err) => {
                                    tracing::error!(
                                        game = game.manifest.game.title.default_translation(),
                                        manifest = resource.url,
                                        ?integration_resource,
                                        ?err,
                                        "Failed to get lua table of the game integration"
                                    );

                                    continue;
                                }
                            }

                            Ok(None) => {
                                tracing::error!(
                                    game = game.manifest.game.title.default_translation(),
                                    manifest = resource.url,
                                    ?integration_resource,
                                    "Failed to load game integration module from the lua engine"
                                );

                                continue;
                            }

                            Err(err) => {
                                tracing::error!(
                                    game = game.manifest.game.title.default_translation(),
                                    manifest = resource.url,
                                    ?integration_resource,
                                    ?err,
                                    "Failed to load game integration module from the lua engine"
                                );

                                continue;
                            }
                        };

                        let engine = match GameEngine::from_lua(&lua, &module) {
                            Ok(engine) => engine,
                            Err(err) => {
                                tracing::error!(?err, "Failed to create game integration engine from the loaded package");

                                continue;
                            }
                        };

                        let (listener, receiver) = tokio::sync::mpsc::unbounded_channel();

                        tracing::debug!(
                            url = game.url,
                            title = game.manifest.game.title.default_translation(),
                            "Loaded game integration engine"
                        );

                        sender.input(LibraryPageInput::AddGameFromGeneration {
                            url: game.url,
                            manifest: game.manifest,
                            listener
                        });

                        games.push((engine, receiver, true));
                    }

                    loop {
                        let mut has_working = false;

                        for (game, listener, working) in &mut games {
                            if *working {
                                match listener.try_recv() {
                                    Ok(command) => {
                                        match command {
                                            SyncGameCommand::GetEditions(listener) => {
                                                let _ = listener.send(game.editions().to_vec());
                                            }
                                        }
                                    }

                                    Err(TryRecvError::Empty) => (),
                                    Err(TryRecvError::Disconnected) => *working = false
                                }

                                has_working |= *working;
                            }
                        }

                        if !has_working {
                            break;
                        }
                    }
                });
            }

            LibraryPageInput::AddGameFromGeneration { url, manifest, listener } => {
                let config = config::get();

                let language = config.general.language.parse::<LanguageIdentifier>();

                self.cards_list.guard().push_back(CardsListInit {
                    image: ImagePath::LazyLoad(manifest.game.images.poster.clone()),

                    title: match language {
                        Ok(lang) => manifest.game.title.translate(&lang).to_string(),
                        Err(_) => manifest.game.title.default_translation().to_string()
                    }
                });

                self.games.push((url, Arc::new(manifest), listener));
            }

            LibraryPageInput::ShowGameDetails(index) => {
                if let Some(details) = self.cards_list.get(index.current_index()) {
                    todo!("{:?}", details);
                }
            }

            LibraryPageInput::ToggleDownloadsPage => {
                self.show_downloads = !self.show_downloads;
            }

            LibraryPageInput::Activate => {
                // Update back button visibility when switching pages
            }
        }
    }
}
