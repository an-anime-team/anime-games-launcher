use relm4::prelude::*;
use gtk::prelude::*;
use adw::prelude::*;

use crate::config;

static mut WINDOW: Option<adw::PreferencesWindow> = None;

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

            set_hide_on_close: true,
            set_modal: true,
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

                    adw::ActionRow {
                        set_title: "Verify games",
                        set_subtitle: "Verify games installations after installation or updating",

                        add_suffix = &gtk::Switch {
                            set_valign: gtk::Align::Center,

                            set_active: config::get().general.verify_games,

                            connect_state_notify[sender] => move |switch| {
                                if let Err(err) = config::set("general.verify_games", switch.is_active()) {
                                    sender.input(PreferencesAppMsg::ShowToast {
                                        title: String::from("Failed to update property"),
                                        message: Some(err.to_string())
                                    })
                                }
                            }
                        }
                    },

                    adw::ActionRow {
                        set_title: "Update games",
                        set_subtitle: "Download updates for installed games when they become available",

                        add_suffix = &gtk::Switch {
                            set_valign: gtk::Align::Center
                        }
                    },

                    adw::ActionRow {
                        set_title: "Pre-download updates",
                        set_subtitle: "Pre-download updates for installed games when they become available",

                        add_suffix = &gtk::Switch {
                            set_valign: gtk::Align::Center
                        }
                    }
                },

                add = &adw::PreferencesGroup {
                    set_title: "Wine",

                    adw::ComboRow {
                        set_title: "Language",
                        set_subtitle: "Language used in the wine environment. Can fix keyboard layout issues",

                        set_model: Some(&gtk::StringList::new(&[
                            "English"
                        ]))
                    },

                    adw::ComboRow {
                        set_title: "Synchronization",
                        set_subtitle: "Technology used to synchronize inner wine events",

                        set_model: Some(&gtk::StringList::new(&[
                            "None",
                            "ESync",
                            "FSync"
                        ]))
                    },

                    adw::ActionRow {
                        set_title: "Borderless window",

                        add_suffix = &gtk::Switch {
                            set_valign: gtk::Align::Center
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
                        ]))
                    },

                    adw::ComboRow {
                        set_title: "FSR",
                        set_subtitle: "Upscales game to your monitor size. To use select lower resolution in the game's settings and press Alt+Enter",

                        set_model: Some(&gtk::StringList::new(&[
                            "Ultra quality",
                            "Quality",
                            "Balanced",
                            "Performance"
                        ])),

                        add_suffix = &gtk::Switch {
                            set_valign: gtk::Align::Center
                        }
                    },

                    adw::ActionRow {
                        set_title: "Gamemode",
                        set_subtitle: "Prioritize the game over the rest of the processes",

                        add_suffix = &gtk::Switch {
                            set_valign: gtk::Align::Center
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

                    adw::ActionRow {
                        set_title: "Install corefonts",

                        add_suffix = &gtk::Switch {
                            set_valign: gtk::Align::Center,

                            set_active: config::get().components.wine.prefix.install_corefonts,

                            connect_state_notify[sender] => move |switch| {
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
                    // dialog.add_response("save", &tr!("save"));

                    // dialog.set_response_appearance("save", adw::ResponseAppearance::Suggested);

                    // dialog.connect_response(Some("save"), |_, _| {
                    //     if let Err(err) = open::that(crate::DEBUG_FILE.as_os_str()) {
                    //         tracing::error!("Failed to open debug file: {err}");
                    //     }
                    // });

                    toast.connect_button_clicked(move |_| {
                        dialog.present();
                    });
                }

                window.add_toast(toast);
            }
        }
    }
}
