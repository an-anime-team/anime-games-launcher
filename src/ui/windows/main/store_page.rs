use adw::prelude::*;
use gtk::prelude::*;

use relm4::prelude::*;

use crate::ui::components::cards_row::*;

#[derive(Debug)]
pub enum StorePageAppMsg {
    Clicked(DynamicIndex),
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
        adw::NavigationPage {
            set_title: "Store",
            #[wrap(Some)]
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 16,
                set_spacing: 16,
                gtk::SearchEntry,
                adw::PreferencesGroup {
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
                        }
                    }
                },
                model.cards.widget(),
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let TEST_PATH = String::from("background.jpg");

        let model = Self {
            cards: CardsRow::builder().launch(String::from("MiHoYo")).forward(
                sender.input_sender(),
                |msg| match msg {
                    CardsRowMsg::Clicked(index) => StorePageAppMsg::Clicked(index),
                },
            ),
        };
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>) {
        match msg {
            StorePageAppMsg::Clicked(index) => {
                println!("Clicked {}", index.current_index());
            }
        }
    }
}
