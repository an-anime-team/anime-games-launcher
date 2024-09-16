use adw::prelude::*;
use gtk::prelude::*;

use relm4::{factory::*, prelude::*};

use super::card::*;

#[derive(Debug)]
pub struct CardsRow {
    cards: AsyncFactoryVecDeque<CardComponentFactory>,
    title: String,
}

#[derive(Debug)]
pub enum CardsRowMsg {
    Clicked(DynamicIndex),
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for CardsRow {
    type Init = String;
    type Input = CardsRowMsg;
    type Output = ();

    view! {
        #[root]
        adw::PreferencesGroup {
            set_title: &model.title,
            gtk::ScrolledWindow {
                set_policy: (gtk::PolicyType::Automatic, gtk::PolicyType::Never),
                model.cards.widget() {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 16,
                }
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let mut model = Self {
            cards: AsyncFactoryVecDeque::builder().launch_default().forward(
                sender.input_sender(),
                |msg| match msg {
                    CardComponentFactoryOutput::Clicked(index) => CardsRowMsg::Clicked(index),
                },
            ),
            title: init,
        };
        let widgets = view_output!();

        for _ in 0..10 {
            model.cards.guard().push_front(CardComponent {
                image: Some(String::from("card.jpg")),
                title: Some(String::from("Amogus")),
                clickable: true,
                ..CardComponent::medium()
            });
        }

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            CardsRowMsg::Clicked(index) => {
                println!("Clicked {:?}", index.current_index());
            }
        }
    }
}
