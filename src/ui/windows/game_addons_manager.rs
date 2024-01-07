use std::collections::HashSet;

use relm4::prelude::*;

use gtk::prelude::*;
use adw::prelude::*;

use crate::config;
use crate::config::games::GameEditionAddon;

use crate::games;
use crate::games::integrations::standards::addons::{
    Addon,
    AddonsGroup
};

use crate::ui::components::addon::addon_group::{
    AddonsGroupComponent,
    AddonsGroupComponentInit,
    AddonsGroupComponentOutput
};

use crate::ui::components::game_card::CardInfo;

use super::main::MainAppMsg;

static mut WINDOW: Option<adw::ApplicationWindow> = None;

#[derive(Debug)]
pub struct GameAddonsManagerApp {
    pub addons_groups_widgets: Vec<AsyncController<AddonsGroupComponent>>,
    pub addons_groups_page: adw::PreferencesPage,

    pub game_info: CardInfo,

    pub enabled_addons: HashSet<GameEditionAddon>
}

#[derive(Debug, Clone)]
pub enum GameAddonsManagerAppMsg {
    SetGameInfo {
        game_info: CardInfo,
        addons: Vec<AddonsGroup>
    },

    ToggleAddon {
        addon: Addon,
        group: AddonsGroup,
        enabled: bool
    }
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for GameAddonsManagerApp {
    type Init = adw::ApplicationWindow;
    type Input = GameAddonsManagerAppMsg;
    type Output = MainAppMsg;

    view! {
        window = adw::ApplicationWindow {
            set_default_size: (700, 560),
            set_title: Some("Game addons"),

            set_hide_on_close: true,
            set_modal: true,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar {
                    add_css_class: "flat"
                },

                #[local_ref]
                addons_groups_page -> adw::PreferencesPage {
                    // add = &adw::PreferencesGroup {
                    //     set_title: "Mockup",

                    //     adw::ActionRow {
                    //         set_title: "Not installed addon",

                    //         add_suffix = &gtk::Button {
                    //             set_valign: gtk::Align::Center,

                    //             add_css_class: "flat",

                    //             adw::ButtonContent {
                    //                 set_icon_name: "folder-download-symbolic",
                    //                 set_label: "Install"
                    //             },
                    //         }
                    //     },

                    //     adw::ActionRow {
                    //         set_title: "Installed addon",

                    //         add_suffix = &gtk::Button {
                    //             set_valign: gtk::Align::Center,

                    //             add_css_class: "flat",

                    //             adw::ButtonContent {
                    //                 set_icon_name: "user-trash-symbolic",
                    //                 set_label: "Uninstall"
                    //             },
                    //         },

                    //         add_suffix = &gtk::Switch {
                    //             set_valign: gtk::Align::Center
                    //         }
                    //     },

                    //     adw::ExpanderRow {
                    //         set_title: "Addon with options",

                    //         add_suffix = &gtk::Switch {
                    //             set_valign: gtk::Align::Center
                    //         },

                    //         add_suffix = &gtk::Button {
                    //             set_valign: gtk::Align::Center,

                    //             add_css_class: "flat",

                    //             adw::ButtonContent {
                    //                 set_icon_name: "user-trash-symbolic",
                    //                 set_label: "Uninstall"
                    //             },
                    //         },

                    //         add_row = &adw::ActionRow {
                    //             set_title: "Option 1"
                    //         },

                    //         add_row = &adw::ActionRow {
                    //             set_title: "Option 2"
                    //         },

                    //         add_row = &adw::ActionRow {
                    //             set_title: "Option 3"
                    //         }
                    //     },

                    //     adw::ExpanderRow {
                    //         set_title: "Addon with an author",
                    //         set_subtitle: "Author: amogus",

                    //         add_suffix = &gtk::Switch {
                    //             set_valign: gtk::Align::Center
                    //         },

                    //         add_suffix = &gtk::Button {
                    //             set_valign: gtk::Align::Center,

                    //             add_css_class: "flat",

                    //             adw::ButtonContent {
                    //                 set_icon_name: "user-trash-symbolic",
                    //                 set_label: "Uninstall"
                    //             },
                    //         },

                    //         add_row = &adw::ActionRow {
                    //             set_title: "Option 1"
                    //         },

                    //         add_row = &adw::ActionRow {
                    //             set_title: "Option 2"
                    //         },

                    //         add_row = &adw::ActionRow {
                    //             set_title: "Option 3"
                    //         }
                    //     }
                    // }
                }
            }
        }
    }

    async fn init(parent: Self::Init, root: Self::Root, _sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let model = Self {
            addons_groups_widgets: Vec::new(),
            addons_groups_page: adw::PreferencesPage::new(),

            game_info: CardInfo::default(),

            enabled_addons: HashSet::default()
        };

        let addons_groups_page = &model.addons_groups_page;

        let widgets = view_output!();

        widgets.window.set_transient_for(Some(&parent));

        unsafe {
            WINDOW = Some(widgets.window.clone());
        }

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            Self::Input::SetGameInfo { game_info, addons } => {
                let game = unsafe {
                    games::get_unsafe(game_info.get_name())
                };

                let settings = config::get()
                    .games.get_game_settings(game_info.get_name())
                    .unwrap();

                let paths = settings.paths.get(game_info.get_edition()).unwrap();

                self.enabled_addons = settings.addons
                    .get(game_info.get_edition())
                    .map(|addons| HashSet::from_iter(addons.clone()))
                    .unwrap_or_default();

                self.game_info = game_info.clone();

                for group in &self.addons_groups_widgets {
                    self.addons_groups_page.remove(group.widget());
                }

                self.addons_groups_widgets.clear();

                let mut installed_addons = HashSet::new();

                for group in addons {
                    for addon in &group.addons {
                        let addon_path = paths.addons
                            .join(&group.name)
                            .join(&addon.name);

                        // FIXME: handle errors
                        if let Ok(true) = game.is_addon_installed(&group.name, &addon.name, addon_path.to_string_lossy(), game_info.get_edition()) {
                            installed_addons.insert(GameEditionAddon {
                                group: group.name.clone(),
                                name: addon.name.clone()
                            });
                        }
                    }

                    let group = AddonsGroupComponent::builder()
                        .launch(AddonsGroupComponentInit {
                            addons_group: group,
                            game_info: game_info.clone(),
                            enabled_addons: self.enabled_addons.clone(),
                            installed_addons: installed_addons.clone()
                        })
                        .forward(sender.input_sender(), |msg| {
                            match msg {
                                AddonsGroupComponentOutput::ToggleAddon { addon, group, enabled }
                                    => Self::Input::ToggleAddon { addon, group, enabled }
                            }
                        });

                    self.addons_groups_page.add(group.widget());
                    self.addons_groups_widgets.push(group);
                }
            }

            GameAddonsManagerAppMsg::ToggleAddon { addon, group, enabled } => {
                let addon = GameEditionAddon {
                    group: group.name,
                    name: addon.name
                };

                if enabled {
                    self.enabled_addons.insert(addon);
                }

                else {
                    self.enabled_addons.remove(&addon);
                }

                // FIXME move it to the window closing event
                sender.output(MainAppMsg::SetEnabledAddons {
                    game: self.game_info.clone(),
                    addons: self.enabled_addons.clone()
                }).unwrap();
            }
        }
    }
}
