use std::borrow::Borrow;

use adw::prelude::*;
use gtk::prelude::*;

use relm4::factory::*;
use relm4::prelude::*;

use crate::ui::components::graph::{GraphComponent, GraphComponentMsg};

#[derive(Debug)]
pub struct DownloadsPageApp {
    pub graph: AsyncController<GraphComponent>,
}

#[derive(Debug, Clone)]
pub enum DownloadsPageAppMsg {}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for DownloadsPageApp {
    type Init = ();
    type Input = DownloadsPageAppMsg;
    type Output = ();

    view! {
        #[root]
        adw::PreferencesPage {
            add = &adw::PreferencesGroup {
                gtk::Box {
                    model.graph.widget(),
                }
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self {
            graph: GraphComponent::builder().launch(()).detach(),
        };
        let widgets = view_output!();

        model
            .graph
            .borrow()
            .sender()
            .send(GraphComponentMsg::PushPoint(10.0))
            .unwrap();

        model
            .graph
            .borrow()
            .sender()
            .send(GraphComponentMsg::PushPoint(5.0))
            .unwrap();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {}
    }
}
