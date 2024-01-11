use relm4::prelude::*;
use adw::prelude::*;

use crate::games::integrations::standards::addons::{
    Addon,
    AddonsGroup
};

use crate::ui::components::game_card::CardInfo;

use super::addon_group::AddonsGroupComponentInput;

pub struct AddonRowComponentInit {
    pub addons_group: AddonsGroup,
    pub addon_info: Addon,
    pub game_info: CardInfo,

    pub installed: bool,
    pub enabled: bool
}

#[derive(Debug)]
pub struct AddonRowComponent {
    pub switch: gtk::Switch,

    pub addons_group: AddonsGroup,
    pub addon_info: Addon,
    pub game_info: CardInfo,

    pub installed: bool,
    pub enabled: bool
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddonRowComponentMsg {
    PerformAction,
    ToggleAddon
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for AddonRowComponent {
    type Init = AddonRowComponentInit;
    type Input = AddonRowComponentMsg;
    type Output = AddonsGroupComponentInput;

    view! {
        #[root]
        adw::ActionRow {
            set_title: &model.addon_info.title,

            set_subtitle: if model.addon_info.required {
                "Required"
            } else {
                ""
            },

            add_suffix = &gtk::Button {
                set_valign: gtk::Align::Center,

                set_visible: !model.addon_info.required || !model.installed,

                set_css_classes: if model.installed {
                    &["flat", "error"]
                } else {
                    &["flat"]
                },

                adw::ButtonContent {
                    set_icon_name: if model.installed {
                        "user-trash-symbolic"
                    } else {
                        "folder-download-symbolic"
                    },

                    set_label: if model.installed {
                        "Uninstall"
                    } else {
                        "Install"
                    }
                },

                connect_clicked => AddonRowComponentMsg::PerformAction
            },

            #[local_ref]
            add_suffix = switch -> gtk::Switch {
                set_valign: gtk::Align::Center,

                set_visible: model.installed && !model.addon_info.required,

                #[watch]
                #[block_signal(toggle_handler)]
                set_active: model.enabled,

                connect_active_notify => AddonRowComponentMsg::ToggleAddon @toggle_handler
            }
        }
    }

    async fn init(init: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let model = Self {
            switch: gtk::Switch::new(),

            addons_group: init.addons_group,
            addon_info: init.addon_info,
            game_info: init.game_info,

            installed: init.installed,
            enabled: init.enabled
        };

        let switch = &model.switch;

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            AddonRowComponentMsg::PerformAction => {
                if self.installed {
                    self.switch.set_active(false);

                    sender.output(AddonsGroupComponentInput::UninstallAddon(self.addon_info.clone())).unwrap();
                }

                else {
                    self.switch.set_active(true);

                    sender.output(AddonsGroupComponentInput::InstallAddon(self.addon_info.clone())).unwrap();
                }
            }

            AddonRowComponentMsg::ToggleAddon => {
                self.enabled = !self.enabled;

                sender.output(AddonsGroupComponentInput::ToggleAddon {
                    addon: self.addon_info.clone(),
                    enabled: self.enabled
                }).unwrap();
            }
        }
    }
}
