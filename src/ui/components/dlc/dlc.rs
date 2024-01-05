use relm4::prelude::*;
use relm4::component::*;
use relm4::factory::*;

use gtk::prelude::*;
use adw::prelude::*;

use crate::games::integrations::standards::dlc::{
    Dlc,
    DlcGroup
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DlcComponent {
    pub info: Dlc
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DlcComponentInput {

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DlcComponentOutput {

}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for DlcComponent {
    type Init = Dlc;
    type Input = DlcComponentInput;
    type Output = DlcComponentOutput;

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
