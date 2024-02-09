use gtk::prelude::*;
use adw::prelude::*;

use relm4::prelude::*;
use relm4::factory::*;

use super::prelude::CardComponent;

#[derive(Debug, Clone)]
pub struct CardsListFactoryInit {
    pub title: String,
    pub image: String
}

impl CardsListFactoryInit {
    #[inline]
    pub fn new(title: impl ToString, image: impl ToString) -> Self {
        Self {
            title: title.to_string(),
            image: image.to_string()
        }
    }
}

#[derive(Debug)]
pub enum CardsListFactoryInput {
    Clicked
}

#[derive(Debug)]
pub enum CardsListFactoryOutput {
    Selected(DynamicIndex)
}

#[derive(Debug)]
pub struct CardsListFactory {
    pub card: AsyncController<CardComponent>,

    pub title: String,

    index: DynamicIndex
}

#[relm4::factory(pub, async)]
impl AsyncFactoryComponent for CardsListFactory {
    type Init = CardsListFactoryInit;
    type Input = CardsListFactoryInput;
    type Output = CardsListFactoryOutput;
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

            connect_activate => CardsListFactoryInput::Clicked
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
            CardsListFactoryInput::Clicked => {
                sender.output(CardsListFactoryOutput::Selected(self.index.clone())).unwrap();
            }
        }
    }
}

