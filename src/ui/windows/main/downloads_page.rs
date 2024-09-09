use adw::prelude::*;
use gtk::prelude::*;

use relm4::factory::*;
use relm4::prelude::*;

use crate::ui::components::downloads_row::{
    DownloadsRow, DownloadsRowFactory, DownloadsRowFactoryMsg, DownloadsRowInit,
};
use crate::ui::components::graph::{Graph, GraphInit, GraphMsg};

#[derive(Debug)]
pub struct DownloadsPageApp {
    pub graph: AsyncController<Graph>,
    pub active: AsyncController<DownloadsRow>,
    pub scheduled: AsyncFactoryVecDeque<DownloadsRowFactory>,
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
                gtk::Grid {
                    attach[0, 0, 1, 1] = &adw::ActionRow {
                        set_title: "Current speed:",
                        add_suffix = &gtk::Label {
                            set_text: "2.5 MB/s",
                        }
                    },
                    attach[1, 0, 1, 1] = &adw::ActionRow {
                        set_title: "Avg. speed:",
                        add_suffix = &gtk::Label {
                            set_text: "2.2 MB/s",
                        }
                    },
                },
            },
            add = &adw::PreferencesGroup {
                set_title: "active",
                model.active.widget(),
            },
            add = model.scheduled.widget() {
                set_title: "Scheduled",
            },
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let mut model = Self {
            graph: Graph::builder()
                .launch(GraphInit::new(800, 150, (1.0, 1.0, 1.0)))
                .detach(),
            active: DownloadsRow::builder()
                .launch(DownloadsRowInit::new(
                    String::from("/path/to/card.jpg"),
                    "Genshin Impact",
                    "5.0.0",
                    "Global",
                    64500000000,
                    true,
                ))
                .detach(),
            scheduled: AsyncFactoryVecDeque::builder().launch_default().detach(),
        };

        model
            .graph
            .sender()
            .send(GraphMsg::PushPoints(vec![
                5.1, 10.9, 12.0, 6.0, 3.0, 3.0, 4.0, 5.0, 9.0, 7.0, 1.0, 1.0, 2.5, 6.8, 6.6, 15.5,
                17.1, 0.9, 6.6,
            ]))
            .unwrap();

        model.scheduled.guard().push_back(DownloadsRowInit::new(
            String::from("/path/to/card.jpg"),
            "Honkai Impact 3rd",
            "69.0.1",
            "China",
            6868696990,
            false,
        ));
        model.scheduled.guard().push_back(DownloadsRowInit::new(
            String::from("/path/to/card.jpg"),
            "Honkai Impact 3rd",
            "420.amogus-rc12",
            "Global",
            6969696969,
            false,
        ));

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {}
    }
}
