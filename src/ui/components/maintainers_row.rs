use adw::prelude::*;
use gtk::prelude::*;

use relm4::{factory::*, prelude::*};

#[derive(Debug)]
pub struct MaintainersRowFactory {
    pub name: String,
    pub contact: Option<String>,
}

#[derive(Debug)]
pub enum MaintainersRowFactoryMsg {
    /// Uses the value in contact if present and attempts to open
    /// a supported application with xdg-open so the user can
    /// contact the maintainer - currently the supported appplications
    /// are as follows:
    /// - No application: "John Doe"
    /// - Email: "John Doe <johndoe@mail.com>"
    /// - URL: "John Doe <https://johndoe.com>"
    /// - Phone number: "John Doe <+123123123123>"
    /// - Any URI supported by `xdg-open`: "John Doe <magnet:123123123123>"
    Activate,
}

#[relm4::factory(pub, async)]
impl AsyncFactoryComponent for MaintainersRowFactory {
    type Init = String;
    type Input = MaintainersRowFactoryMsg;
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = adw::ExpanderRow;

    view! {
        #[root]
        adw::ActionRow {
            set_title: &self.name,
            set_subtitle: &self.contact.clone().unwrap_or(String::new()),
            set_tooltip: "Open contact",
            set_activatable: true,
            connect_activated => MaintainersRowFactoryMsg::Activate,
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
                        contact: Some(init[start + 1..end].to_string()),
                    };
                }
            }
        }
        Self {
            name: init,
            contact: None,
        }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncFactorySender<Self>) {
        match msg {
            MaintainersRowFactoryMsg::Activate => {
                if let Some(contact) = &self.contact {
                     // Email could contain url characters so check url first
                    // Assume uri is valid if contact is present
                    let uri = if contact.starts_with("https://") || contact.starts_with("https://") {
                        contact.to_string()
                    } else if contact.contains('@') && contact.split('@').count() == 2 && contact.chars().all(|c| c.is_alphanumeric() || ".-_@".contains(c)) {
                        format!("mailto:{}", contact)
                    } else if contact.chars().all(|c| c.is_digit(10) || " +-".contains(c)) {
                        format!("tel:{}", contact.replace([' ', '-'], ""))
                    } else {
                        contact.to_string()
                    };

                    let out = std::process::Command::new("xdg-open")
                        .arg(&uri)
                        .output()
                        .expect("Failed to open contact");

                    if out.status.success() {
                        println!("Opened: {}", uri);
                    }
                }
            }
        }
    }
}
