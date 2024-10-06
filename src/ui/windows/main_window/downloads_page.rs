use adw::prelude::*;
use gtk::prelude::*;

use relm4::factory::*;
use relm4::prelude::*;

use crate::ui::components::downloads_row::{
    DownloadsRow, DownloadsRowFactory, DownloadsRowFactoryOutput, DownloadsRowInit,
};
use crate::ui::components::graph::{Graph, GraphInit, GraphMsg, GraphOutput};
use crate::utils::{pretty_bytes, pretty_seconds};

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

    // Graph
    pub speed: u64,
    pub avg_speed: u64,
    pub total: u64,
    pub elapsed: u64,
}

#[derive(Debug, Clone)]
pub enum DownloadsPageAppMsg {
    SetActive(DynamicIndex),
    SetNone,
    SetDownloading,
    SetExtracting,
    SetStreamUnpacking,
    SetVerifying,

    // Graph
    PushGraph(u64),
    UpdateMean(u64),
}

#[derive(Debug)]
pub enum DownloadsPageAppOutput {}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for DownloadsPageApp {
    type Init = ();
    type Input = DownloadsPageAppMsg;
    type Output = DownloadsPageAppOutput;

    view! {
        #[root]
        adw::PreferencesPage {
            adw::PreferencesGroup {
                model.graph.widget(),
            },

            adw::PreferencesGroup {
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
                    set_spacing: 16,
                    adw::PreferencesGroup {
                        adw::ActionRow {
                            set_title: "Current speed",
                            #[watch]
                            set_subtitle: &format!("{}/s", pretty_bytes(model.speed)),
                        }
                    },
                    adw::PreferencesGroup {
                        adw::ActionRow {
                            set_title: "Average speed",
                            #[watch]
                            set_subtitle: &format!("{}/s", pretty_bytes(model.avg_speed)),
                        }
                    },
                    adw::PreferencesGroup {
                        adw::ActionRow {
                            set_title: "Time elapsed",
                            #[watch]
                            set_subtitle: &pretty_seconds(model.elapsed),
                        }
                    },
                    adw::PreferencesGroup {
                        adw::ActionRow {
                            set_title: "Current ETA",
                            set_subtitle: "amogus",
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
                            #[watch]
                            set_subtitle: &pretty_bytes(model.total),
                        }
                    },
                }
            },

            adw::PreferencesGroup {
                set_title: "Active",
                model.active.widget(),
            },

            model.scheduled.widget() {
                set_title: "Scheduled",
            },
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let TEST_PATH = String::from("card.jpg");

        let mut model = Self {
            graph: Graph::builder()
                .launch(GraphInit::new(800, 150, (1.0, 1.0, 1.0)))
                .forward(sender.input_sender(), |msg| match msg {
                    GraphOutput::UpdateMean(mean) => DownloadsPageAppMsg::UpdateMean(mean),
                }),
            active: DownloadsRow::builder()
                .launch(DownloadsRowInit::new(
                    TEST_PATH.clone(),
                    String::from("Genshin Impact"),
                    String::from("5.0.0"),
                    String::from("Global"),
                    64500000000,
                    true,
                ))
                .detach(),
            scheduled: AsyncFactoryVecDeque::builder().launch_default().forward(
                sender.input_sender(),
                |msg| match msg {
                    DownloadsRowFactoryOutput::Queue(index) => {
                        DownloadsPageAppMsg::SetActive(index)
                    }
                },
            ),
            state: DownloadsAppState::None,
            speed: 0,
            avg_speed: 0,
            total: 0,
            elapsed: 0,
        };

        model
            .graph
            .sender()
            .send(GraphMsg::PushPoints(vec![
                452904458, 777220094, 526592924, 426636796, 55706776, 466556367, 627206652,
                569798866, 787685971, 844515495, 455001528, 573125330, 281186686, 270649229,
                269895417, 738853220, 808851483, 284943234, 394286493, 102257847, 298817512,
                134789158, 411591134, 570248522, 376836031, 733191506, 647243898, 20064546,
                71899336, 308387460, 426337509, 954825850, 582178251, 477989262, 526292385,
                985058876, 591576225, 741071822, 835657398, 642072403, 203395692, 36618218,
                430474183, 118050934, 320472918, 980503848, 261719052, 869404642, 548574685,
                123899716, 708913548, 68543217, 26470188, 582797287, 597725069, 114298253,
                538887098, 691888414, 692313962, 650691223, 366081763, 951507556, 740370818,
                889678510, 467961836, 306535248, 338989600, 247635488, 382553911, 141272676,
                202169419, 793615757, 727655586, 814807597, 343162717, 636274439, 980003775,
                462723132, 60623157, 368616204, 68203759, 812854107, 816411328, 176272232,
                133554123, 46871030, 979449449, 955908633, 231917883, 864525004, 361324818,
                764085453, 688441572, 15607384, 417882545, 729230332, 930415001, 732634987,
                398998026, 542747212, 710945136, 163321328, 246550246, 412162156, 941365870,
                969007376, 499733967, 579997158, 235054989, 376817421, 363769278, 884677738,
                407359888, 496772160, 572000509, 606011370, 320182380, 453670032, 980451841,
                623097551, 446886241, 875198569, 892212680, 741131490, 269979849, 888089509,
                344921416, 169832084, 937979311, 495754357, 941159130, 416680453, 172892222,
                569899913, 600207900, 818665275, 620662303, 244007114, 274570744, 884232940,
                845198408, 901314588, 3480684, 758698412, 522948809, 666529434, 822910389,
                817039622, 95285823, 579982733, 978308146, 941480666, 877187712, 53513477,
                28232160, 926517104, 325215439, 186264697, 796805569, 954038814, 252809913,
                368305204, 91573137, 937745378, 518250698, 933410773, 745392663, 489911761,
                760767021, 159508754, 866679635, 389648061, 760131838, 547049442, 238544489,
                864558969, 407592895, 707924883, 261370446, 801808253, 632801191, 387502075,
                571185268, 318911598, 546977930, 697134392, 379699234, 670421531, 586621788,
                239806623, 531908386, 652938349, 879474224, 631865484, 331509897, 594835618,
                699535467, 163176811, 86562844, 104910415,
            ]))
            .unwrap();

        model.scheduled.guard().push_back(DownloadsRowInit::new(
            TEST_PATH.clone(),
            String::from("Honkai Impact 3rd"),
            String::from("69.0.1"),
            String::from("China"),
            6868696990,
            false,
        ));
        model.scheduled.guard().push_back(DownloadsRowInit::new(
            TEST_PATH.clone(),
            String::from("Honkai Impact 3rd"),
            String::from("420.amogus-rc12"),
            String::from("Global"),
            6969696969,
            false,
        ));

        sender.input(DownloadsPageAppMsg::SetVerifying);
        sender.input(DownloadsPageAppMsg::PushGraph(879474224));

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>) {
        // https://developer.gnome.org/hig/reference/palette.html
        // https://rgbcolorpicker.com/0-1
        match msg {
            DownloadsPageAppMsg::SetActive(index) => {
                println!("Selected {}", index.current_index());
            }
            DownloadsPageAppMsg::SetNone => {
                self.graph
                    .sender()
                    .send(GraphMsg::SetColor((1.0, 1.0, 1.0)))
                    .unwrap();
                self.state = DownloadsAppState::None;
            }
            DownloadsPageAppMsg::SetDownloading => {
                self.graph
                    .sender()
                    .send(GraphMsg::SetColor((0.101, 0.373, 0.706))) // Blue 5
                    .unwrap();
                self.state = DownloadsAppState::Downloading;
            }
            DownloadsPageAppMsg::SetExtracting => {
                self.graph
                    .sender()
                    .send(GraphMsg::SetColor((0.149, 0.635, 0.412))) // Green 5
                    .unwrap();
                self.state = DownloadsAppState::Extracting;
            }
            DownloadsPageAppMsg::SetStreamUnpacking => {
                self.graph
                    .sender()
                    .send(GraphMsg::SetColor((0.647, 0.114, 0.176))) // Red 5
                    .unwrap();
                self.state = DownloadsAppState::StreamUnpacking;
            }
            DownloadsPageAppMsg::SetVerifying => {
                self.graph
                    .sender()
                    .send(GraphMsg::SetColor((0.976, 0.941, 0.42))) // Yellow 1
                    .unwrap();
                self.state = DownloadsAppState::Verifying;
            }
            DownloadsPageAppMsg::PushGraph(point) => {
                self.graph
                    .sender()
                    .send(GraphMsg::PushPoint(point))
                    .unwrap();
                self.speed = point;
            }
            DownloadsPageAppMsg::UpdateMean(mean) => {
                self.avg_speed = mean;
            }
        }
    }
}
