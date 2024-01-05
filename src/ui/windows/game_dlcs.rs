use relm4::prelude::*;
use relm4::component::*;
use relm4::factory::*;

use gtk::prelude::*;
use adw::prelude::*;

use crate::games::integrations::standards::dlc::{
    Dlc,
    DlcGroup
};

use crate::{
    config,
    STARTUP_CONFIG
};

static mut WINDOW: Option<adw::ApplicationWindow> = None;

pub struct GameDlcsApp {
    
}

#[derive(Debug, Clone)]
pub enum GameDlcsAppMsg {
    
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for GameDlcsApp {
    type Init = adw::ApplicationWindow;
    type Input = GameDlcsAppMsg;
    type Output = ();

    view! {
        window = adw::ApplicationWindow {
            set_default_size: (700, 560),
            set_title: Some("Game DLCs"),

            set_hide_on_close: true,
            set_modal: true,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar {
                    add_css_class: "flat"
                },

                adw::PreferencesPage {
                    add = &adw::PreferencesGroup {
                        set_title: "Voiceovers",

                        adw::ActionRow {
                            set_title: "English"
                        },

                        adw::ActionRow {
                            set_title: "Japanese"
                        },

                        adw::ActionRow {
                            set_title: "Korean"
                        },

                        adw::ActionRow {
                            set_title: "Chinese"
                        }
                    },

                    add = &adw::PreferencesGroup {
                        set_title: "Extras",

                        adw::ActionRow {
                            set_title: "FPS Unlocker"
                        }
                    },

                    add = &adw::PreferencesGroup {
                        set_title: "Patch",

                        adw::ActionRow {
                            set_title: "Jadeite",
                            set_subtitle: "Required",

                            set_activatable: false
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
            
        }
    }
}
