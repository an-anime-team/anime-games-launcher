use relm4::prelude::*;
use relm4::component::*;

use crate::ui::windows::main::MainAppMsg;

use crate::ui::components::game_card::{
    GameCardInfo,
    GameCardComponent,
    GameCardComponentInput,
    GameCardComponentOutput
};

#[derive(Debug)]
pub struct GameCardFactory {
    pub component: AsyncController<GameCardComponent>
}

#[relm4::factory(pub)]
impl FactoryComponent for GameCardFactory {
    type Init = GameCardInfo;
    type Input = GameCardComponentInput;
    type Output = GameCardComponentOutput;
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
            GameCardComponentOutput::CardClicked { info, installed }
                => Some(Self::ParentInput::OpenDetails { info, installed })
        }
    }

    #[inline]
    fn init_model(init: Self::Init, _index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        Self {
            component: GameCardComponent::builder()
                .launch(init)
                .forward(sender.output_sender(), std::convert::identity)
        }
    }

    #[inline]
    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        self.component.emit(msg);
    }
}
