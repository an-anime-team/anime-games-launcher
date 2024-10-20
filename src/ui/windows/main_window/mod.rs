use std::collections::HashMap;
use std::sync::Arc;

use adw::prelude::*;
use relm4::prelude::*;

use crate::prelude::*;

pub mod downloads_page;
pub mod library_page;
pub mod profile_page;
pub mod store_page;

use downloads_page::*;
use library_page::*;
use profile_page::*;
use store_page::*;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum MainWindowMsg {
    AddGamesRegistry {
        url: String,
        manifest: GamesRegistryManifest
    },

    AddGame {
        url: String,
        manifest: GameManifest
    },

    SetGeneration(GenerationManifest),
    OpenWindow,

    ToggleSearching,
    SetShowSearch(bool),
    SetShowBack(bool),
    GoBack,

    ActivateStorePage,
    ActivateLibraryPage,
}

pub struct MainWindow {
    store_page: AsyncController<StorePage>,
    library_page: AsyncController<LibraryPage>,
    profile_page: AsyncController<ProfilePageApp>,

    view_stack: adw::ViewStack,

    registries: HashMap<String, Arc<GamesRegistryManifest>>,
    games: HashMap<String, Arc<GameManifest>>,

    visible: bool,

    show_search: bool,
    searching: bool,

    show_back: bool
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for MainWindow {
    type Init = ();
    type Input = MainWindowMsg;
    type Output = ();

    view! {
        #[root]
        window = adw::Window {
            set_title: Some("Anime Games Launcher"),
            set_size_request: (1200, 800),

            add_css_class?: crate::APP_DEBUG.then_some("devel"),

            #[watch]
            set_visible: model.visible,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

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

                    add = &gtk::Box {
                        set_vexpand: true,
                        set_hexpand: true,

                        model.library_page.widget(),
                    } -> {
                        set_title: Some("Library"),
                        set_name: Some("library"),
                        set_icon_name: Some("applications-games-symbolic")
                    },

                    add = &gtk::Box {
                        set_vexpand: true,
                        set_hexpand: true,

                        model.profile_page.widget(),
                    } -> {
                        set_title: Some("Profile"),
                        set_name: Some("profile"),
                        set_icon_name: Some("person-symbolic")
                    },

                    connect_visible_child_notify => move |stack| {
                        if let Some(name) = stack.visible_child_name() {
                            sender.input(MainWindowMsg::SetShowSearch(
                                ["store", "library", "profile"].contains(&name.as_str())
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

    async fn init(_init: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let model = Self {
            store_page: StorePage::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    StorePageOutput::SetShowBack(s) => MainWindowMsg::SetShowBack(s)
                }),

            library_page: LibraryPage::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    LibraryPageOutput::SetShowBack(s) => MainWindowMsg::SetShowBack(s)
                }),

            profile_page: ProfilePageApp::builder()
                .launch(())
                .detach(),

            view_stack: adw::ViewStack::new(),

            registries: HashMap::new(),
            games: HashMap::new(),

            visible: false,

            show_search: true,
            searching: false,

            show_back: false,
        };

        let view_stack = &model.view_stack;

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, _sender: AsyncComponentSender<Self>) {
        match message {
            MainWindowMsg::AddGamesRegistry { url, manifest } => {
                self.registries.insert(url, Arc::new(manifest));
            }

            MainWindowMsg::AddGame { url, manifest } => {
                let manifest = Arc::new(manifest);

                self.games.insert(url.clone(), manifest.clone());

                self.store_page.emit(StorePageInput::AddGame {
                    url,
                    manifest: manifest.clone()
                });
            }

            MainWindowMsg::SetGeneration(generation) => {
                self.library_page.emit(LibraryPageInput::SetGeneration(generation));
            }

            MainWindowMsg::OpenWindow => self.visible = true,

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
                        "library" => self.library_page.emit(LibraryPageInput::ToggleDownloadsPage),

                        _ => ()
                    }
                }
            }

            MainWindowMsg::ActivateStorePage => {
                self.store_page.emit(StorePageInput::Activate);
            }

            MainWindowMsg::ActivateLibraryPage => {
                self.library_page.emit(LibraryPageInput::Activate);
            }
        }
    }
}
