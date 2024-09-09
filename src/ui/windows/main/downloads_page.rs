use adw::prelude::*;
use gtk::prelude::*;

use relm4::factory::*;
use relm4::prelude::*;

use crate::ui::components::downloads_list::DownloadsRow;
use crate::ui::components::graph::{Graph, GraphInit, GraphMsg};

#[derive(Debug)]
pub struct DownloadsPageApp {
    pub graph: AsyncController<Graph>,
    pub active_download: AsyncController<DownloadsRow>,
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
            },
            add = &adw::PreferencesGroup {
                set_title: "Download",
                #[local_ref]
                grid -> gtk::Grid,
            },
            add = &adw::PreferencesGroup {
                set_title: "Current",
                model.active_download.widget(),
            },
            add = &adw::PreferencesGroup {
                set_title: "Scheduled",
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self {
            graph: Graph::builder()
                .launch(GraphInit::new(800, 150, (1.0, 1.0, 1.0)))
                .detach(),
            active_download: DownloadsRow::builder().launch(()).detach(),
        };

        let grid = gtk::Grid::new();

        let current_speed = adw::ActionRow::builder().title("Current. speed:").build();
        current_speed.add_suffix(&gtk::Label::new(Some("2.5 MB")));

        let avg_speed = adw::ActionRow::builder().title("Avg. speed:").build();
        avg_speed.add_suffix(&gtk::Label::new(Some("2.2 MB")));

        grid.attach(&current_speed, 0, 0, 1, 1);
        grid.attach(&avg_speed, 1, 0, 1, 1);

        let widgets = view_output!();

        model
            .graph
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
