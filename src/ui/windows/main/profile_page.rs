use gtk::prelude::*;
use adw::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub struct ProfilePageApp {

}

#[derive(Debug, Clone)]
pub enum ProfilePageAppMsg {
    
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for ProfilePageApp {
    type Init = ();
    type Input = ProfilePageAppMsg;
    type Output = ();

    view! {
        root = adw::PreferencesPage {
            add = &adw::PreferencesGroup {
                set_title: "Wine profiles",

                #[wrap(Some)]
                set_header_suffix = &gtk::Button {
                    add_css_class: "flat",

                    set_label: "New"
                },

                adw::ActionRow {
                    set_title: "Default profile",
                    set_subtitle: "Wine-Staging-TkG 9.0 ∙ DXVK 2.1"
                }
            }
        }
    }

    async fn init(_init: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let model = Self {

        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            
        }
    }
}
