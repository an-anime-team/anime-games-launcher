use adw::prelude::*;
use relm4::prelude::*;

use mlua::prelude::*;

use crate::prelude::*;
use crate::ui::components::*;

use super::DownloadsPageApp;

thread_local! {
    static LUA_ENGINE: Lua = Lua::new();
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LibraryPageInput {
    SetGeneration(GenerationManifest),

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

    packages_store: PackagesStore,
    packages_engine: Option<PackagesEngine<'static>>,

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

            packages_store: PackagesStore::new(&STARTUP_CONFIG.packages.resources_store.path),
            packages_engine: None,

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
                LUA_ENGINE.with(|lua| {
                    let engine = match PackagesEngine::create(lua, &self.packages_store, generation.lock_file) {
                        Ok(engine) => engine,
                        Err(err) => {
                            tracing::error!(?err, "Failed to load locked packages to the lua engine");

                            return;
                        }
                    };

                    let root_modules = match engine.load_root_modules() {
                        Ok(modules) => modules,
                        Err(err) => {
                            tracing::error!(?err, "Failed to get loaded modules from the lua engine");

                            return;
                        }
                    };

                    for module in root_modules {
                        let module = match module.get::<_, LuaTable>("value") {
                            Ok(module) => module,
                            Err(err) => {
                                tracing::error!(?err, "Failed to get lua table of the game integration");

                                return;
                            }
                        };

                        let game = match GameEngine::from_lua(lua, &module) {
                            Ok(game) => game,
                            Err(err) => {
                                tracing::error!(?err, "Failed to create game integration engine from the loaded package");

                                return;
                            }
                        };

                        dbg!(game);
                    }
                });
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

        // Update back button visibility
        let _ = sender.output(LibraryPageOutput::SetShowBack(self.show_downloads));
    }
}
