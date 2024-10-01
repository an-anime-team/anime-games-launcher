use adw::prelude::*;
use gtk::prelude::*;
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
use store_page::StorePageAppOutput;
pub use store_page::{StorePageApp, StorePageAppMsg};

pub static mut WINDOW: Option<adw::Window> = None;

#[derive(Debug, Clone)]
pub enum MainAppMsg {
    ToggleSearching,
    ShowSearch,
    HideSearch,
    ShowBack,
    GoBack,
}

#[derive(Debug)]
pub struct MainApp {
    store_page: AsyncController<StorePageApp>,
    library_page: AsyncController<LibraryPageApp>,
    profile_page: AsyncController<ProfilePageApp>,

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

                #[name = "view_stack"]
                adw::ViewStack {
                    connect_visible_child_notify => move |stack| {
                        if let Some(name) = stack.visible_child_name() {
                            // Show search on these page name
                            if ["store", "library", "profile"].contains(&name.as_str()) {
                                sender.input(MainAppMsg::ShowSearch);
                            } else {
                                sender.input(MainAppMsg::HideSearch);
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
                    StorePageAppOutput::ShowBack => MainAppMsg::ShowBack,
                }),

            library_page: LibraryPageApp::builder().launch(()).detach(),

            profile_page: ProfilePageApp::builder().launch(()).detach(),

            show_search: true,
            searching: false,

            show_back: false,
        };

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
            MainAppMsg::ShowSearch => {
                self.show_search = true;
            }
            MainAppMsg::HideSearch => {
                self.show_search = false;
            }
            MainAppMsg::ShowBack => {
                self.show_back = true;
            }
            MainAppMsg::GoBack => {
                self.show_back = false;
                self.store_page.sender().emit(StorePageAppMsg::HideGamePage);
            }
        }
    }
}
