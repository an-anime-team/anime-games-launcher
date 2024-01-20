// FIXME: get rid of deprecated libadwaita components
#![allow(deprecated)]

use std::path::PathBuf;
use std::collections::{HashMap, HashSet};

use relm4::prelude::*;
use relm4::factory::*;
use relm4::actions::*;

use gtk::prelude::*;
use adw::prelude::*;

use crate::tr;

use crate::config;
use crate::games;

use crate::components::wine::*;
use crate::components::dxvk::*;

use crate::config::games::settings::edition_addons::GameEditionAddon;

use crate::games::metadata::LauncherMetadata;
use crate::games::integrations::standards::addons::{
    Addon,
    AddonsGroup
};

use crate::ui::windows::preferences::PreferencesApp;

use crate::ui::windows::about::{
    AboutDialog,
    AboutDialogMsg
};

use crate::ui::windows::loading::load_app::LoadingResult;

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

    apply_dxvk_task::ApplyDxvkQueuedTask,
    create_prefix_task::CreatePrefixQueuedTask
};

use crate::LAUNCHER_FOLDER;
use crate::CONFIG_FILE;
use crate::DEBUG_FILE;

pub mod launch_game;
pub mod kill_game;
pub mod download_game_task;
pub mod download_addon_task;
pub mod uninstall_addon_task;
pub mod verify_game_task;

pub static mut WINDOW: Option<adw::Window> = None;
pub static mut PREFERENCES_APP: Option<AsyncController<PreferencesApp>> = None;
pub static mut GAME_ADDONS_MANAGER_APP: Option<AsyncController<GameAddonsManagerApp>> = None;
pub static mut ABOUT_DIALOG: Option<Controller<AboutDialog>> = None;

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
    outdated_games: FactoryVecDeque<CardFactory>,
    available_games: FactoryVecDeque<CardFactory>,

    running_games_indexes: HashMap<CardInfo, DynamicIndex>,
    installed_games_indexes: HashMap<CardInfo, DynamicIndex>,
    queued_games_indexes: HashMap<CardInfo, DynamicIndex>,
    outdated_games_indexes: HashMap<CardInfo, DynamicIndex>,
    available_games_indexes: HashMap<CardInfo, DynamicIndex>,

    tasks_queue: AsyncController<TasksQueueComponent>
}

#[derive(Debug)]
pub enum MainAppMsg {
    InitMainApp(LoadingResult),

    OpenDetails {
        info: CardInfo,
        installed: bool,
        running: bool
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

    AddDownloadAddonTask {
        game_info: CardInfo,
        addon: Addon,
        group: AddonsGroup
    },

    AddUninstallAddonTask {
        game_info: CardInfo,
        addon: Addon,
        group: AddonsGroup
    },

    AddDownloadWineTask(Wine),
    AddDownloadDxvkTask(Dxvk),
    AddApplyDxvkTask(Dxvk),

    AddCreatePrefixTask {
        path: PathBuf,
        install_corefonts: bool
    },

    LaunchGame(CardInfo),
    KillGame(CardInfo),
    FinishRunningGame(CardInfo),

    ShowToast {
        title: String,
        message: Option<String>
    }
}

relm4::new_action_group!(WindowActionGroup, "win");

relm4::new_stateless_action!(LauncherFolder, WindowActionGroup, "launcher_folder");
relm4::new_stateless_action!(ConfigFile, WindowActionGroup, "config_file");
relm4::new_stateless_action!(DebugFile, WindowActionGroup, "debug_file");

relm4::new_stateless_action!(About, WindowActionGroup, "about");

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for MainApp {
    type Init = ();
    type Input = MainAppMsg;
    type Output = ();

    menu! {
        main_menu: {
            section! {
                &tr!("main-menu-launcher-folder") => LauncherFolder,
                &tr!("main-menu-config-file")     => ConfigFile,
                &tr!("main-menu-debug-file")      => DebugFile,
            },

            section! {
                &tr!("main-menu-about") => About
            }
        }
    }

    view! {
        window = adw::Window {
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

                            pack_end = &gtk::MenuButton {
                                set_icon_name: "open-menu-symbolic",
                                set_menu_model: Some(&main_menu)
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

                                        #[watch]
                                        set_margin_all: if model.running_games.is_empty() { 0 } else { 16 },

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

                                        #[watch]
                                        set_margin_all: if model.installed_games.is_empty() { 0 } else { 16 },

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

                                        #[watch]
                                        set_margin_all: if model.queued_games.is_empty() { 0 } else { 16 },

                                        set_homogeneous: true,
                                        set_selection_mode: gtk::SelectionMode::None
                                    },

                                    gtk::Label {
                                        set_halign: gtk::Align::Start,

                                        set_margin_start: 24,
                                        add_css_class: "title-4",

                                        #[watch]
                                        set_visible: !model.outdated_games.is_empty(),

                                        set_label: "Outdated games"
                                    },

                                    #[local_ref]
                                    outdated_games_flow_box -> gtk::FlowBox {
                                        set_row_spacing: 12,
                                        set_column_spacing: 12,

                                        #[watch]
                                        set_margin_all: if model.outdated_games.is_empty() { 0 } else { 16 },

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

                                        #[watch]
                                        set_margin_all: if model.available_games.is_empty() { 0 } else { 16 },

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

                        #[watch]
                        set_css_classes: &[
                            &format!("game-details--{}--{}", model.game_details_info.get_name(), model.game_details_info.get_edition())
                        ],

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

    async fn init(_init: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let  model = Self {
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

                    GameDetailsComponentOutput::KillGame(info)
                        => MainAppMsg::KillGame(info),

                    GameDetailsComponentOutput::OpenAddonsManager(info)
                        => MainAppMsg::OpenAddonsManager(info),

                    GameDetailsComponentOutput::ShowToast { title, message }
                        => MainAppMsg::ShowToast { title, message }
                }),

            game_details_info: CardInfo::default(),

            running_games_indexes: HashMap::new(),
            installed_games_indexes: HashMap::new(),
            queued_games_indexes: HashMap::new(),
            outdated_games_indexes: HashMap::new(),
            available_games_indexes: HashMap::new(),

            running_games: FactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |output: CardComponentOutput| -> MainAppMsg {
                    match output {
                        CardComponentOutput::CardClicked { info, installed }
                            => MainAppMsg::OpenDetails { info, installed, running: true }
                    }
                }),

            installed_games: FactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |output: CardComponentOutput| -> MainAppMsg {
                    match output {
                        CardComponentOutput::CardClicked { info, installed }
                            => MainAppMsg::OpenDetails { info, installed, running: false }
                    }
                }),

            queued_games: FactoryVecDeque::builder()
                .launch_default()
                .detach(),
                // .forward(sender.input_sender(), |output: CardComponentOutput| -> MainAppMsg {
                //     match output {
                //         CardComponentOutput::CardClicked { info, installed }
                //             => MainAppMsg::OpenDetails { info, installed, running: false }
                //     }
                // }),

            outdated_games: FactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |output: CardComponentOutput| -> MainAppMsg {
                    match output {
                        CardComponentOutput::CardClicked { info, installed: _ }
                            => MainAppMsg::OpenDetails { info, installed: false, running: false }
                    }
                }),

            available_games: FactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |output: CardComponentOutput| -> MainAppMsg {
                    match output {
                        CardComponentOutput::CardClicked { info, installed }
                            => MainAppMsg::OpenDetails { info, installed, running: false }
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

        let leaflet = &model.leaflet;
        let flap = &model.flap;

        let main_toast_overlay = &model.main_toast_overlay;
        let game_details_toast_overlay = &model.game_details_toast_overlay;

        let running_games_flow_box = model.running_games.widget();
        let installed_games_flow_box = model.installed_games.widget();
        let queued_games_flow_box = model.queued_games.widget();
        let outdated_games_flow_box = model.outdated_games.widget();
        let available_games_flow_box = model.available_games.widget();

        let widgets = view_output!();

        let about_dialog_broker: relm4::MessageBroker<AboutDialogMsg> = relm4::MessageBroker::new();

        unsafe {
            WINDOW = Some(widgets.window.clone());

            PREFERENCES_APP = Some(PreferencesApp::builder()
                .launch(widgets.window.clone())
                .detach());

            GAME_ADDONS_MANAGER_APP = Some(GameAddonsManagerApp::builder()
                .launch(widgets.window.clone())
                .forward(sender.input_sender(), std::convert::identity));

            ABOUT_DIALOG = Some(AboutDialog::builder()
                .transient_for(widgets.window.clone())
                .launch_with_broker((), &about_dialog_broker)
                .detach());
        }

        let mut group = RelmActionGroup::<WindowActionGroup>::new();

        group.add_action::<LauncherFolder>(RelmAction::new_stateless(gtk::glib::clone!(@strong sender => move |_| {
            if let Err(err) = open::that(LAUNCHER_FOLDER.as_path()) {
                sender.input(MainAppMsg::ShowToast {
                    title: String::from("Failed to open launcher folder"),
                    message: Some(err.to_string())
                });

                tracing::error!("Failed to open launcher folder: {err}");
            }
        })));

        group.add_action::<ConfigFile>(RelmAction::new_stateless(gtk::glib::clone!(@strong sender => move |_| {
            if let Err(err) = open::that(CONFIG_FILE.as_path()) {
                sender.input(MainAppMsg::ShowToast {
                    title: String::from("Failed to open config file"),
                    message: Some(err.to_string())
                });

                tracing::error!("Failed to open config file: {err}");
            }
        })));

        group.add_action::<DebugFile>(RelmAction::new_stateless(gtk::glib::clone!(@strong sender => move |_| {
            if let Err(err) = open::that(DEBUG_FILE.as_path()) {
                sender.input(MainAppMsg::ShowToast {
                    title: String::from("Failed to open debug file"),
                    message: Some(err.to_string())
                });

                tracing::error!("Failed to open debug file: {err}");
            }
        })));

        group.add_action::<About>(RelmAction::new_stateless(move |_| {
            about_dialog_broker.send(AboutDialogMsg::Show);
        }));

        widgets.window.insert_action_group("win", Some(&group.into_action_group()));

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            MainAppMsg::InitMainApp(init) => {
                for game in init.games_list.available {
                    let card = CardInfo::Game {
                        name: game.game_name.clone(),
                        title: game.game_title.clone(),
                        developer: game.game_developer.clone(),
                        edition: game.edition.name.clone(),
                        picture_uri: game.card_picture.clone()
                    };

                    self.available_games_indexes.insert(
                        card.to_owned(),
                        self.available_games.guard().push_back(card.to_owned())
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

                    self.installed_games_indexes.insert(
                        card.to_owned(),
                        self.installed_games.guard().push_back(card.to_owned())
                    );
                }

                for game in init.games_list.outdated {
                    let card = CardInfo::Game {
                        name: game.game_name.clone(),
                        title: game.game_title.clone(),
                        developer: game.game_developer.clone(),
                        edition: game.edition.name.clone(),
                        picture_uri: game.card_picture.clone()
                    };

                    self.outdated_games_indexes.insert(
                        card.to_owned(),
                        self.outdated_games.guard().push_back(card.to_owned())
                    );
                }

                self.available_games.broadcast(CardComponentInput::SetInstalled(false));
                self.outdated_games.broadcast(CardComponentInput::SetInstalled(false));

                if let Some(wine) = init.download_wine {
                    sender.input(MainAppMsg::AddDownloadWineTask(wine));
                    sender.input(MainAppMsg::ShowTasksFlap);
                }

                if let Some(dxvk) = init.download_dxvk {
                    sender.input(MainAppMsg::AddDownloadDxvkTask(dxvk));
                    sender.input(MainAppMsg::ShowTasksFlap);
                }

                if let Some(dxvk) = init.apply_dxvk {
                    sender.input(MainAppMsg::AddApplyDxvkTask(dxvk));
                    sender.input(MainAppMsg::ShowTasksFlap);
                }

                if let Some(prefix) = init.create_prefix {
                    sender.input(MainAppMsg::AddCreatePrefixTask {
                        path: prefix.path.clone(),
                        install_corefonts: prefix.install_corefonts
                    });

                    sender.input(MainAppMsg::ShowTasksFlap);
                }

                for addon in init.download_addons {
                    sender.input(MainAppMsg::AddDownloadAddonTask {
                        game_info: addon.game_info,
                        addon: addon.addon,
                        group: addon.group
                    });

                    sender.input(MainAppMsg::ShowTasksFlap);
                }
            }

            MainAppMsg::OpenDetails { info, installed, running } => {
                self.game_details_info = info.clone();

                self.game_details.emit(GameDetailsComponentInput::SetInfo(info.clone()));
                self.game_details.emit(GameDetailsComponentInput::SetInstalled(installed));
                self.game_details.emit(GameDetailsComponentInput::SetRunning(running));

                if !installed {
                    self.game_details.emit(GameDetailsComponentInput::SetStatus(None));
                }

                else {
                    let game = unsafe {
                        games::get_unsafe(info.get_name())
                    };

                    let settings = config::get().games.get_game_settings(game).unwrap();

                    let paths = settings
                        .paths
                        .get(info.get_edition())
                        .unwrap();

                    let metadata = LauncherMetadata::load_for_game(info.get_name(), info.get_edition()).unwrap();

                    self.game_details.emit(GameDetailsComponentInput::SetMetadata(metadata));

                    match game.driver.get_game_status(&paths.game.to_string_lossy(), info.get_edition()) {
                        Ok(status) => {
                            self.game_details.emit(GameDetailsComponentInput::SetStatus(status));
                        }

                        Err(err) => {
                            sender.input(MainAppMsg::ShowToast {
                                title: format!("Unable to get {} status", info.get_title()),
                                message: Some(err.to_string())
                            });
                        }
                    }
                }

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

                match game.driver.get_addons_list(game_info.get_edition()) {
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

            MainAppMsg::AddDownloadGameTask(game_info) => {
                let config = config::get();

                match download_game_task::get_download_game_task(&game_info, &config) {
                    Ok(result) => {
                        self.tasks_queue.emit(TasksQueueComponentInput::AddTask(result.game_task));

                        if let Some(index) = self.available_games_indexes.get(&game_info) {
                            self.available_games.guard().remove(index.current_index());
                            self.available_games_indexes.remove(&game_info);
                        }

                        else if let Some(index) = self.outdated_games_indexes.get(&game_info) {
                            self.outdated_games.guard().remove(index.current_index());
                            self.outdated_games_indexes.remove(&game_info);
                        }

                        #[allow(clippy::map_entry)]
                        if !self.queued_games_indexes.contains_key(&game_info) {
                            self.queued_games_indexes.insert(game_info.clone(), self.queued_games.guard().push_back(game_info.clone()));

                            self.queued_games.broadcast(CardComponentInput::SetInstalled(false));
                            self.queued_games.broadcast(CardComponentInput::SetClickable(false));
                        }

                        if config.general.verify_games {
                            sender.input(MainAppMsg::AddVerifyGameTask(game_info.clone()));
                        }

                        for addon in result.download_addons {
                            sender.input(MainAppMsg::AddDownloadAddonTask {
                                game_info: game_info.clone(),
                                addon: addon.addon,
                                group: addon.group
                            });
                        }
                    }

                    Err(err) => sender.input(*err)
                }
            }

            MainAppMsg::AddVerifyGameTask(game_info) => {
                let config = config::get();

                match verify_game_task::get_verify_game_task(&game_info, &config) {
                    Ok(task) => {
                        self.tasks_queue.emit(TasksQueueComponentInput::AddTask(task));

                        if let Some(index) = self.installed_games_indexes.get(&game_info) {
                            self.installed_games.guard().remove(index.current_index());
                            self.installed_games_indexes.remove(&game_info);

                            #[allow(clippy::map_entry)]
                            if !self.queued_games_indexes.contains_key(&game_info) {
                                self.queued_games_indexes.insert(game_info.clone(), self.queued_games.guard().push_back(game_info.clone()));

                                self.queued_games.broadcast(CardComponentInput::SetInstalled(false));
                                self.queued_games.broadcast(CardComponentInput::SetClickable(false));
                            }
                        }
                    }

                    Err(err) => sender.input(*err)
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

            MainAppMsg::AddDownloadAddonTask { game_info, addon, group } => {
                unsafe {
                    GAME_ADDONS_MANAGER_APP.as_ref()
                        .unwrap_unchecked()
                        .widget()
                        .close();
                }

                sender.input(MainAppMsg::HideDetails);
                sender.input(MainAppMsg::ShowTasksFlap);

                match download_addon_task::get_download_addon_task(&game_info, &addon, &group) {
                    Ok(task) => {
                        // TODO: should I move game to "queued"?
                        self.tasks_queue.emit(TasksQueueComponentInput::AddTask(task));
                    }

                    Err(err) => sender.input(*err)
                }
            }

            MainAppMsg::AddUninstallAddonTask { game_info, addon, group } => {
                unsafe {
                    GAME_ADDONS_MANAGER_APP.as_ref()
                        .unwrap_unchecked()
                        .widget()
                        .close();
                }

                sender.input(MainAppMsg::HideDetails);
                sender.input(MainAppMsg::ShowTasksFlap);

                match uninstall_addon_task::get_uninstall_addon_task(&game_info, &addon, &group) {
                    Ok(task) => {
                        // TODO: should I move game to "queued"?
                        self.tasks_queue.emit(TasksQueueComponentInput::AddTask(task));
                    }

                    Err(err) => sender.input(*err)
                }
            }

            MainAppMsg::AddDownloadWineTask(version) => {
                self.tasks_queue.emit(TasksQueueComponentInput::AddTask(Box::new(DownloadWineQueuedTask {
                    card_info: CardInfo::Component {
                        name: version.name.clone(),
                        title: version.title.clone(),
                        developer: String::new()
                    },
                    version
                })));
            }

            MainAppMsg::AddDownloadDxvkTask(version) => {
                self.tasks_queue.emit(TasksQueueComponentInput::AddTask(Box::new(DownloadDxvkQueuedTask {
                    card_info: CardInfo::Component {
                        name: version.name.clone(),
                        title: version.name.clone(), // version.title.clone(),
                        developer: String::new()
                    },
                    version
                })));
            }

            MainAppMsg::AddApplyDxvkTask(version) => {
                self.tasks_queue.emit(TasksQueueComponentInput::AddTask(Box::new(ApplyDxvkQueuedTask {
                    card_info: CardInfo::Component {
                        name: version.name.clone(),
                        title: version.title.clone(),
                        developer: String::new()
                    },
                    dxvk_version: version,
                    prefix_path: config::get().components.wine.prefix.path
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

                    if self.game_details_info == info {
                        self.game_details.emit(GameDetailsComponentInput::SetRunning(true));
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

            MainAppMsg::KillGame(info) => {
                if let Err(err) = kill_game::kill_game(&info) {
                    sender.input(MainAppMsg::ShowToast {
                        title: format!("Failed to kill {}", info.get_title()),
                        message: Some(err.to_string())
                    });
                }

                // TODO: set_sensitive(false) for a few seconds
            }

            MainAppMsg::FinishRunningGame(info) => {
                if let Some(index) = self.running_games_indexes.get(&info) {
                    self.running_games.guard().remove(index.current_index());

                    self.running_games_indexes.remove(&info);

                    #[allow(clippy::map_entry)]
                    if !self.installed_games_indexes.contains_key(&info) {
                        self.installed_games_indexes.insert(info.clone(), self.installed_games.guard().push_back(info.clone()));
                    }
                }

                if self.game_details_info == info {
                    let metadata = LauncherMetadata::load_for_game(info.get_name(), info.get_edition()).unwrap();

                    self.game_details.emit(GameDetailsComponentInput::SetRunning(false));
                    self.game_details.emit(GameDetailsComponentInput::SetMetadata(metadata));
                }
            }

            MainAppMsg::ShowToast { title, message } => {
                let window = unsafe {
                    WINDOW.as_ref().unwrap_unchecked()
                };

                let toast = adw::Toast::new(&title);

                // toast.set_timeout(7);

                if let Some(message) = message {
                    toast.set_button_label(Some(&tr!("dialog-toast-details")));

                    let dialog = adw::MessageDialog::new(
                        Some(window),
                        Some(&title),
                        Some(&message)
                    );

                    dialog.add_response("close", &tr!("dialog-close"));
                    dialog.add_response("save", &tr!("dialog-save"));

                    dialog.set_response_appearance("save", adw::ResponseAppearance::Suggested);

                    dialog.connect_response(Some("save"), |_, _| {
                        if let Err(err) = open::that(DEBUG_FILE.as_path()) {
                            tracing::error!("Failed to open debug file: {err}");
                        }
                    });

                    toast.connect_button_clicked(move |_| {
                        dialog.present();
                    });
                }

                self.main_toast_overlay.add_toast(toast);
            }
        }
    }
}
