use gtk::prelude::*;
use adw::prelude::*;
use relm4::prelude::*;

pub static mut WINDOW: Option<adw::Window> = None;

#[derive(Debug)]
pub struct CreateWineProfileApp {

}

#[derive(Debug, Clone)]
pub enum CreateWineProfileAppMsg {
    Create {
        name: String
    }
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for CreateWineProfileApp {
    type Init = ();
    type Input = CreateWineProfileAppMsg;
    type Output = ();

    view! {
        window = adw::Window {
            set_size_request: (700, 560),
            set_title: Some("Create profile"),

            set_hide_on_close: true,
            set_modal: true,

            add_css_class?: crate::APP_DEBUG.then_some("devel"),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar {
                    add_css_class: "flat"
                },

                adw::PreferencesPage {
                    add = &adw::PreferencesGroup {
                        #[name = "profile_name_row"]
                        adw::EntryRow {
                            set_title: "Profile name"
                        }
                    },

                    add = &adw::PreferencesGroup {
                        adw::ComboRow {
                            set_title: "Wine version",

                            set_model: Some(&gtk::StringList::new(&[
                                "Wine-Staging-TkG 9.1",
                                "Wine-Staging-TkG 9.8"
                            ]))
                        },

                        adw::ComboRow {
                            set_title: "DXVK version",

                            set_model: Some(&gtk::StringList::new(&[
                                "DXVK 2.1",
                                "DXVK 2.0"
                            ]))
                        },

                        adw::ExpanderRow {
                            set_title: "Containerization",

                            add_row = &adw::SwitchRow {
                                set_title: "Enabled"
                            },

                            add_row = &adw::ComboRow {
                                set_title: "System",

                                set_model: Some(&gtk::StringList::new(&[
                                    "Alpine 3.19",
                                    "Alpine 3.18"
                                ]))
                            }
                        }
                    },

                    add = &adw::PreferencesGroup {
                        gtk::Button {
                            add_css_class: "pill",
                            add_css_class: "suggested-action",

                            set_label: "Create",

                            connect_clicked[sender, profile_name_row] => move |_| {
                                sender.input(CreateWineProfileAppMsg::Create {
                                    name: profile_name_row.text().to_string()
                                })
                            }
                        }
                    }
                }
            }
        }
    }

    async fn init(_init: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let model = Self {

        };

        let widgets = view_output!();

        unsafe {
            WINDOW = Some(widgets.window.clone());
        }

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            CreateWineProfileAppMsg::Create { name } => {

            }
        }
    }
}
