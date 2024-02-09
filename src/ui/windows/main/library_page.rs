use gtk::prelude::*;
use adw::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub struct LibraryPageApp {

}

#[derive(Debug, Clone)]
pub enum LibraryPageAppMsg {
    
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for LibraryPageApp {
    type Init = ();
    type Input = LibraryPageAppMsg;
    type Output = ();

    view! {
        root = adw::PreferencesPage {
            add = &adw::PreferencesGroup {
                set_title: "Library"
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
