use adw::prelude::*;
use gtk::prelude::*;

use relm4::{factory::*, prelude::*};

use super::card::*;

#[derive(Debug)]
pub struct CardsRowFactory {
    pub cards: AsyncFactoryVecDeque<CardComponentFactory>,
    pub title: String,
}

#[derive(Debug)]
pub enum CardsRowFactoryMsg {
    Add(CardComponent),
}

#[derive(Debug)]
pub enum CardsRowFactoryOutput {
    /// row, column
    Clicked(DynamicIndex, DynamicIndex),
}

#[relm4::factory(pub, async)]
impl AsyncFactoryComponent for CardsRowFactory {
    type Init = String;
    type Input = CardsRowFactoryMsg;
    type Output = CardsRowFactoryOutput;
    type ParentWidget = gtk::Box;
    type CommandOutput = ();

    view! {
        #[root]
        adw::PreferencesGroup {
            set_title: &self.title,
            gtk::ScrolledWindow {
                set_policy: (gtk::PolicyType::Automatic, gtk::PolicyType::Never),
                self.cards.widget() {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 16,
                }
            }
        }
    }

    async fn init_model(
        init: Self::Init,
        index: &DynamicIndex,
        sender: AsyncFactorySender<Self>,
    ) -> Self {
        let index = index.clone();
        Self {
            cards: AsyncFactoryVecDeque::builder().launch_default().forward(
                sender.output_sender(),
                move |msg| match msg {
                    CardComponentFactoryOutput::Clicked(ix) => {
                        CardsRowFactoryOutput::Clicked(index.clone(), ix)
                    }
                },
            ),
            title: init,
        }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncFactorySender<Self>) {
        match msg {
            CardsRowFactoryMsg::Add(card) => {
                self.cards.guard().push_back(card);
            }
        }
    }
}
