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
    type Init = Self;
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

                add_css_class: "flat",

                adw::ButtonContent {
                    set_icon_name: "folder-download-symbolic",

                    set_label: if model.installed {
                        "Uninstall"
                    } else {
                        "Install"
                    }
                },

                connect_clicked => AddonRowComponentMsg::PerformAction
            },

            add_suffix = &gtk::Switch {
                set_valign: gtk::Align::Center,

                set_sensitive: !model.addon_info.required,
                set_visible: model.installed,

                #[watch]
                set_active: model.enabled,

                connect_active_notify => AddonRowComponentMsg::ToggleAddon
            }
        }
    }

    async fn init(model: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            AddonRowComponentMsg::PerformAction => {
                let message = if self.installed {
                    AddonsGroupComponentInput::UninstallAddon(self.addon_info.clone())
                } else {
                    AddonsGroupComponentInput::InstallAddon(self.addon_info.clone())
                };

                sender.output(message).unwrap();
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
