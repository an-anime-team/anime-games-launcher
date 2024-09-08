use std::borrow::Borrow;

use adw::prelude::*;
use gtk::prelude::*;

use relm4::factory::*;
use relm4::prelude::*;

use crate::ui::components::graph::{Graph, GraphMsg};

#[derive(Debug)]
pub struct DownloadsPageApp {
    pub graph: AsyncController<Graph>,
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
            graph: Graph::builder().launch(()).detach(),
        };
        let widgets = view_output!();

        model
            .graph
            .borrow()
            .sender()
            .send(GraphMsg::PushPoints(vec![
                5.1, 10.9, 12.0, 6.0, 3.0, 3.0, 4.0, 5.0, 9.0, 7.0, 1.0, 1.0, 2.5, 6.8, 6.6, 15.5,
                17.1, 0.9, 6.6,
            ]))
            .unwrap();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {}
    }
}
