use relm4::prelude::*;
use adw::prelude::*;

use crate::games::integrations::standards::addons::AddonsGroup;
use crate::ui::components::game_card::CardInfo;

use super::addon_row::AddonRowComponent;

#[derive(Debug)]
pub struct AddonsGroupComponent {
    pub addons_widgets: Vec<AsyncController<AddonRowComponent>>,

    pub addons_group: AddonsGroup,
    pub game_info: CardInfo
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddonsGroupComponentInput {

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddonsGroupComponentOutput {

}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for AddonsGroupComponent {
    type Init = (AddonsGroup, CardInfo);
    type Input = AddonsGroupComponentInput;
    type Output = AddonsGroupComponentOutput;

    view! {
        #[root]
        group = adw::PreferencesGroup {
            set_title: &model.addons_group.title
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self {
            addons_widgets: init.0.addons
                .iter()
                .cloned()
                .map(|addon| {
                    AddonRowComponent::builder()
                        .launch((addon, init.1.clone()))
                        .detach()
                })
                .collect(),

            addons_group: init.0,
            game_info: init.1
        };

        let widgets = view_output!();

        for widget in &model.addons_widgets {
            widgets.group.add(widget.widget());
        }

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        
    }
}
