use adw::prelude::*;
use gtk::prelude::*;

use relm4::factory::*;
use relm4::prelude::*;

use crate::ui::components::cards_row::*;

#[derive(Debug, Clone)]
pub enum StorePageAppMsg {
    Test,
}

#[derive(Debug)]
pub struct StorePageApp {
    cards: AsyncController<CardsRow>,
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for StorePageApp {
    type Init = ();
    type Input = StorePageAppMsg;
    type Output = ();

    view! {
        #[root]
        adw::PreferencesPage {
            set_title: "Store",
            add = &adw::PreferencesGroup {
                gtk::SearchEntry,
            },
            add = &adw::PreferencesGroup {
                set_title: "Featured",
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_align: gtk::Align::Center,
                    set_spacing: 16,
                    #[name = "carousel"]
                    adw::Carousel {
                        gtk::Picture {
                            set_filename: Some(&TEST_PATH),
                        },
                        gtk::Picture {
                            set_filename: Some(&TEST_PATH),
                        },
                        gtk::Picture {
                            set_filename: Some(&TEST_PATH),
                        }
                    },
                    adw::CarouselIndicatorDots {
                        set_carousel: Some(&carousel),
                    },
                }
            },
            model.cards.widget(),
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let TEST_PATH = String::from("background.jpg");

        let model = Self {
            cards: CardsRow::builder().launch(String::from("MiHoYo")).detach(),
        };
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>) {
        match msg {
            StorePageAppMsg::Test => {}
        }
    }
}
