use relm4::prelude::*;
use relm4::component::*;

use gtk::prelude::*;
use adw::prelude::*;

use crate::controller::Controller;
use crate::ui::windows::preferences::PreferencesApp;

static mut MAIN_WINDOW: Option<adw::ApplicationWindow> = None;
static mut PREFERENCES_WINDOW: Option<AsyncController<PreferencesApp>> = None;

pub struct MainApp {
    toast_overlay: adw::ToastOverlay
}

#[derive(Debug)]
pub enum MainAppMsg {
    OpenPreferences,

    ShowToast {
        title: String,
        message: Option<String>
    }
}

#[relm4::component(pub)]
impl SimpleComponent for MainApp {
    type Init = ();
    type Input = MainAppMsg;
    type Output = ();

    view! {
        window = adw::ApplicationWindow {
            // w = 1280 / 730 * h, where 1280x730 is default background picture resolution
            set_default_size: (1094, 624),

            set_title: Some("Anime Games Launcher"),

            #[local_ref]
            toast_overlay -> adw::ToastOverlay {
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    adw::HeaderBar {
                        add_css_class: "flat",

                        pack_end = &gtk::Button {
                            set_icon_name: "emblem-system-symbolic",

                            connect_clicked => MainAppMsg::OpenPreferences
                        }
                    },

                    gtk::Box {
                        set_halign: gtk::Align::Center,
                        set_spacing: 8,

                        gtk::ToggleButton {
                            add_css_class: "flat",

                            gtk::Image {
                                set_width_request: 48,
                                set_height_request: 48,

                                set_from_resource: Some(&crate::resource!("images/games/genshin/icon"))
                            }
                        },

                        gtk::ToggleButton {
                            add_css_class: "flat",

                            gtk::Image {
                                set_width_request: 48,
                                set_height_request: 48,

                                set_from_resource: Some(&crate::resource!("images/games/honkai/icon"))
                            }
                        },

                        gtk::ToggleButton {
                            add_css_class: "flat",

                            gtk::Image {
                                set_width_request: 48,
                                set_height_request: 48,

                                set_from_resource: Some(&crate::resource!("images/games/star-rail/icon"))
                            }
                        },

                        gtk::ToggleButton {
                            add_css_class: "flat",

                            gtk::Image {
                                set_width_request: 48,
                                set_height_request: 48,

                                set_from_resource: Some(&crate::resource!("images/games/pgr/icon"))
                            }
                        },

                        gtk::Button {
                            set_valign: gtk::Align::Center,

                            add_css_class: "flat",

                            gtk::Image {
                                set_width_request: 48,
                                set_height_request: 48,

                                set_icon_name: Some("grid-large-symbolic")
                            }
                        }
                    }
                }
            }
        }
    }

    fn init(
        _parent: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            toast_overlay: adw::ToastOverlay::new()
        };

        let toast_overlay = &model.toast_overlay;

        let widgets = view_output!();

        unsafe {
            MAIN_WINDOW = Some(widgets.window.clone());

            PREFERENCES_WINDOW = Some(PreferencesApp::builder()
                .launch(widgets.window.clone())
                .detach());
        }

        Controller::register_main_sender(sender.input_sender().clone());

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            MainAppMsg::OpenPreferences => unsafe {
                PREFERENCES_WINDOW.as_ref()
                    .unwrap_unchecked()
                    .widget()
                    .present();
            }

            MainAppMsg::ShowToast { title, message } => {
                let window = unsafe {
                    MAIN_WINDOW.as_ref().unwrap_unchecked()
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

                self.toast_overlay.add_toast(toast);
            }
        }
    }
}
