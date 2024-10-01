use adw::prelude::*;
use gtk::prelude::*;

use relm4::{factory::AsyncFactoryVecDeque, prelude::*};

use crate::ui::components::{card::*, cards_row::*};

use super::game_page::*;

#[derive(Debug)]
pub enum StorePageAppMsg {
    ToggleSearching,
    HideGamePage,
    Clicked(DynamicIndex, DynamicIndex),
}

#[derive(Debug)]
pub enum StorePageAppOutput {
    ShowBack,
}

#[derive(Debug)]
pub struct StorePageApp {
    rows: AsyncFactoryVecDeque<CardsRowFactory>,
    game_page: AsyncController<GamePageApp>,
    searching: bool,
    show_game_page: bool,
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for StorePageApp {
    type Init = ();
    type Input = StorePageAppMsg;
    type Output = StorePageAppOutput;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            #[transition(SlideLeftRight)]
            append = if !model.show_game_page {
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_margin_all: 16,
                    set_spacing: 16,
                    gtk::SearchEntry {
                        #[watch]
                        set_visible: model.searching,
                    },
                    gtk::ScrolledWindow {
                        model.rows.widget() {
                            set_orientation: gtk::Orientation::Vertical,
                            set_vexpand: true,
                        }
                    }
                }
            } else {
                gtk::Box {
                    model.game_page.widget() {}
                }
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let mut model = Self {
            rows: AsyncFactoryVecDeque::builder().launch_default().forward(
                sender.input_sender(),
                |msg| match msg {
                    CardsRowFactoryOutput::Clicked(row, column) => {
                        StorePageAppMsg::Clicked(row, column)
                    }
                },
            ),
            game_page: GamePageApp::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    GamePageAppOutput::Hide => StorePageAppMsg::HideGamePage,
                }),
            searching: false,
            show_game_page: false,
        };
        let widgets = view_output!();

        for name in 'a'..'z' {
            let index = model.rows.guard().push_back(String::from(name));
            for i in 0..3 {
                model.rows.send(
                    index.current_index(),
                    CardsRowFactoryMsg::Add(CardComponent {
                        image: Some(String::from("card.jpg")),
                        clickable: true,
                        title: Some(format!(
                            "Card {}",
                            if name as u32 % 2 == 1 { i + 1 } else { i }
                        )),
                        ..CardComponent::medium()
                    }),
                )
            }
        }

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            StorePageAppMsg::ToggleSearching => {
                self.searching = !self.searching;
            }
            StorePageAppMsg::HideGamePage => {
                self.show_game_page = false;
            }
            StorePageAppMsg::Clicked(r, c) => {
                println!(
                    "Clicked element {} of row {}",
                    c.current_index(),
                    r.current_index()
                );
                self.show_game_page = true;
                sender.output(StorePageAppOutput::ShowBack).unwrap();
            }
        }
    }
}
