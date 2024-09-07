use std::fmt::Display;

use adw::prelude::*;
use gtk::prelude::*;

use relm4::factory::positions::GridPosition;
use relm4::factory::*;
use relm4::prelude::*;

use super::prelude::CardComponent;

#[derive(Debug, Clone)]
pub struct CardsGridFactoryInit {
    title: String,
    image: String,
}

impl CardsGridFactoryInit {
    #[inline]
    pub fn new(title: impl Display, image: impl Display) -> Self {
        Self {
            title: title.to_string(),
            image: image.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct CardsGridFactory {
    pub card: AsyncController<CardComponent>,
    pub title: String,
    pub index: DynamicIndex,
}

#[derive(Debug)]
pub enum CardsGridFactoryOutput {
    Click(DynamicIndex),
}

impl Position<GridPosition, DynamicIndex> for CardsGridFactory {
    fn position(&self, index: &DynamicIndex) -> GridPosition {
        let index = index.current_index();
        let x = index / 9;
        let y = index % 9;
        GridPosition {
            column: y as i32,
            row: x as i32,
            width: 1,
            height: 1,
        }
    }
}

impl FactoryComponent for CardsGridFactory {
    type ParentWidget = gtk::Grid;
    type CommandOutput = ();
    type Input = ();
    type Output = CardsGridFactoryOutput;
    type Init = CardsGridFactoryInit;
    type Root = gtk::Box;
    type Widgets = ();
    type Index = DynamicIndex;

    fn init_model(init: Self::Init, index: &Self::Index, sender: FactorySender<Self>) -> Self {
        Self {
            title: init.title,
            card: CardComponent::builder()
                .launch(CardComponent {
                    image: Some(init.image),
                    title: None,
                    ..CardComponent::medium()
                })
                .detach(),
            index: index.clone(),
        }
    }

    fn init_root(&self) -> Self::Root {
        relm4::view! {
            root = gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 16,
                set_margin_all: 32,
            }
        }
        root
    }

    fn init_widgets(
        &mut self,
        index: &Self::Index,
        root: Self::Root,
        returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        relm4::view! {
            #[local_ref]
            root -> gtk::Box {
                self.card.widget(),
                gtk::Button {
                    gtk::Label {
                        set_text: self.title.as_str(),
                    },
                    connect_clicked[sender, index] => move |_| {
                        sender.output(CardsGridFactoryOutput::Click(index.clone())).unwrap();
                    },
                },
            }
        }
    }
}

pub struct CardsGrid {
    items: FactoryVecDeque<CardsGridFactory>,
}

#[derive(Debug)]
pub enum CardsGridMsg {
    Click(DynamicIndex),
}

#[relm4::component(pub)]
impl SimpleComponent for CardsGrid {
    type Init = ();
    type Input = CardsGridMsg;
    type Output = ();

    view! {
        gtk::Box {
            #[local_ref]
            cards_grid -> gtk::Grid {
                set_orientation: gtk::Orientation::Horizontal,
                set_column_spacing: 32,
                set_row_spacing: 32,
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let cards =
            FactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |msg| match msg {
                    CardsGridFactoryOutput::Click(item) => CardsGridMsg::Click(item),
                });

        let model = CardsGrid { items: cards };
        let cards_grid = model.items.widget();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            CardsGridMsg::Click(item) => {
                println!("Clicked {}", item.current_index());
            }
        }
    }
}
