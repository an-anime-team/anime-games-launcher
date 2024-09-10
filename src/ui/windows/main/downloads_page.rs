use adw::prelude::*;
use gtk::prelude::*;

use relm4::factory::*;
use relm4::prelude::*;

use crate::ui::components::downloads_row::{
    DownloadsRow, DownloadsRowFactory, DownloadsRowFactoryOutput, DownloadsRowInit,
};
use crate::ui::components::graph::{Graph, GraphInit, GraphMsg};

#[derive(Debug)]
pub enum DownloadsAppState {
    None,
    Downloading,
    StreamUnpacking,
    Extracting,
    Verifying,
}

#[derive(Debug)]
pub struct DownloadsPageApp {
    pub graph: AsyncController<Graph>,
    pub active: AsyncController<DownloadsRow>,
    pub scheduled: AsyncFactoryVecDeque<DownloadsRowFactory>,
    pub state: DownloadsAppState,
}

#[derive(Debug, Clone)]
pub enum DownloadsPageAppMsg {
    SetNone,
    StartDownloading,
    StartExtracting,
    StartStreamUnpacking,
    StartVerifying,
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for DownloadsPageApp {
    type Init = ();
    type Input = DownloadsPageAppMsg;
    type Output = ();

    view! {
        #[root]
        adw::PreferencesPage {
            // A bit more space before graph
            add = &adw::PreferencesGroup {
                gtk::Box {
                    set_height_request: 16,
                }
            },

            add = &adw::PreferencesGroup {
                model.graph.widget(),
            },

            add = &adw::PreferencesGroup {
                #[watch]
                set_visible: match model.state {
                    DownloadsAppState::None => false,
                    _ => true,
                },
                #[watch]
                set_title: match model.state {
                    DownloadsAppState::None => "",
                    DownloadsAppState::Downloading => "Downloading",
                    DownloadsAppState::Extracting => "Extracting",
                    DownloadsAppState::StreamUnpacking => "Stream unpacking",
                    DownloadsAppState::Verifying => "Verifying",
                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,
                    set_halign: gtk::Align::Fill,
                    set_spacing: 16,
                    adw::PreferencesGroup {
                        adw::ActionRow {
                            set_title: "Current speed",
                            set_subtitle: "2.50 MB/s",
                        }
                    },
                    adw::PreferencesGroup {
                        adw::ActionRow {
                            set_title: "Average speed",
                            set_subtitle: "2.20 MB/s",
                        }
                    },
                    adw::PreferencesGroup {
                        adw::ActionRow {
                            set_title: "Time elapsed",
                            set_subtitle: "02:12",
                        }
                    },
                    adw::PreferencesGroup {
                        adw::ActionRow {
                            set_title: "Current ETA",
                            set_subtitle: "12 hours",
                        }
                    },
                    adw::PreferencesGroup {
                        adw::ActionRow {
                            #[watch]
                            set_title: match model.state {
                                DownloadsAppState::None => "",
                                DownloadsAppState::Downloading => "Total download",
                                DownloadsAppState::Extracting => "Total extracted",
                                DownloadsAppState::StreamUnpacking => "Total unpacked",
                                DownloadsAppState::Verifying => "Total verified",
                            },
                            set_subtitle: "69.42 GB",
                        }
                    },
                }
            },

            add = &adw::PreferencesGroup {
                set_title: "Active",
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
            state: DownloadsAppState::None,
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

        sender.input(DownloadsPageAppMsg::SetNone);

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            DownloadsPageAppMsg::SetNone => {
                self.graph
                    .sender()
                    .send(GraphMsg::SetColor((1.0, 1.0, 1.0)))
                    .unwrap();
                self.state = DownloadsAppState::None;
            }
            DownloadsPageAppMsg::StartDownloading => {
                self.graph
                    .sender()
                    .send(GraphMsg::SetColor((0.0, 0.0, 1.0)))
                    .unwrap();
                self.state = DownloadsAppState::Downloading;
            }
            DownloadsPageAppMsg::StartExtracting => {
                self.graph
                    .sender()
                    .send(GraphMsg::SetColor((0.0, 1.0, 0.0)))
                    .unwrap();
                self.state = DownloadsAppState::Extracting;
            }
            DownloadsPageAppMsg::StartStreamUnpacking => {
                self.graph
                    .sender()
                    .send(GraphMsg::SetColor((1.0, 0.0, 0.0)))
                    .unwrap();
                self.state = DownloadsAppState::StreamUnpacking;
            }
            DownloadsPageAppMsg::StartVerifying => {
                self.graph
                    .sender()
                    .send(GraphMsg::SetColor((1.0, 1.0, 0.0)))
                    .unwrap();
                self.state = DownloadsAppState::Verifying;
            }
        }
    }
}
