use relm4::prelude::*;
use adw::prelude::*;

use crate::games;
use crate::config;

use crate::games::integrations::standards::addons::{
    Addon,
    AddonsGroup
};

use crate::ui::components::game_card::CardInfo;

#[derive(Debug)]
pub struct AddonRowComponent {
    pub addons_group: AddonsGroup,
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
    type Init = (AddonsGroup, Addon, CardInfo);
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

    async fn init(init: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let model = Self {
            addons_group: init.0,
            addon_info: init.1,
            game_info: init.2
        };

        // TODO: handle errors

        // // Given game must be already listed by the singleton
        // let game = games::get(model.game_info.get_name()).unwrap().unwrap();

        // // Get game settings
        // let settings = config::get().games.get_game_settings(model.game_info.get_name()).unwrap();

        // // Get game paths
        // let paths = settings.paths.get(model.game_info.get_edition()).unwrap();

        // // Get addons folder driver
        // let driver = paths.addons.to_dyn_trait();

        // // Deploy addons folder and get current addon folder path
        // let addon_path = driver.deploy().unwrap()
        //     .join(&model.addons_group.name)
        //     .join(&model.addon_info.name);

        // // Get addon diff
        // let diff = game.get_addon_diff(
        //     &model.addons_group.name,
        //     &model.addon_info.name,
        //     addon_path.to_string_lossy(),
        //     model.game_info.get_edition()
        // ).unwrap();

        // dbg!(diff);

        // // Dismantle addons folder
        // driver.dismantle().unwrap();

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        
    }
}
