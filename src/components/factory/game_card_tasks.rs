use relm4::prelude::*;
use relm4::component::*;

use crate::games::GameVariant;
use crate::components::tasks_queue::TasksQueueComponentInput;

use crate::components::game_card::{
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
    type Init = GameVariant;
    type Input = GameCardComponentInput;
    type Output = GameCardComponentOutput;
    type CommandOutput = ();
    type ParentInput = TasksQueueComponentInput;
    type ParentWidget = gtk::FlowBox;

    view! {
        root = gtk::Box {
            self.component.widget(),
        }
    }

    fn forward_to_parent(output: Self::Output) -> Option<TasksQueueComponentInput> {
        None
    }

    #[inline]
    fn init_model(init: Self::Init, _index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        let component = GameCardComponent::builder()
            .launch(init)
            .forward(sender.output_sender(), std::convert::identity);

        component.emit(GameCardComponentInput::SetWidth(60));
        component.emit(GameCardComponentInput::SetHeight(84));
        component.emit(GameCardComponentInput::SetClickable(false));
        component.emit(GameCardComponentInput::SetDisplayTitle(false));

        Self { component }
    }

    #[inline]
    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        self.component.emit(msg);
    }
}
