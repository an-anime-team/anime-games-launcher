use relm4::prelude::*;
use adw::prelude::*;

use crate::games;
use crate::games::integrations::standards::addons::Addon;

use crate::ui::components::game_card::CardInfo;

#[derive(Debug)]
pub struct AddonRowComponent {
    pub addon_info: Addon,
    pub game_info: CardInfo
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddonRowComponentInput {

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddonRowComponentOutput {

}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for AddonRowComponent {
    type Init = (Addon, CardInfo);
    type Input = AddonRowComponentInput;
    type Output = AddonRowComponentOutput;

    view! {
        #[root]
        adw::ActionRow {
            set_title: &model.addon_info.title,

            set_subtitle: if model.addon_info.required {
                "Required"
            } else {
                ""
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self {
            addon_info: init.0,
            game_info: init.1
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        
    }
}
