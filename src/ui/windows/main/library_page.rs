use adw::prelude::*;
use gtk::prelude::*;

use relm4::factory::*;
use relm4::prelude::*;

use crate::ui::components::downloads_row::DownloadsRow;
use crate::ui::components::downloads_row::DownloadsRowInit;
use crate::ui::components::prelude::*;

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
            #[transition(Crossfade)]
            append = if model.show_downloads {
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    gtk::Button {
                        set_icon_name: "go-previous-symbolic",
                        set_css_classes: &["flat"],
                        set_halign: gtk::Align::Start,
                        set_margin_all: 16,
                        connect_clicked => LibraryPageAppMsg::ToggleDownloadsPage,
                    },
                    model.downloads_page.widget(),
                }
            } else {
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
                        adw::PreferencesGroup {
                            model.active_download.widget() {
                                set_width_request: 1000,
                                set_activatable: true,
                                connect_activated => LibraryPageAppMsg::ToggleDownloadsPage,
                            }
                        }
                    }
                }
            }
        },
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let TEST_PATH = "card.jpg";

        let model = Self {
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
                    TEST_PATH,
                    String::from("Punishing: Gray Raven"),
                    String::from("69.42.0"),
                    String::from("Global"),
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

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>) {
        match msg {
            LibraryPageAppMsg::ShowGameDetails(index) => {
                if let Some(details) = self.cards_list.get(index.current_index()) {
                    todo!("{:?}", details);
                }
            }
            LibraryPageAppMsg::ToggleDownloadsPage => {
                self.show_downloads = !self.show_downloads;
            }
        }
    }
}
