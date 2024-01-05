use relm4::prelude::*;
use adw::prelude::*;

use crate::games::integrations::standards::addons::Addon;

#[derive(Debug)]
pub struct AddonRowComponent {
    pub info: Addon
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddonRowComponentInput {

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddonRowComponentOutput {

}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for AddonRowComponent {
    type Init = Addon;
    type Input = AddonRowComponentInput;
    type Output = AddonRowComponentOutput;

    view! {
        #[root]
        adw::ActionRow {
            set_title: &model.info.title
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self {
            info: init
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        
    }
}
