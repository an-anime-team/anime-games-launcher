use relm4::prelude::*;

use crate::prelude::*;

#[derive(Debug)]
pub enum CardsGridOutput {
    Clicked(DynamicIndex)
}

#[derive(Debug)]
pub struct CardsGrid {
    pub card: AsyncController<CardComponent>
}

#[relm4::factory(pub, async)]
impl AsyncFactoryComponent for CardsGrid {
    type Init = CardComponent;
    type Input = ();
    type Output = CardsGridOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::FlowBox;

    view! {
        #[root]
        gtk::Box {
            self.card.widget(),
        }
    }

    async fn init_model(init: Self::Init, index: &DynamicIndex, sender: AsyncFactorySender<Self>) -> Self {
        let index = index.to_owned();

        Self {
            card: CardComponent::builder()
                .launch(init)
                .forward(sender.output_sender(), move |msg| {
                    match msg {
                        CardComponentOutput::Clicked => CardsGridOutput::Clicked(index.clone())
                    }
                })
        }
    }
}
