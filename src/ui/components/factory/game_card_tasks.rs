use relm4::prelude::*;

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
    type ParentWidget = gtk::FlowBox;

    view! {
        root = gtk::Box {
            self.component.widget(),
        }
    }

    #[inline]
    fn init_model(init: Self::Init, _index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        let component = CardComponent::builder()
            .launch(init)
            .forward(sender.output_sender(), std::convert::identity);

        component.emit(CardComponentInput::SetWidth(60));
        component.emit(CardComponentInput::SetHeight(84));
        component.emit(CardComponentInput::SetClickable(false));
        component.emit(CardComponentInput::SetDisplayTitle(false));

        Self { component }
    }

    #[inline]
    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        self.component.emit(msg);
    }
}
