use relm4::prelude::*;
use gtk::prelude::*;
use adw::prelude::*;

use crate::tr;

use crate::i18n;
use crate::config;

use crate::components::wine::Wine;
use crate::components::dxvk::Dxvk;

use crate::config::games::wine::prelude::*;
use crate::config::games::enhancements::prelude::*;

pub static mut WINDOW: Option<adw::PreferencesWindow> = None;

pub struct PreferencesApp {
    wine_versions: Vec<Wine>,
    dxvk_versions: Vec<Dxvk>,

    selected_wine: Wine,
    selected_dxvk: Dxvk
}

#[derive(Debug, Clone)]
pub enum PreferencesAppMsg {
    SelectWineVersion(u32),
    SelectDxvkVersion(u32),

    ShowToast {
        title: String,
        message: Option<String>
    }
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for PreferencesApp {
    type Init = adw::Window;
    type Input = PreferencesAppMsg;
    type Output = ();

    view! {
        window = adw::PreferencesWindow {
            set_default_size: (700, 560),
            set_title: Some(&tr!("preferences")),

            set_modal: true,
            set_hide_on_close: true,
            set_search_enabled: true,

            add = &adw::PreferencesPage {
                add = &adw::PreferencesGroup {
                    set_title: &tr!("preferences--general"),

                    adw::ComboRow {
                        set_title: &tr!("general-launcher-language"),
                        set_subtitle: &tr!("general-launcher-language-description"),

                        set_model: Some(&{
                            let model = gtk::StringList::new(&[]);

                            for lang in i18n::SUPPORTED_LANGUAGES {
                                model.append(&tr!(i18n::format_language(lang).as_str()));
                            }

                            model
                        }),

                        set_selected: {
                            let selected = config::get().general.language;

                            i18n::SUPPORTED_LANGUAGES.iter()
                                .position(|lang| i18n::format_language(lang) == selected)
                                .unwrap_or(0) as u32
                        },

                        connect_selected_notify[sender] => move |row| {
                            let language = i18n::format_language(i18n::SUPPORTED_LANGUAGES
                                .get(row.selected() as usize)
                                .unwrap_or(&i18n::SUPPORTED_LANGUAGES[0]));

                            if let Err(err) = config::set("general.language", language) {
                                sender.input(PreferencesAppMsg::ShowToast {
                                    title: tr!("config-property-update-failed"),
                                    message: Some(err.to_string())
                                })
                            }
                        }
                    },

                    adw::SwitchRow {
                        set_title: &tr!("general-verify-games"),
                        set_subtitle: &tr!("general-verify-games-description"),

                        set_active: config::get().general.verify_games,

                        connect_active_notify[sender] => move |switch| {
                            if let Err(err) = config::set("general.verify_games", switch.is_active()) {
                                sender.input(PreferencesAppMsg::ShowToast {
                                    title: tr!("config-property-update-failed"),
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
                    set_title: &tr!("preferences--wine"),

                    adw::ComboRow {
                        set_title: &tr!("wine-language"),
                        set_subtitle: &tr!("wine-language-description"),

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
                                    title: tr!("config-property-update-failed"),
                                    message: Some(err.to_string())
                                })
                            }
                        }
                    },

                    adw::ComboRow {
                        set_title: &tr!("wine-sync"),
                        set_subtitle: &tr!("wine-sync-description"),

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
                                    title: tr!("config-property-update-failed"),
                                    message: Some(err.to_string())
                                })
                            }
                        }
                    },

                    adw::SwitchRow {
                        set_title: &tr!("wine-borderless"),

                        set_active: config::get().games.wine.borderless,

                        connect_active_notify[sender] => move |switch| {
                            if let Err(err) = config::set("games.wine.borderless", switch.is_active()) {
                                sender.input(PreferencesAppMsg::ShowToast {
                                    title: tr!("config-property-update-failed"),
                                    message: Some(err.to_string())
                                })
                            }
                        }
                    }
                },

                add = &adw::PreferencesGroup {
                    set_title: &tr!("preferences--gaming"),

                    adw::ComboRow {
                        set_title: &tr!("game-hud"),

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
                                    title: tr!("config-property-update-failed"),
                                    message: Some(err.to_string())
                                })
                            }
                        }
                    },

                    adw::ExpanderRow {
                        set_title: &tr!("game-fsr"),
                        set_subtitle: &tr!("game-fsr-description"),

                        add_row = &adw::SwitchRow {
                            set_title: &tr!("game-fsr-enabled"),
                            set_subtitle: &tr!("game-fsr-enabled-description"),

                            set_active: config::get().games.enhancements.fsr.enabled,

                            connect_active_notify[sender] => move |switch| {
                                if let Err(err) = config::set("games.enhancements.fsr.enabled", switch.is_active()) {
                                    sender.input(PreferencesAppMsg::ShowToast {
                                        title: tr!("config-property-update-failed"),
                                        message: Some(err.to_string())
                                    })
                                }
                            }
                        },

                        add_row = &adw::ComboRow {
                            set_title: &tr!("game-fsr-quality"),
                            set_subtitle: &tr!("game-fsr-quality-description"),

                            set_model: Some(&gtk::StringList::new(&[
                                &tr!("game-fsr-quality-ultra"),
                                &tr!("game-fsr-quality-quality"),
                                &tr!("game-fsr-quality-balanced"),
                                &tr!("game-fsr-quality-performance")
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
                                        title: tr!("config-property-update-failed"),
                                        message: Some(err.to_string())
                                    })
                                }
                            }
                        },

                        add_row = &adw::SpinRow {
                            set_title: &tr!("game-fsr-strength"),
                            set_subtitle: &tr!("game-fsr-strength-description"),

                            set_adjustment: Some(&gtk::Adjustment::new(
                                config::get().games.enhancements.fsr.strength as f64,
                                0.0, 5.0, 1.0, 1.0, 0.0
                            )),

                            connect_value_notify[sender] => move |row| {
                                if let Err(err) = config::set("games.enhancements.fsr.strength", row.value() as u64) {
                                    sender.input(PreferencesAppMsg::ShowToast {
                                        title: tr!("config-property-update-failed"),
                                        message: Some(err.to_string())
                                    })
                                }
                            }
                        }
                    },

                    adw::SwitchRow {
                        set_title: &tr!("game-gamemode"),
                        set_subtitle: &tr!("game-gamemode-description"),

                        set_active: config::get().games.enhancements.gamemode,

                        connect_active_notify[sender] => move |switch| {
                            if let Err(err) = config::set("games.enhancements.gamemode", switch.is_active()) {
                                sender.input(PreferencesAppMsg::ShowToast {
                                    title: tr!("config-property-update-failed"),
                                    message: Some(err.to_string())
                                })
                            }
                        }
                    }
                },

                add = &adw::PreferencesGroup {
                    set_title: &tr!("preferences--components"),

                    adw::ComboRow {
                        set_title: &tr!("components-wine"),
                        set_subtitle: &tr!("components-wine-description"),

                        set_model: Some(&{
                            let strings = gtk::StringList::new(&[]);

                            strings.append(&tr!("components-wine-latest"));

                            for version in &model.wine_versions {
                                strings.append(&version.title);
                            }

                            strings
                        }),

                        set_selected: model.wine_versions.iter()
                            .position(|version| version == &model.selected_wine)
                            .unwrap_or(0) as u32,

                        connect_selected_notify[sender] => move |row| {
                            sender.input(PreferencesAppMsg::SelectWineVersion(row.selected()));
                        }
                    },

                    adw::ComboRow {
                        set_title: &tr!("components-dxvk"),
                        set_subtitle: &tr!("components-dxvk-description"),

                        set_model: Some(&{
                            let strings = gtk::StringList::new(&[]);

                            strings.append(&tr!("components-dxvk-latest"));

                            for version in &model.dxvk_versions {
                                strings.append(&version.name);
                            }

                            strings
                        }),

                        set_selected: model.dxvk_versions.iter()
                            .position(|version| version == &model.selected_dxvk)
                            .unwrap_or(0) as u32,

                        connect_selected_notify[sender] => move |row| {
                            sender.input(PreferencesAppMsg::SelectDxvkVersion(row.selected()));
                        }
                    },

                    adw::SwitchRow {
                        set_title: &tr!("components-install-corefonts"),
                        set_subtitle: &tr!("components-install-corefonts-description"),

                        set_active: config::get().components.wine.prefix.install_corefonts,

                        connect_active_notify[sender] => move |switch| {
                            if let Err(err) = config::set("components.wine.prefix.install_corefonts", switch.is_active()) {
                                sender.input(PreferencesAppMsg::ShowToast {
                                    title: tr!("config-property-update-failed"),
                                    message: Some(err.to_string())
                                })
                            }
                        }
                    }
                }
            }
        }
    }

    async fn init(parent: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let model = Self {
            wine_versions: Wine::versions()
                .unwrap()
                .into_iter()
                .take(12)
                .collect(),

            dxvk_versions: Dxvk::versions()
                .unwrap()
                .into_iter()
                .take(12)
                .collect(),

            selected_wine: Wine::from_config().unwrap(),
            selected_dxvk: Dxvk::from_config().unwrap()
        };

        let widgets = view_output!();

        widgets.window.set_transient_for(Some(&parent));

        unsafe {
            WINDOW = Some(widgets.window.clone());
        }

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            PreferencesAppMsg::SelectWineVersion(index) => {
                let version = if index == 0 {
                    tr!("components-wine-latest")
                } else {
                    self.wine_versions[index as usize - 1].name.clone()
                };

                if let Err(err) = config::set("components.wine.version", version) {
                    sender.input(PreferencesAppMsg::ShowToast {
                        title: tr!("config-property-update-failed"),
                        message: Some(err.to_string())
                    })
                }
            }

            PreferencesAppMsg::SelectDxvkVersion(index) => {
                let version = if index == 0 {
                    tr!("components-dxvk-latest")
                } else {
                    self.dxvk_versions[index as usize - 1].version.clone()
                };

                if let Err(err) = config::set("components.dxvk.version", version) {
                    sender.input(PreferencesAppMsg::ShowToast {
                        title: tr!("config-property-update-failed"),
                        message: Some(err.to_string())
                    })
                }
            }

            PreferencesAppMsg::ShowToast { title, message } => {
                let window = unsafe {
                    WINDOW.as_ref().unwrap_unchecked()
                };

                let toast = adw::Toast::new(&title);

                // toast.set_timeout(7);

                if let Some(message) = message {
                    toast.set_button_label(Some(&tr!("dialog-toast-details")));

                    let dialog = adw::MessageDialog::new(
                        Some(window),
                        Some(&title),
                        Some(&message)
                    );

                    dialog.add_response("close", &tr!("dialog-close"));
                    dialog.add_response("save", &tr!("dialog-save"));

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
