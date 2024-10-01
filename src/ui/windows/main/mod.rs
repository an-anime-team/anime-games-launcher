use adw::prelude::*;
use gtk::prelude::*;
use library_page::LibraryPageAppOutput;
use relm4::prelude::*;

pub mod downloads_page;
pub mod game_page;
pub mod library_page;
pub mod profile_page;
pub mod store_page;

pub use downloads_page::{DownloadsPageApp, DownloadsPageAppMsg};
pub use game_page::{GamePageApp, GamePageAppMsg};
pub use library_page::{LibraryPageApp, LibraryPageAppMsg};
pub use profile_page::{ProfilePageApp, ProfilePageAppMsg};
pub use store_page::{StorePageApp, StorePageAppMsg, StorePageAppOutput};

pub static mut WINDOW: Option<adw::Window> = None;

#[derive(Debug, Clone)]
pub enum MainAppMsg {
    ToggleSearching,
    SetShowSearch(bool),
    SetShowBack(bool),
    GoBack,
    ActivateStorePage,
    ActivateLibraryPage,
}

#[derive(Debug)]
pub struct MainApp {
    store_page: AsyncController<StorePageApp>,
    library_page: AsyncController<LibraryPageApp>,
    profile_page: AsyncController<ProfilePageApp>,

    view_stack: adw::ViewStack,

    show_search: bool,
    searching: bool,

    show_back: bool,
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for MainApp {
    type Init = ();
    type Input = MainAppMsg;
    type Output = ();

    view! {
        #[root]
        window = adw::Window {
            set_size_request: (1200, 800),
            set_title: Some("Anime Games Launcher"),

            add_css_class?: crate::APP_DEBUG.then_some("devel"),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar {
                    add_css_class: "flat",

                    pack_start = &gtk::Button {
                        set_icon_name: "loupe-symbolic",
                        add_css_class: "flat",
                        #[watch]
                        set_visible: model.show_search && !model.show_back,
                        connect_clicked => MainAppMsg::ToggleSearching,
                    },

                    pack_start = &gtk::Button {
                        set_icon_name: "go-previous-symbolic",
                        add_css_class: "flat",
                        #[watch]
                        set_visible: model.show_back,
                        connect_clicked => MainAppMsg::GoBack,
                    },

                    #[wrap(Some)]
                    set_title_widget = &adw::ViewSwitcher {
                        set_policy: adw::ViewSwitcherPolicy::Wide,

                        set_stack: Some(&view_stack)
                    }
                },

                #[local_ref]
                view_stack -> adw::ViewStack {
                    connect_visible_child_notify => move |stack| {
                        if let Some(name) = stack.visible_child_name() {
                            // Show search on these page names
                            sender.input(
                                MainAppMsg::SetShowSearch(
                                    ["store", "library", "profile"].contains(&name.as_str())
                                )
                            );

                            // Update back button
                            match name.as_str() {
                                "store" => {
                                    sender.input(MainAppMsg::ActivateStorePage);
                                },
                                "library" => {
                                    sender.input(MainAppMsg::ActivateLibraryPage);
                                },
                                _ => {}
                            }
                        }
                    },

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
                    }
                }
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self {
            store_page: StorePageApp::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    StorePageAppOutput::SetShowBack(s) => MainAppMsg::SetShowBack(s),
                }),

            library_page: LibraryPageApp::builder().launch(()).forward(
                sender.input_sender(),
                |msg| match msg {
                    LibraryPageAppOutput::SetShowBack(s) => MainAppMsg::SetShowBack(s),
                },
            ),

            view_stack: adw::ViewStack::new(),

            profile_page: ProfilePageApp::builder().launch(()).detach(),

            show_search: true,
            searching: false,

            show_back: false,
        };

        let view_stack = &model.view_stack;

        let widgets = view_output!();

        unsafe {
            WINDOW = Some(widgets.window.clone());
        }

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            MainAppMsg::ToggleSearching => {
                self.store_page
                    .sender()
                    .emit(StorePageAppMsg::ToggleSearching);
                self.searching = !self.searching;
            }
            MainAppMsg::SetShowSearch(state) => {
                self.show_search = state;
            }
            MainAppMsg::SetShowBack(state) => {
                self.show_back = state;
            }
            MainAppMsg::GoBack => {
                self.show_back = false;

                // Navigate back only on the visible page
                if let Some(name) = self.view_stack.visible_child_name() {
                    match name.as_str() {
                        "store" => self.store_page.sender().emit(StorePageAppMsg::HideGamePage),
                        "library" => self
                            .library_page
                            .sender()
                            .emit(LibraryPageAppMsg::ToggleDownloadsPage),
                        _ => {}
                    }
                }
            }
            MainAppMsg::ActivateStorePage => {
                self.store_page.sender().emit(StorePageAppMsg::Activate);
            }
            MainAppMsg::ActivateLibraryPage => {
                self.library_page.sender().emit(LibraryPageAppMsg::Activate);
            }
        }
    }
}
