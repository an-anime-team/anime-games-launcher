use adw::prelude::*;
use gtk::prelude::*;

use relm4::{factory::*, prelude::*};

#[derive(Debug)]
pub struct MaintainersRowFactory {
    pub name: String,
    pub email: Option<String>,
}

#[relm4::factory(pub, async)]
impl AsyncFactoryComponent for MaintainersRowFactory {
    type Init = String;
    type Input = ();
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = adw::ExpanderRow;

    view! {
        #[root]
        adw::ActionRow {
            set_title: &self.name,
            set_subtitle: &self.email.clone().unwrap_or(String::new()),
        }
    }

    async fn init_model(
        init: Self::Init,
        index: &DynamicIndex,
        sender: AsyncFactorySender<Self>,
    ) -> Self {
        if let Some(start) = init.find('<') {
            if let Some(end) = init.find('>') {
                if end > start {
                    return Self {
                        name: init[0..start].to_string(),
                        email: Some(init[start + 1..end].to_string()),
                    };
                }
            }
        }
        Self {
            name: init,
            email: None,
        }
    }
}
