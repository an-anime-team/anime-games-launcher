use relm4::prelude::*;
use relm4::component::*;
use relm4::factory::*;

use gtk::glib::clone;

use gtk::prelude::*;
use adw::prelude::*;

use crate::components::game_card::{
    GameCardComponent,
    GameCardFactory,
    GameCardVariant,
    GameCardComponentMsg
};

pub struct MainApp {
    toast_overlay: adw::ToastOverlay,

    installed_games: FactoryVecDeque<GameCardFactory>,
    available_games: FactoryVecDeque<GameCardFactory>,

    downloading_game: AsyncController<GameCardComponent>
}

#[derive(Debug, Clone)]
pub enum MainAppMsg {

}

#[relm4::component(pub)]
impl SimpleComponent for MainApp {
    type Init = ();
    type Input = MainAppMsg;
    type Output = ();

    view! {
        window = adw::ApplicationWindow {
            set_default_size: (900, 600),
            set_title: Some("Anime Games Launcher"),

            #[local_ref]
            toast_overlay -> adw::ToastOverlay {
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    adw::HeaderBar {
                        add_css_class: "flat",

                        pack_start = &gtk::ToggleButton {
                            set_icon_name: "view-dual-symbolic",

                            #[chain(build())]
                            bind_property: ("active", &flap, "reveal-flap"),
                        }
                    },

                    gtk::ScrolledWindow {
                        set_hexpand: true,
                        set_vexpand: true,

                        #[name(flap)]
                        adw::Flap {
                            set_fold_policy: adw::FlapFoldPolicy::Always,

                            #[wrap(Some)]
                            set_flap = &gtk::Box {
                                add_css_class: "background",

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,

                                    set_margin_start: 24,
                                    set_margin_end: 24,

                                    model.downloading_game.widget(),

                                    gtk::Label {
                                        set_halign: gtk::Align::Start,

                                        set_margin_top: 24,

                                        add_css_class: "title-4",

                                        set_label: "Downloading Honkai: Star Rail..."
                                    },
    
                                    gtk::ProgressBar {
                                        set_margin_top: 16,
                                        set_fraction: 0.7
                                    },

                                    gtk::Label {
                                        set_halign: gtk::Align::Start,

                                        set_margin_top: 16,

                                        set_label: "Download speed: 20 MB/s"
                                    },

                                    gtk::Label {
                                        set_halign: gtk::Align::Start,

                                        set_margin_top: 8,

                                        set_label: "ETA: 7 minutes"
                                    }
                                }
                            },

                            #[wrap(Some)]
                            set_content = &gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,

                                gtk::Label {
                                    set_halign: gtk::Align::Start,

                                    set_margin_start: 24,
                                    add_css_class: "title-4",

                                    set_label: "Installed games"
                                },

                                #[local_ref]
                                installed_games_flow_box ->gtk::FlowBox {
                                    set_row_spacing: 12,
                                    set_column_spacing: 12,

                                    set_margin_all: 16,

                                    set_homogeneous: true,
                                    set_selection_mode: gtk::SelectionMode::None
                                },

                                gtk::Label {
                                    set_halign: gtk::Align::Start,

                                    set_margin_start: 24,
                                    add_css_class: "title-4",

                                    set_label: "Available games"
                                },

                                #[local_ref]
                                available_games_flow_box -> gtk::FlowBox {
                                    set_row_spacing: 12,
                                    set_column_spacing: 12,

                                    set_margin_all: 16,

                                    set_homogeneous: true,
                                    set_selection_mode: gtk::SelectionMode::None
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn init(
        _parent: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = Self {
            toast_overlay: adw::ToastOverlay::new(),

            installed_games: FactoryVecDeque::new(gtk::FlowBox::new(), sender.input_sender()),
            available_games: FactoryVecDeque::new(gtk::FlowBox::new(), sender.input_sender()),

            downloading_game: GameCardComponent::builder()
                .launch(GameCardVariant::Genshin)
                .detach(),

            // installed_games: vec![
            //     GameCardComponent::builder()
            //         .launch(GameCardVariant::Genshin)
            //         .detach(),

            //     GameCardComponent::builder()
            //         .launch(GameCardVariant::Honkai)
            //         .detach()
            // ],

            // available_games: vec![
            //     GameCardComponent::builder()
            //         .launch(GameCardVariant::StarRail)
            //         .detach()
            // ]
        };

        model.downloading_game.emit(GameCardComponentMsg::SetWidth(160));
        model.downloading_game.emit(GameCardComponentMsg::SetHeight(224));

        model.installed_games.guard().push_back(GameCardVariant::Genshin);
        model.installed_games.guard().push_back(GameCardVariant::Honkai);

        model.available_games.guard().push_back(GameCardVariant::StarRail);

        model.available_games.broadcast(GameCardComponentMsg::SetInstalled(false));

        let toast_overlay = &model.toast_overlay;

        let installed_games_flow_box = model.installed_games.widget();
        let available_games_flow_box = model.available_games.widget();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            
        }
    }
}