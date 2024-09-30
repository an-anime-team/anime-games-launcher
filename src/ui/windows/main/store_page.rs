use adw::prelude::*;
use gtk::prelude::*;

use relm4::{factory::AsyncFactoryVecDeque, prelude::*};

use crate::ui::components::{card::*, cards_row::*};

#[derive(Debug)]
pub enum StorePageAppMsg {
    Clicked(DynamicIndex, DynamicIndex),
}

#[derive(Debug)]
pub struct StorePageApp {
    rows: AsyncFactoryVecDeque<CardsRowFactory>,
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
                gtk::ScrolledWindow {
                    set_propagate_natural_width: true,
                    model.rows.widget() {
                        set_orientation: gtk::Orientation::Vertical,
                        set_vexpand: true,
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
        let TEST_PATH = String::from("background.jpg");

        let mut model = Self {
            rows: AsyncFactoryVecDeque::builder().launch_default().forward(
                sender.input_sender(),
                |msg| match msg {
                    CardsRowFactoryOutput::Clicked(row, column) => {
                        StorePageAppMsg::Clicked(row, column)
                    }
                },
            ),
        };
        let widgets = view_output!();

        for name in 'a'..'z' {
            let index = model.rows.guard().push_back(String::from(name));
            for i in 0..10 {
                model.rows.send(
                    index.current_index(),
                    CardsRowFactoryMsg::Add(CardComponent {
                        image: Some(String::from("card.jpg")),
                        clickable: true,
                        title: Some(format!("Card {}", i)),
                        ..CardComponent::medium()
                    }),
                )
            }
        }

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>) {
        match msg {
            StorePageAppMsg::Clicked(r, c) => {
                println!(
                    "Clicked element {} of row {}",
                    c.current_index(),
                    r.current_index()
                );
            }
        }
    }
}
