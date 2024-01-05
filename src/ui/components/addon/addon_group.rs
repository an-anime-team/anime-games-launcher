use relm4::prelude::*;
use adw::prelude::*;

use crate::games::integrations::standards::addons::AddonsGroup;

use super::addon_row::AddonRowComponent;

#[derive(Debug)]
pub struct AddonsGroupComponent {
    pub addons_widgets: Vec<AsyncController<AddonRowComponent>>,

    pub info: AddonsGroup
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddonsGroupComponentInput {

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddonsGroupComponentOutput {

}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for AddonsGroupComponent {
    type Init = AddonsGroup;
    type Input = AddonsGroupComponentInput;
    type Output = AddonsGroupComponentOutput;

    view! {
        #[root]
        group = adw::PreferencesGroup {
            set_title: &model.info.title
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self {
            addons_widgets: init.addons
                .iter()
                .cloned()
                .map(|dlc| {
                    AddonRowComponent::builder()
                        .launch(dlc)
                        .detach()
                })
                .collect(),

            info: init
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
