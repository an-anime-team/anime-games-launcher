use std::collections::HashSet;

use relm4::prelude::*;
use adw::prelude::*;

use crate::config::games::settings::edition_addons::GameEditionAddon;

use crate::games::integrations::standards::addons::{
    AddonsGroup,
    Addon
};

use crate::ui::components::game_card::CardInfo;

use super::addon_row::AddonRowComponent;

pub struct AddonsGroupComponentInit {
    pub addons_group: AddonsGroup,
    pub game_info: CardInfo,
    pub enabled_addons: HashSet<GameEditionAddon>,
    pub installed_addons: HashSet<GameEditionAddon>
}

#[derive(Debug)]
pub struct AddonsGroupComponent {
    pub addons_widgets: Vec<AsyncController<AddonRowComponent>>,

    pub addons_group: AddonsGroup,
    pub game_info: CardInfo
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddonsGroupComponentInput {
    InstallAddon(Addon),
    UninstallAddon(Addon),

    ToggleAddon {
        addon: Addon,
        enabled: bool
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddonsGroupComponentOutput {
    InstallAddon {
        addon: Addon,
        group: AddonsGroup
    },

    UninstallAddon {
        addon: Addon,
        group: AddonsGroup
    },

    ToggleAddon {
        addon: GameEditionAddon,
        enabled: bool
    }
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for AddonsGroupComponent {
    type Init = AddonsGroupComponentInit;
    type Input = AddonsGroupComponentInput;
    type Output = AddonsGroupComponentOutput;

    view! {
        #[root]
        group = adw::PreferencesGroup {
            set_title: &model.addons_group.title
        }
    }

    async fn init(init: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let mut addons = init.addons_group.addons
            .clone()
            .into_iter()
            .map(|addon| {
                let enabled = init.enabled_addons.iter().any(|enabled_addon| {
                    enabled_addon.group == init.addons_group.name && enabled_addon.name == addon.name
                });

                let installed = init.installed_addons.iter().any(|installed_addon| {
                    installed_addon.group == init.addons_group.name && installed_addon.name == addon.name
                });

                (addon, enabled, installed)
            })
            .collect::<Vec<_>>();

        addons.sort_by(|a, b| {
            if a.2 == b.2 {
                b.1.cmp(&a.1)
            } else if a.1 == b.1 {
                b.0.title.cmp(&a.0.title)
            } else {
                b.2.cmp(&a.2)
            }
        });

        let model = Self {
            addons_widgets: addons
                .into_iter()
                .map(|(addon, enabled, installed)| {
                    AddonRowComponent::builder()
                        .launch(AddonRowComponent {
                            addons_group: init.addons_group.clone(),
                            game_info: init.game_info.clone(),

                            addon_info: addon,
                            enabled,
                            installed
                        })
                        .forward(sender.input_sender(), std::convert::identity)
                })
                .collect(),

            addons_group: init.addons_group,
            game_info: init.game_info
        };

        let widgets = view_output!();

        for widget in &model.addons_widgets {
            widgets.group.add(widget.widget());
        }

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            AddonsGroupComponentInput::InstallAddon(addon) => {
                sender.output(AddonsGroupComponentOutput::InstallAddon {
                    addon,
                    group: self.addons_group.clone()
                }).unwrap();
            }

            AddonsGroupComponentInput::UninstallAddon(addon) => {
                sender.output(AddonsGroupComponentOutput::UninstallAddon {
                    addon,
                    group: self.addons_group.clone()
                }).unwrap();
            }

            AddonsGroupComponentInput::ToggleAddon { addon, enabled } => {
                sender.output(AddonsGroupComponentOutput::ToggleAddon {
                    addon: GameEditionAddon {
                        group: self.addons_group.name.clone(),
                        name: addon.name
                    },
                    enabled
                }).unwrap();
            }
        }
    }
}
