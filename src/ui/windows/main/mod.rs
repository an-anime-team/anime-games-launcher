use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::Arc;

use relm4::prelude::*;
use relm4::factory::*;

use gtk::prelude::*;
use adw::prelude::*;

use anime_game_core::filesystem::DriverExt;

use crate::{
    config,
    games,
    STARTUP_CONFIG
};

use crate::components::wine::*;
use crate::components::dxvk::*;

use crate::ui::windows::preferences::PreferencesApp;

use crate::ui::windows::game_dlcs::{
    GameDlcsApp,
    GameDlcsAppMsg
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

    download_diff_task::DownloadDiffQueuedTask,
    create_prefix_task::CreatePrefixQueuedTask,
    verify_integrity_task::VerifyIntegrityQueuedTask
};

pub mod launch_game;

static mut WINDOW: Option<adw::ApplicationWindow> = None;
static mut PREFERENCES_APP: Option<AsyncController<PreferencesApp>> = None;
static mut GAME_DLCS_APP: Option<AsyncController<GameDlcsApp>> = None;

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
    OpenDlcsManager(CardInfo),

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
    type Init = ();
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

    fn init(
        _parent: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
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

                    GameDetailsComponentOutput::OpenDlcsManager(info)
                        => MainAppMsg::OpenDlcsManager(info),

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

        match games::list() {
            Ok(games) => {
                for (name, game) in games {
                    match game.get_game_editions_list() {
                        Ok(editions) => {
                            for edition in editions {
                                let card = CardInfo::Game {
                                    name: game.game_name.clone(),
                                    title: game.game_title.clone(),
                                    developer: game.game_developer.clone(),
                                    edition: edition.name.clone(),

                                    picture_uri: match game.get_card_picture(&edition.name) {
                                        Ok(uri) => uri,
                                        Err(err) => {
                                            sender.input(MainAppMsg::ShowToast {
                                                title: format!("Failed to get card picture for game '{name}'"),
                                                message: Some(err.to_string())
                                            });

                                            continue;
                                        }
                                    }
                                };

                                let game_settings = match STARTUP_CONFIG.games.get_game_settings(name) {
                                    Ok(settings) => settings,
                                    Err(err) => {
                                        sender.input(MainAppMsg::ShowToast {
                                            title: format!("Unable to find {name} settings"),
                                            message: Some(err.to_string())
                                        });

                                        continue;
                                    }
                                };

                                let installed = match game_settings.paths.get(&edition.name) {
                                    Some(driver) => {
                                        let driver = driver.to_dyn_trait();

                                        match driver.deploy() {
                                            Ok(path) => {
                                                match game.is_game_installed(path.to_string_lossy()) {
                                                    Ok(installed) => {
                                                        if let Err(err) = driver.dismantle() {
                                                            sender.input(MainAppMsg::ShowToast {
                                                                title: format!("Failed to deploy folder for game '{name}' with '{}' edition", edition.name),
                                                                message: Some(err.to_string())
                                                            });

                                                            continue;
                                                        }

                                                        installed
                                                    }

                                                    Err(err) => {
                                                        sender.input(MainAppMsg::ShowToast {
                                                            title: format!("Failed to get game '{name}' info with '{}' edition", edition.name),
                                                            message: Some(err.to_string())
                                                        });

                                                        continue;
                                                    }
                                                }
                                            }

                                            Err(err) => {
                                                sender.input(MainAppMsg::ShowToast {
                                                    title: format!("Failed to deploy folder for game '{name}' with '{}' edition", edition.name),
                                                    message: Some(err.to_string())
                                                });

                                                continue;
                                            }
                                        }
                                    }

                                    None => {
                                        sender.input(MainAppMsg::ShowToast {
                                            title: format!("No path given for game '{name}' with '{}' edition", edition.name),
                                            message: None
                                        });

                                        continue;
                                    }
                                };

                                if installed {
                                    model.installed_games_indexes.insert(
                                        card.to_owned(),
                                        model.installed_games.guard().push_back(card.to_owned())
                                    );
                                }

                                else {
                                    model.available_games_indexes.insert(
                                        card.to_owned(),
                                        model.available_games.guard().push_back(card.to_owned())
                                    );
                                }
                            }
                        }

                        Err(err) => {
                            sender.input(MainAppMsg::ShowToast {
                                title: format!("Failed to get {name} editions list"),
                                message: Some(err.to_string())
                            });

                            continue;
                        }
                    }
                }
            }

            Err(err) => {
                sender.input(MainAppMsg::ShowToast {
                    title: String::from("Failed to list games integrations scripts"),
                    message: Some(err.to_string())
                });
            }
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

            GAME_DLCS_APP = Some(GameDlcsApp::builder()
                .launch(widgets.window.clone())
                .detach());
        }

        std::thread::spawn(move || {
            // Update wine component

            match Wine::from_config() {
                Ok(wine) => {
                    if !wine.is_downloaded() {
                        sender.input(MainAppMsg::AddDownloadWineTask {
                            name: wine.name.clone(),
                            title: wine.title.clone(),
                            developer: String::new(),
                            version: wine
                        });

                        sender.input(MainAppMsg::ShowTasksFlap);
                    }
                }

                Err(err) => {
                    sender.input(MainAppMsg::ShowToast {
                        title: String::from("Failed to get wine version"),
                        message: Some(err.to_string())
                    });
                }
            }

            // Update dxvk component

            match Dxvk::from_config() {
                Ok(dxvk) => {
                    if !dxvk.is_downloaded() {
                        sender.input(MainAppMsg::AddDownloadDxvkTask {
                            name: dxvk.name.clone(),
                            title: dxvk.name.clone(), // name > title in case of dxvks
                            developer: String::new(),
                            version: dxvk
                        });

                        sender.input(MainAppMsg::ShowTasksFlap);
                    }
                }

                Err(err) => {
                    sender.input(MainAppMsg::ShowToast {
                        title: String::from("Failed to get dxvk version"),
                        message: Some(err.to_string())
                    });
                }
            }

            // Create wine prefix

            let prefix = &STARTUP_CONFIG.components.wine.prefix;

            if !prefix.path.exists() {
                sender.input(MainAppMsg::AddCreatePrefixTask {
                    path: prefix.path.clone(),
                    install_corefonts: prefix.install_corefonts
                });

                sender.input(MainAppMsg::ShowTasksFlap);
            }
        });

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

            MainAppMsg::OpenDlcsManager(info) => unsafe {
                let controller = GAME_DLCS_APP.as_ref()
                    .unwrap_unchecked();

                match games::get(info.get_name()) {
                    Ok(Some(game)) => {
                        match game.get_dlc_list(info.get_edition()) {
                            Ok(dlcs) => {
                                controller.emit(GameDlcsAppMsg::SetGameInfo {
                                    info,
                                    dlcs
                                });
                
                                controller.widget().present();
                            }

                            Err(err) => {
                                sender.input(MainAppMsg::ShowToast {
                                    title: format!("Unable to get {} DLC list", info.get_title()),
                                    message: Some(err.to_string())
                                });
                            }
                        }
                    }

                    Ok(None) => {
                        sender.input(MainAppMsg::ShowToast {
                            title: format!("Unable to find {} integration script", info.get_title()),
                            message: None
                        });
                    }

                    Err(err) => {
                        sender.input(MainAppMsg::ShowToast {
                            title: format!("Unable to find {} integration script", info.get_title()),
                            message: Some(err.to_string())
                        });
                    }
                }
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

                let settings = match config.games.get_game_settings(info.get_name()) {
                    Ok(settings) => settings,
                    Err(err) => {
                        sender.input(MainAppMsg::ShowToast {
                            title: format!("Unable to find {} settings", info.get_title()),
                            message: Some(err.to_string())
                        });

                        return;
                    }
                };

                let Some(driver) = settings.paths.get(info.get_edition()) else {
                    sender.input(MainAppMsg::ShowToast {
                        title: format!("Unable to find {} installation path", info.get_title()),
                        message: None
                    });

                    return;
                };

                match games::get(info.get_name()) {
                    Ok(Some(game)) => {
                        let driver = Arc::new(driver.to_dyn_trait());

                        // FIXME handle error
                        let path = driver.deploy().unwrap();

                        let diff_info = match game.is_game_installed(path.to_string_lossy()) {
                            Ok(true) => {
                                match game.get_game_diff(path.to_string_lossy(), info.get_edition()) {
                                    Ok(Some(diff)) => diff.diff,

                                    Ok(None) => {
                                        sender.input(MainAppMsg::ShowToast {
                                            title: format!("{} is not installed", info.get_title()),
                                            message: None
                                        });

                                        return;
                                    }

                                    Err(err) => {
                                        sender.input(MainAppMsg::ShowToast {
                                            title: format!("Unable to find {} version diff", info.get_title()),
                                            message: Some(err.to_string())
                                        });

                                        return;
                                    }
                                }
                            }

                            Ok(false) => {
                                match game.get_game_download(info.get_edition()) {
                                    Ok(download) => Some(download.download),
                                    Err(err) => {
                                        sender.input(MainAppMsg::ShowToast {
                                            title: format!("Unable to find {} version diff", info.get_title()),
                                            message: Some(err.to_string())
                                        });
    
                                        return;
                                    }
                                }
                            }

                            Err(err) => {
                                sender.input(MainAppMsg::ShowToast {
                                    title: format!("Unable to find {} version diff", info.get_title()),
                                    message: Some(err.to_string())
                                });

                                return;
                            }
                        };

                        if let Some(diff_info) = diff_info {
                            let task = Box::new(DownloadDiffQueuedTask {
                                driver: driver.clone(),
                                card_info: info.clone(),
                                diff_info
                            });

                            // FIXME handle error
                            driver.dismantle().unwrap();

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
                    }

                    Ok(None) => {
                        sender.input(MainAppMsg::ShowToast {
                            title: format!("Unable to find {} integration script", info.get_title()),
                            message: None
                        });
                    }

                    Err(err) => {
                        sender.input(MainAppMsg::ShowToast {
                            title: format!("Unable to find {} integration script", info.get_title()),
                            message: Some(err.to_string())
                        });
                    }
                }
            }

            MainAppMsg::AddVerifyGameTask(info) => {
                let task = Box::new(VerifyIntegrityQueuedTask {
                    info: info.clone()
                });

                self.tasks_queue.emit(TasksQueueComponentInput::AddTask(task));

                if let Some(index) = self.available_games_indexes.get(&info) {
                    self.available_games.guard().remove(index.current_index());

                    self.available_games_indexes.remove(&info);

                    #[allow(clippy::map_entry)]
                    if !self.queued_games_indexes.contains_key(&info) {
                        self.queued_games_indexes.insert(info.clone(), self.queued_games.guard().push_back(info));

                        self.queued_games.broadcast(CardComponentInput::SetInstalled(false));
                        self.queued_games.broadcast(CardComponentInput::SetClickable(false));
                    }
                }
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
