use relm4::prelude::*;
use relm4::component::*;

use gtk::prelude::*;
use adw::prelude::*;

use crate::config;

static mut WINDOW: Option<adw::PreferencesWindow> = None;

pub struct PreferencesApp {

}

#[derive(Debug, Clone)]
pub enum PreferencesAppMsg {
    ShowTitle {
        title: String,
        message: Option<String>
    }
}

#[relm4::component(async, pub)]
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
        }
    }

    async fn init(
        parent: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self {

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
            PreferencesAppMsg::ShowTitle { title, message } => {
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