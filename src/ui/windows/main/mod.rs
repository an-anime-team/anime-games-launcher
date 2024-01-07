use std::path::PathBuf;
use std::collections::{HashMap, HashSet};

use relm4::prelude::*;
use relm4::factory::*;

use gtk::prelude::*;
use adw::prelude::*;

use crate::{
    config,
    games
};

use crate::components::wine::*;
use crate::components::dxvk::*;

use crate::config::games::GameEditionAddon;

use crate::ui::windows::preferences::PreferencesApp;

use crate::ui::windows::game_addons_manager::{
    GameAddonsManagerApp,
    GameAddonsManagerAppMsg
};

use crate::ui::components::game_card::{
    CardInfo,
    CardComponentInput,
    CardComponentOutput
};

use crate::ui::components::factory::game_card_main::CardFactory;

use crate::ui::components::game_details::{
    GameDetailsComponent,
    GameDetailsComponentInput,
    GameDetailsComponentOutput
};

use crate::ui::components::tasks_queue::{
    TasksQueueComponent,
    TasksQueueComponentInput,
    TasksQueueComponentOutput,

    create_prefix_task::CreatePrefixQueuedTask
};

use super::loading::load_app::LoadingResult;

pub mod launch_game;
pub mod download_game_task;

static mut WINDOW: Option<adw::ApplicationWindow> = None;
static mut PREFERENCES_APP: Option<AsyncController<PreferencesApp>> = None;
static mut GAME_ADDONS_MANAGER_APP: Option<AsyncController<GameAddonsManagerApp>> = None;

pub struct MainApp {
    leaflet: adw::Leaflet,
    flap: adw::Flap,

    main_toast_overlay: adw::ToastOverlay,
    game_details_toast_overlay: adw::ToastOverlay,

    game_details: AsyncController<GameDetailsComponent>,
    game_details_info: CardInfo,

    running_games: FactoryVecDeque<CardFactory>,
    installed_games: FactoryVecDeque<CardFactory>,
    queued_games: FactoryVecDeque<CardFactory>,
    available_games: FactoryVecDeque<CardFactory>,

    running_games_indexes: HashMap<CardInfo, DynamicIndex>,
    installed_games_indexes: HashMap<CardInfo, DynamicIndex>,
    queued_games_indexes: HashMap<CardInfo, DynamicIndex>,
    available_games_indexes: HashMap<CardInfo, DynamicIndex>,

    tasks_queue: AsyncController<TasksQueueComponent>
}

#[derive(Debug)]
pub enum MainAppMsg {
    OpenDetails {
        info: CardInfo,
        installed: bool
    },

    HideDetails,

    OpenPreferences,
    OpenAddonsManager(CardInfo),

    SetEnabledAddons {
        game: CardInfo,
        addons: HashSet<GameEditionAddon>
    },

    ShowTasksFlap,
    HideTasksFlap,
    ToggleTasksFlap,

    AddDownloadGameTask(CardInfo),
    AddVerifyGameTask(CardInfo),
    FinishQueuedTask(CardInfo),

    AddDownloadWineTask {
        name: String,
        title: String,
        developer: String,
        version: Wine
    },

    AddDownloadDxvkTask {
        name: String,
        title: String,
        developer: String,
        version: Dxvk
    },

    AddCreatePrefixTask {
        path: PathBuf,
        install_corefonts: bool
    },

    LaunchGame(CardInfo),
    FinishRunningGame(CardInfo),

    ShowToast {
        title: String,
        message: Option<String>
    }
}

#[relm4::component(pub)]
impl SimpleComponent for MainApp {
    type Init = LoadingResult;
    type Input = MainAppMsg;
    type Output = ();

    view! {
        window = adw::ApplicationWindow {
            set_default_size: (1200, 800),
            set_title: Some("Anime Games Launcher"),

            #[local_ref]
            leaflet -> adw::Leaflet {
                set_can_unfold: false,

                #[local_ref]
                append = main_toast_overlay -> adw::ToastOverlay {
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        adw::HeaderBar {
                            add_css_class: "flat",

                            pack_start = &gtk::Button {
                                set_icon_name: "view-dual-symbolic",

                                connect_clicked => MainAppMsg::ToggleTasksFlap
                            },

                            pack_end = &gtk::Button {
                                set_icon_name: "emblem-system-symbolic",

                                connect_clicked => MainAppMsg::OpenPreferences
                            }
                        },

                        #[local_ref]
                        flap -> adw::Flap {
                            set_fold_policy: adw::FlapFoldPolicy::Always,
                            // set_transition_type: adw::FlapTransitionType::Slide,

                            // set_modal: false,

                            #[wrap(Some)]
                            set_flap = &adw::Clamp {
                                add_css_class: "background",

                                set_maximum_size: 240,
                                set_tightening_threshold: 400,

                                model.tasks_queue.widget(),
                            },

                            #[wrap(Some)]
                            set_separator = &gtk::Separator,

                            #[wrap(Some)]
                            set_content = &gtk::ScrolledWindow {
                                set_hexpand: true,
                                set_vexpand: true,
                                
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,

                                    gtk::Label {
                                        set_halign: gtk::Align::Start,

                                        set_margin_start: 24,
                                        add_css_class: "title-4",

                                        #[watch]
                                        set_visible: !model.running_games.is_empty(),

                                        set_label: "Running games"
                                    },

                                    #[local_ref]
                                    running_games_flow_box -> gtk::FlowBox {
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

                                        #[watch]
                                        set_visible: !model.installed_games.is_empty(),

                                        set_label: "Installed games"
                                    },

                                    #[local_ref]
                                    installed_games_flow_box -> gtk::FlowBox {
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

                                        #[watch]
                                        set_visible: !model.queued_games.is_empty(),

                                        set_label: "Queued games"
                                    },

                                    #[local_ref]
                                    queued_games_flow_box -> gtk::FlowBox {
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

                                        #[watch]
                                        set_visible: !model.available_games.is_empty(),

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
                },

                #[local_ref]
                append = game_details_toast_overlay -> adw::ToastOverlay {
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        // #[watch]
                        // set_css_classes: &[
                        //     model.game_details_info.get_details_style()
                        // ],

                        adw::HeaderBar {
                            add_css_class: "flat",

                            pack_start = &gtk::Button {
                                set_icon_name: "go-previous-symbolic",

                                connect_clicked => MainAppMsg::HideDetails
                            }
                        },

                        model.game_details.widget(),
                    }
                }
            }
        }
    }

    fn init(init: Self::Init, root: &Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let mut model = Self {
            leaflet: adw::Leaflet::new(),
            flap: adw::Flap::new(),

            main_toast_overlay: adw::ToastOverlay::new(),
            game_details_toast_overlay: adw::ToastOverlay::new(),

            game_details: GameDetailsComponent::builder()
                .launch(CardInfo::default())
                .forward(sender.input_sender(), |message| match message {
                    GameDetailsComponentOutput::HideDetails => MainAppMsg::HideDetails,
                    GameDetailsComponentOutput::ShowTasksFlap => MainAppMsg::ShowTasksFlap,

                    GameDetailsComponentOutput::DownloadGame(info)
                        => MainAppMsg::AddDownloadGameTask(info),

                    GameDetailsComponentOutput::VerifyGame(info)
                        => MainAppMsg::AddVerifyGameTask(info),

                    GameDetailsComponentOutput::LaunchGame(info)
                        => MainAppMsg::LaunchGame(info),

                    GameDetailsComponentOutput::OpenAddonsManager(info)
                        => MainAppMsg::OpenAddonsManager(info),

                    GameDetailsComponentOutput::ShowToast { title, message }
                        => MainAppMsg::ShowToast { title, message }
                }),

            game_details_info: CardInfo::default(),

            running_games_indexes: HashMap::new(),
            installed_games_indexes: HashMap::new(),
            queued_games_indexes: HashMap::new(),
            available_games_indexes: HashMap::new(),

            running_games: FactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |output: CardComponentOutput| -> MainAppMsg {
                    match output {
                        CardComponentOutput::CardClicked { info, installed }
                            => MainAppMsg::OpenDetails { info, installed }
                    }
                }),

            installed_games: FactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |output: CardComponentOutput| -> MainAppMsg {
                    match output {
                        CardComponentOutput::CardClicked { info, installed }
                            => MainAppMsg::OpenDetails { info, installed }
                    }
                }),

            queued_games: FactoryVecDeque::builder()
                .launch_default()
                .detach(),
                // .forward(sender.input_sender(), |output: CardComponentOutput| -> MainAppMsg {
                //     match output {
                //         CardComponentOutput::CardClicked { info, installed }
                //             => MainAppMsg::OpenDetails { info, installed }
                //     }
                // }),

            available_games: FactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |output: CardComponentOutput| -> MainAppMsg {
                    match output {
                        CardComponentOutput::CardClicked { info, installed }
                            => MainAppMsg::OpenDetails { info, installed }
                    }
                }),

            tasks_queue: TasksQueueComponent::builder()
                .launch(CardInfo::default())
                .forward(sender.input_sender(), |output| match output {
                    TasksQueueComponentOutput::TaskFinished(info)
                        => MainAppMsg::FinishQueuedTask(info),

                    TasksQueueComponentOutput::HideTasksFlap
                        => MainAppMsg::HideTasksFlap,

                    TasksQueueComponentOutput::ShowToast { title, message }
                        => MainAppMsg::ShowToast { title, message }
                }),
        };

        for game in init.games_list.available {
            let card = CardInfo::Game {
                name: game.game_name.clone(),
                title: game.game_title.clone(),
                developer: game.game_developer.clone(),
                edition: game.edition.name.clone(),
                picture_uri: game.card_picture.clone()
            };

            model.available_games_indexes.insert(
                card.to_owned(),
                model.available_games.guard().push_back(card.to_owned())
            );
        }

        for game in init.games_list.installed {
            let card = CardInfo::Game {
                name: game.game_name.clone(),
                title: game.game_title.clone(),
                developer: game.game_developer.clone(),
                edition: game.edition.name.clone(),
                picture_uri: game.card_picture.clone()
            };

            model.installed_games_indexes.insert(
                card.to_owned(),
                model.installed_games.guard().push_back(card.to_owned())
            );
        }

        model.available_games.broadcast(CardComponentInput::SetInstalled(false));

        let leaflet = &model.leaflet;
        let flap = &model.flap;

        let main_toast_overlay = &model.main_toast_overlay;
        let game_details_toast_overlay = &model.game_details_toast_overlay;

        let running_games_flow_box = model.running_games.widget();
        let installed_games_flow_box = model.installed_games.widget();
        let queued_games_flow_box = model.queued_games.widget();
        let available_games_flow_box = model.available_games.widget();

        let widgets = view_output!();

        unsafe {
            WINDOW = Some(widgets.window.clone());

            PREFERENCES_APP = Some(PreferencesApp::builder()
                .launch(widgets.window.clone())
                .detach());

            GAME_ADDONS_MANAGER_APP = Some(GameAddonsManagerApp::builder()
                .launch(widgets.window.clone())
                .forward(sender.input_sender(), std::convert::identity));
        }

        if let Some(wine) = init.download_wine {
            sender.input(MainAppMsg::AddDownloadWineTask {
                name: wine.name.clone(),
                title: wine.title.clone(),
                developer: String::new(),
                version: wine
            });

            sender.input(MainAppMsg::ShowTasksFlap);
        }

        if let Some(dxvk) = init.download_dxvk {
            sender.input(MainAppMsg::AddDownloadDxvkTask {
                name: dxvk.name.clone(),
                title: dxvk.name.clone(), // name > title in case of dxvks
                developer: String::new(),
                version: dxvk
            });

            sender.input(MainAppMsg::ShowTasksFlap);
        }

        if let Some(prefix) = init.create_prefix {
            sender.input(MainAppMsg::AddCreatePrefixTask {
                path: prefix.path.clone(),
                install_corefonts: prefix.install_corefonts
            });

            sender.input(MainAppMsg::ShowTasksFlap);
        }

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            MainAppMsg::OpenDetails { info, installed } => {
                self.game_details_info = info.clone();

                self.game_details.emit(GameDetailsComponentInput::SetInfo(info));
                self.game_details.emit(GameDetailsComponentInput::SetInstalled(installed));

                self.leaflet.navigate(adw::NavigationDirection::Forward);
            }

            MainAppMsg::HideDetails => {
                self.leaflet.navigate(adw::NavigationDirection::Back);
            }

            MainAppMsg::OpenPreferences => unsafe {
                PREFERENCES_APP.as_ref()
                    .unwrap_unchecked()
                    .widget()
                    .present();
            }

            MainAppMsg::OpenAddonsManager(game_info) => unsafe {
                let controller = GAME_ADDONS_MANAGER_APP.as_ref()
                    .unwrap_unchecked();

                let game = games::get_unsafe(game_info.get_name());

                match game.get_addons_list(game_info.get_edition()) {
                    Ok(addons) => {
                        controller.emit(GameAddonsManagerAppMsg::SetGameInfo {
                            game_info,
                            addons
                        });

                        controller.widget().present();
                    }

                    Err(err) => {
                        sender.input(MainAppMsg::ShowToast {
                            title: format!("Unable to get {} addons list", game_info.get_title()),
                            message: Some(err.to_string())
                        });
                    }
                }
            }

            // FIXME: doesn't look really safe
            MainAppMsg::SetEnabledAddons { game, addons } => {
                let property = format!("games.settings.{}.addons.{}", game.get_name(), game.get_edition());
                let value = serde_json::to_value(&addons).unwrap();

                config::set(property, value).unwrap();


            }

            MainAppMsg::ShowTasksFlap => {
                self.flap.set_reveal_flap(true);
            }

            MainAppMsg::HideTasksFlap => {
                self.flap.set_reveal_flap(false);
            }

            MainAppMsg::ToggleTasksFlap => {
                self.flap.set_reveal_flap(!self.flap.reveals_flap());
            }

            MainAppMsg::AddDownloadGameTask(info) => {
                let config = config::get();

                match download_game_task::get_download_game_task(&info, &config) {
                    Ok(task) => {
                        self.tasks_queue.emit(TasksQueueComponentInput::AddTask(task));

                        if let Some(index) = self.available_games_indexes.get(&info) {
                            self.available_games.guard().remove(index.current_index());

                            self.available_games_indexes.remove(&info);

                            #[allow(clippy::map_entry)]
                            if !self.queued_games_indexes.contains_key(&info) {
                                self.queued_games_indexes.insert(info.clone(), self.queued_games.guard().push_back(info.clone()));

                                self.queued_games.broadcast(CardComponentInput::SetInstalled(false));
                                self.queued_games.broadcast(CardComponentInput::SetClickable(false));
                            }
                        }

                        if config.general.verify_games {
                            sender.input(MainAppMsg::AddVerifyGameTask(info));
                        }
                    }

                    Err(err) => sender.input(*err)
                }
            }

            MainAppMsg::AddVerifyGameTask(info) => {
                // let task = Box::new(VerifyIntegrityQueuedTask {
                //     info: info.clone()
                // });

                // self.tasks_queue.emit(TasksQueueComponentInput::AddTask(task));

                // if let Some(index) = self.available_games_indexes.get(&info) {
                //     self.available_games.guard().remove(index.current_index());

                //     self.available_games_indexes.remove(&info);

                //     #[allow(clippy::map_entry)]
                //     if !self.queued_games_indexes.contains_key(&info) {
                //         self.queued_games_indexes.insert(info.clone(), self.queued_games.guard().push_back(info));

                //         self.queued_games.broadcast(CardComponentInput::SetInstalled(false));
                //         self.queued_games.broadcast(CardComponentInput::SetClickable(false));
                //     }
                // }
            }

            MainAppMsg::FinishQueuedTask(info) => {
                if let Some(index) = self.queued_games_indexes.get(&info) {
                    self.queued_games.guard().remove(index.current_index());

                    self.queued_games_indexes.remove(&info);

                    #[allow(clippy::map_entry)]
                    if !self.installed_games_indexes.contains_key(&info) {
                        self.installed_games_indexes.insert(info.clone(), self.installed_games.guard().push_back(info));
                    }
                }
            }

            MainAppMsg::AddDownloadWineTask { name, title, developer, version } => {
                self.tasks_queue.emit(TasksQueueComponentInput::AddTask(Box::new(DownloadWineQueuedTask {
                    name,
                    title,
                    developer,
                    version
                })));
            }

            MainAppMsg::AddDownloadDxvkTask { name, title, developer, version } => {
                self.tasks_queue.emit(TasksQueueComponentInput::AddTask(Box::new(DownloadDxvkQueuedTask {
                    name,
                    title,
                    developer,
                    version
                })));
            }

            MainAppMsg::AddCreatePrefixTask { path, install_corefonts } => {
                self.tasks_queue.emit(TasksQueueComponentInput::AddTask(Box::new(CreatePrefixQueuedTask {
                    path,
                    install_corefonts
                })));
            }

            MainAppMsg::LaunchGame(info) => {
                if let Some(index) = self.installed_games_indexes.get(&info) {
                    self.installed_games.guard().remove(index.current_index());

                    self.installed_games_indexes.remove(&info);

                    #[allow(clippy::map_entry)]
                    if !self.running_games_indexes.contains_key(&info) {
                        self.running_games_indexes.insert(info.clone(), self.running_games.guard().push_back(info.clone()));
                    }

                    std::thread::spawn(move || {
                        if let Err(err) = launch_game::launch_game(&info) {
                            sender.input(MainAppMsg::ShowToast {
                                title: format!("Failed to launch {}", info.get_title()),
                                message: Some(err.to_string())
                            });
                        }

                        sender.input(MainAppMsg::FinishRunningGame(info));
                    });
                }
            }

            MainAppMsg::FinishRunningGame(info) => {
                if let Some(index) = self.running_games_indexes.get(&info) {
                    self.running_games.guard().remove(index.current_index());

                    self.running_games_indexes.remove(&info);

                    #[allow(clippy::map_entry)]
                    if !self.installed_games_indexes.contains_key(&info) {
                        self.installed_games_indexes.insert(info.clone(), self.installed_games.guard().push_back(info));
                    }
                }
            }

            MainAppMsg::ShowToast { title, message } => {
                let window = unsafe {
                    WINDOW.as_ref().unwrap_unchecked()
                };

                let toast = adw::Toast::new(&title);

                // toast.set_timeout(7);

                if let Some(message) = message {
                    toast.set_button_label(Some("Details"));

                    let dialog = adw::MessageDialog::new(
                        Some(window),
                        Some(&title),
                        Some(&message)
                    );

                    dialog.add_response("close", "Close");
                    // dialog.add_response("save", &tr!("save"));

                    // dialog.set_response_appearance("save", adw::ResponseAppearance::Suggested);

                    // dialog.connect_response(Some("save"), |_, _| {
                    //     if let Err(err) = open::that(crate::DEBUG_FILE.as_os_str()) {
                    //         tracing::error!("Failed to open debug file: {err}");
                    //     }
                    // });

                    toast.connect_button_clicked(move |_| {
                        dialog.present();
                    });
                }

                self.main_toast_overlay.add_toast(toast);
            }
        }
    }
}
