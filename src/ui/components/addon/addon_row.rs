use relm4::prelude::*;
use adw::prelude::*;

use crate::games::integrations::standards::addons::{
    Addon,
    AddonsGroup
};

use crate::ui::components::game_card::CardInfo;

use super::addon_group::AddonsGroupComponentInput;

#[derive(Debug)]
pub struct AddonRowComponent {
    pub addons_group: AddonsGroup,
    pub addon_info: Addon,
    pub game_info: CardInfo,

    pub enabled: bool
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddonRowComponentMsg {
    ToggleAddon
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for AddonRowComponent {
    type Init = Self;
    type Input = AddonRowComponentMsg;
    type Output = AddonsGroupComponentInput;

    view! {
        #[root]
        adw::SwitchRow {
            set_title: &model.addon_info.title,

            set_subtitle: if model.addon_info.required {
                "Required"
            } else {
                ""
            },

            set_activatable: !model.addon_info.required,

            #[watch]
            set_active: model.enabled,

            connect_active_notify => AddonRowComponentMsg::ToggleAddon
        }
    }

    async fn init(model: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            Self::Input::ToggleAddon => {
                self.enabled = !self.enabled;

                sender.output(Self::Output::ToggleAddon {
                    addon: self.addon_info.clone(),
                    enabled: self.enabled
                }).unwrap();
            }
        }
    }
}
