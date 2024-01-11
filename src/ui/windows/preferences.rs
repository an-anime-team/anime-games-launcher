use relm4::prelude::*;
use gtk::prelude::*;
use adw::prelude::*;

use crate::config;

use crate::config::games::wine::prelude::*;
use crate::config::games::enhancements::prelude::*;

pub static mut WINDOW: Option<adw::PreferencesWindow> = None;

pub struct PreferencesApp {

}

#[derive(Debug, Clone)]
pub enum PreferencesAppMsg {
    ShowToast {
        title: String,
        message: Option<String>
    }
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for PreferencesApp {
    type Init = adw::ApplicationWindow;
    type Input = PreferencesAppMsg;
    type Output = ();

    view! {
        window = adw::PreferencesWindow {
            set_default_size: (700, 560),
            set_title: Some("Preferences"),

            set_modal: true,
            set_hide_on_close: true,
            set_search_enabled: true,

            add = &adw::PreferencesPage {
                add = &adw::PreferencesGroup {
                    set_title: "General",

                    adw::ComboRow {
                        set_title: "Launcher language",
                        set_subtitle: "Changes after restart",

                        set_model: Some(&gtk::StringList::new(&[
                            "English"
                        ]))
                    },

                    adw::SwitchRow {
                        set_title: "Verify games",
                        set_subtitle: "Verify games installations after installation or updating",

                        set_active: config::get().general.verify_games,

                        connect_active_notify[sender] => move |switch| {
                            if let Err(err) = config::set("general.verify_games", switch.is_active()) {
                                sender.input(PreferencesAppMsg::ShowToast {
                                    title: String::from("Failed to update property"),
                                    message: Some(err.to_string())
                                })
                            }
                        }
                    },

                    // adw::ActionRow {
                    //     set_title: "Update games",
                    //     set_subtitle: "Download updates for installed games when they become available",

                    //     add_suffix = &gtk::Switch {
                    //         set_valign: gtk::Align::Center
                    //     }
                    // },

                    // adw::ActionRow {
                    //     set_title: "Pre-download updates",
                    //     set_subtitle: "Pre-download updates for installed games when they become available",

                    //     add_suffix = &gtk::Switch {
                    //         set_valign: gtk::Align::Center
                    //     }
                    // }
                },

                add = &adw::PreferencesGroup {
                    set_title: "Wine",

                    adw::ComboRow {
                        set_title: "Language",
                        set_subtitle: "Language used in the wine environment. Can fix keyboard layout issues",

                        set_model: Some(&gtk::StringList::new({
                            WineLang::list()
                                .into_iter()
                                .map(|lang| lang.name())
                                .collect::<Vec<_>>()
                                .as_slice()
                        })),

                        set_selected: WineLang::list().iter()
                            .position(|lang| lang == &config::get().games.wine.language)
                            .unwrap_or(0) as u32,

                        connect_selected_notify[sender] => move |row| {
                            let value = serde_json::to_value(WineLang::list()[row.selected() as usize]).unwrap();

                            if let Err(err) = config::set("games.wine.language", value) {
                                sender.input(PreferencesAppMsg::ShowToast {
                                    title: String::from("Failed to update property"),
                                    message: Some(err.to_string())
                                })
                            }
                        }
                    },

                    adw::ComboRow {
                        set_title: "Synchronization",
                        set_subtitle: "Technology used to synchronize inner wine events",

                        set_model: Some(&gtk::StringList::new(&[
                            "None",
                            "ESync",
                            "FSync"
                        ])),

                        set_selected: match config::get().games.wine.sync {
                            WineSync::None  => 0,
                            WineSync::ESync => 1,
                            WineSync::FSync => 2
                        },

                        connect_selected_notify[sender] => move |row| {
                            let sync = [
                                WineSync::None,
                                WineSync::ESync,
                                WineSync::FSync 
                            ][row.selected() as usize];

                            let value = serde_json::to_value(sync).unwrap();

                            if let Err(err) = config::set("games.wine.sync", value) {
                                sender.input(PreferencesAppMsg::ShowToast {
                                    title: String::from("Failed to update property"),
                                    message: Some(err.to_string())
                                })
                            }
                        }
                    },

                    adw::SwitchRow {
                        set_title: "Borderless window",

                        set_active: config::get().games.wine.borderless,

                        connect_active_notify[sender] => move |switch| {
                            if let Err(err) = config::set("games.wine.borderless", switch.is_active()) {
                                sender.input(PreferencesAppMsg::ShowToast {
                                    title: String::from("Failed to update property"),
                                    message: Some(err.to_string())
                                })
                            }
                        }
                    }
                },

                add = &adw::PreferencesGroup {
                    set_title: "Gaming",

                    adw::ComboRow {
                        set_title: "HUD",

                        set_model: Some(&gtk::StringList::new(&[
                            "None",
                            "DXVK",
                            "MangoHUD"
                        ])),

                        set_selected: match config::get().games.enhancements.hud {
                            HUD::None     => 0,
                            HUD::DXVK     => 1,
                            HUD::MangoHUD => 2
                        },

                        connect_selected_notify[sender] => move |row| {
                            let hud = [
                                HUD::None,
                                HUD::DXVK,
                                HUD::MangoHUD 
                            ][row.selected() as usize];

                            let value = serde_json::to_value(hud).unwrap();

                            if let Err(err) = config::set("games.enhancements.hud", value) {
                                sender.input(PreferencesAppMsg::ShowToast {
                                    title: String::from("Failed to update property"),
                                    message: Some(err.to_string())
                                })
                            }
                        }
                    },

                    adw::ExpanderRow {
                        set_title: "FSR",
                        set_subtitle: "Upscales game to your monitor size. To use select lower resolution in the game's settings and press Alt+Enter",

                        add_row = &adw::SwitchRow {
                            set_title: "Enabled",
                            set_subtitle: "Render the game in lower resolution and upscale it",

                            set_active: config::get().games.enhancements.fsr.enabled,

                            connect_active_notify[sender] => move |switch| {
                                if let Err(err) = config::set("games.enhancements.fsr.enabled", switch.is_active()) {
                                    sender.input(PreferencesAppMsg::ShowToast {
                                        title: String::from("Failed to update property"),
                                        message: Some(err.to_string())
                                    })
                                }
                            }
                        },

                        add_row = &adw::ComboRow {
                            set_title: "Quality",
                            set_subtitle: "Specifies game rendering resolution. Ultra quality renders the game in 1.3 smaller resolution, while preformance in 2.0 smaller",

                            set_model: Some(&gtk::StringList::new(&[
                                "Ultra quality",
                                "Quality",
                                "Balanced",
                                "Performance"
                            ])),

                            set_selected: match config::get().games.enhancements.fsr.quality {
                                FsrQuality::Ultra       => 0,
                                FsrQuality::Quality     => 1,
                                FsrQuality::Balanced    => 2,
                                FsrQuality::Performance => 3
                            },
    
                            connect_selected_notify[sender] => move |row| {
                                let hud = [
                                    FsrQuality::Ultra,
                                    FsrQuality::Quality,
                                    FsrQuality::Balanced,
                                    FsrQuality::Performance
                                ][row.selected() as usize];
    
                                let value = serde_json::to_value(hud).unwrap();
    
                                if let Err(err) = config::set("games.enhancements.fsr.quality", value) {
                                    sender.input(PreferencesAppMsg::ShowToast {
                                        title: String::from("Failed to update property"),
                                        message: Some(err.to_string())
                                    })
                                }
                            }
                        },

                        add_row = &adw::SpinRow {
                            set_title: "Sharpening strength",
                            set_subtitle: "Image sharpening strength where 0 is maximal sharpness",

                            set_adjustment: Some(&gtk::Adjustment::new(
                                config::get().games.enhancements.fsr.strength as f64,
                                0.0, 5.0, 1.0, 1.0, 0.0
                            )),

                            connect_value_notify[sender] => move |row| {
                                if let Err(err) = config::set("games.enhancements.fsr.strength", row.value() as u64) {
                                    sender.input(PreferencesAppMsg::ShowToast {
                                        title: String::from("Failed to update property"),
                                        message: Some(err.to_string())
                                    })
                                }
                            }
                        }
                    },

                    adw::SwitchRow {
                        set_title: "Gamemode",
                        set_subtitle: "Prioritize the game over the rest of the processes",

                        set_active: config::get().games.enhancements.gamemode,

                        connect_active_notify[sender] => move |switch| {
                            if let Err(err) = config::set("games.enhancements.gamemode", switch.is_active()) {
                                sender.input(PreferencesAppMsg::ShowToast {
                                    title: String::from("Failed to update property"),
                                    message: Some(err.to_string())
                                })
                            }
                        }
                    }
                },

                add = &adw::PreferencesGroup {
                    set_title: "Components",

                    adw::ComboRow {
                        set_title: "Wine version",

                        set_model: Some(&gtk::StringList::new(&[
                            "latest"
                        ]))
                    },

                    adw::ComboRow {
                        set_title: "DXVK version",

                        set_model: Some(&gtk::StringList::new(&[
                            "latest"
                        ]))
                    },

                    adw::SwitchRow {
                        set_title: "Install corefonts",

                        set_active: config::get().components.wine.prefix.install_corefonts,

                        connect_active_notify[sender] => move |switch| {
                            if let Err(err) = config::set("components.wine.prefix.install_corefonts", switch.is_active()) {
                                sender.input(PreferencesAppMsg::ShowToast {
                                    title: String::from("Failed to update property"),
                                    message: Some(err.to_string())
                                })
                            }
                        }
                    }
                }
            }
        }
    }

    async fn init(
        parent: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self {
            // todo
        };

        let widgets = view_output!();

        widgets.window.set_transient_for(Some(&parent));

        unsafe {
            WINDOW = Some(widgets.window.clone());
        }

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>) {
        match msg {
            PreferencesAppMsg::ShowToast { title, message } => {
                let window = unsafe {
                    WINDOW.as_ref().unwrap_unchecked()
                };

                let toast = adw::Toast::new(&title);

                // toast.set_timeout(7);

                if let Some(message) = message {
                    toast.set_button_label(Some("Details"));

                    let dialog = adw::MessageDialog::new(
                        Some(window),
                        Some(&title),
                        Some(&message)
                    );

                    dialog.add_response("close", "Close");
                    dialog.add_response("save", "Save");

                    dialog.set_response_appearance("save", adw::ResponseAppearance::Suggested);

                    dialog.connect_response(Some("save"), |_, _| {
                        if let Err(err) = open::that(crate::DEBUG_FILE.as_path()) {
                            tracing::error!("Failed to open debug file: {err}");
                        }
                    });

                    toast.connect_button_clicked(move |_| {
                        dialog.present();
                    });
                }

                window.add_toast(toast);
            }
        }
    }
}
