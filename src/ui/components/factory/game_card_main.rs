use relm4::prelude::*;
use relm4::component::*;

use crate::ui::windows::main::MainAppMsg;

use crate::ui::components::game_card::{
    CardInfo,
    CardComponent,
    CardComponentInput,
    CardComponentOutput
};

#[derive(Debug)]
pub struct CardFactory {
    pub component: AsyncController<CardComponent>
}

#[relm4::factory(pub)]
impl FactoryComponent for CardFactory {
    type Init = CardInfo;
    type Input = CardComponentInput;
    type Output = CardComponentOutput;
    type CommandOutput = ();
    type ParentInput = MainAppMsg;
    type ParentWidget = gtk::FlowBox;

    view! {
        root = gtk::Box {
            self.component.widget(),
        }
    }

    fn forward_to_parent(output: Self::Output) -> Option<Self::ParentInput> {
        match output {
            CardComponentOutput::CardClicked { info, installed }
                => Some(Self::ParentInput::OpenDetails { info, installed })
        }
    }

    #[inline]
    fn init_model(init: Self::Init, _index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        Self {
            component: CardComponent::builder()
                .launch(init)
                .forward(sender.output_sender(), std::convert::identity)
        }
    }

    #[inline]
    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        self.component.emit(msg);
    }
}
