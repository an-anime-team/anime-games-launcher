use gtk::prelude::*;

use relm4::prelude::*;
use relm4::factory::*;

use super::*;

#[derive(Debug, Clone)]
pub struct CardsListInit {
    pub image: CardImage,
    pub title: String
}

impl CardsListInit {
    #[inline]
    pub fn new(image: CardImage, title: impl ToString) -> Self {
        Self {
            image,
            title: title.to_string()
        }
    }
}

#[derive(Debug)]
pub enum CardsListInput {
    Clicked
}

#[derive(Debug)]
pub enum CardsListOutput {
    Selected(DynamicIndex)
}

#[derive(Debug)]
pub struct CardsList {
    pub card: AsyncController<CardComponent>,

    pub title: String,

    index: DynamicIndex
}

#[relm4::factory(pub, async)]
impl AsyncFactoryComponent for CardsList {
    type Init = CardsListInit;
    type Input = CardsListInput;
    type Output = CardsListOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        #[root]
        gtk::ListBoxRow {
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,

                set_spacing: 12,

                self.card.widget() -> &adw::Clamp {
                    set_margin_top: 6,
                    set_margin_bottom: 6
                },

                gtk::Label {
                    set_label: &self.title
                }
            },

            set_activatable: true,

            connect_activate => CardsListInput::Clicked
        }
    }

    async fn init_model(init: Self::Init, index: &DynamicIndex, _sender: AsyncFactorySender<Self>) -> Self {
        Self {
            card: CardComponent::builder()
                .launch(CardComponent {
                    image: Some(init.image),
                    title: None,

                    ..CardComponent::small()
                })
                .detach(),

            title: init.title,

            index: index.clone()
        }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncFactorySender<Self>) {
        match msg {
            CardsListInput::Clicked => {
                let _ = sender.output(CardsListOutput::Selected(self.index.clone()));
            }
        }
    }
}

