use adw::prelude::*;
use gtk::prelude::*;

use relm4::factory::*;
use relm4::prelude::*;

use crate::ui::components::downloads_row::DownloadsRow;
use crate::ui::components::downloads_row::DownloadsRowInit;
use crate::ui::components::{cards_list, game_details::GameDetailsInit, prelude::*};

use super::DownloadsPageApp;

#[derive(Debug, Clone)]
pub enum LibraryPageAppMsg {
    ShowGameDetails(DynamicIndex),
    ToggleDownloadsPage,
}

#[derive(Debug)]
pub struct LibraryPageApp {
    cards_list: AsyncFactoryVecDeque<CardsListFactory>,
    game_details: AsyncController<GameDetails>,
    active_download: AsyncController<DownloadsRow>,
    downloads_page: AsyncController<DownloadsPageApp>,
    show_downloads: bool,
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for LibraryPageApp {
    type Init = ();
    type Input = LibraryPageAppMsg;
    type Output = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            adw::NavigationSplitView {
                #[watch]
                set_visible: !model.show_downloads,
                set_vexpand: true,
                set_hexpand: true,

                #[wrap(Some)]
                set_sidebar = &adw::NavigationPage {
                    // Supress Adwaita-WARNING **: AdwNavigationPage is missing a title
                    set_title: "Games",

                    #[wrap(Some)]
                    set_child = model.cards_list.widget(),
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
                #[watch]
                set_visible: !model.show_downloads,
                adw::PreferencesGroup {
                    model.active_download.widget() {
                        set_width_request: 1000,
                        set_activatable: true,
                        connect_activated => LibraryPageAppMsg::ToggleDownloadsPage,
                    }
                }
            },
            gtk::Box {
                #[watch]
                set_visible: model.show_downloads,
                set_orientation: gtk::Orientation::Vertical,
                gtk::Button {
                    set_icon_name: "draw-arrow-back-symbolic",
                    set_halign: gtk::Align::Start,
                    set_margin_all: 16,
                    connect_clicked => LibraryPageAppMsg::ToggleDownloadsPage,
                },
                model.downloads_page.widget(),
            }
        },
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let mut model = Self {
            cards_list: AsyncFactoryVecDeque::builder().launch_default().forward(
                sender.input_sender(),
                |msg| match msg {
                    CardsListFactoryOutput::Selected(index) => {
                        LibraryPageAppMsg::ShowGameDetails(index)
                    }
                },
            ),
            game_details: GameDetails::builder().launch(()).detach(),
            active_download: DownloadsRow::builder()
                .launch(DownloadsRowInit::new(
                    "/home/dylan/Repos/anime-games-launcher/assets/images/games/pgr/card.jpg",
                    "Punishing: Gray Raven",
                    "69.42.0",
                    "Global",
                    696969696969,
                    true,
                ))
                .detach(),
            downloads_page: DownloadsPageApp::builder().launch(()).detach(),
            show_downloads: false,
        };

        model
            .cards_list
            .widget()
            .add_css_class("navigation-sidebar");

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
            LibraryPageAppMsg::ShowGameDetails(index) => {
                if let Some(details) = self.cards_list.get(index.current_index()) {
                    self.game_details
                        .emit(GameDetailsInput::Update(GameDetailsInit {
                            title: details.title.clone(),
                            card_image: String::from("/path/to/card.jpg"),
                            background_image: String::from("/path/to/background.jpg"),
                        }));
                }
            }
            LibraryPageAppMsg::ToggleDownloadsPage => {
                self.show_downloads = !self.show_downloads;
            }
        }
    }
}
