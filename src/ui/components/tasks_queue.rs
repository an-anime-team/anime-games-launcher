use std::collections::VecDeque;
use std::thread::JoinHandle;
use std::time::Instant;

use relm4::factory::FactoryVecDeque;
use relm4::prelude::*;
use relm4::component::*;

use gtk::prelude::*;

use anime_game_core::updater::UpdaterExt;

use anime_game_core::game::genshin::diff::{
    Updater as GenshinDiffUpdater,
    Status as GenshinDiffStatus,
    Error as GenshinDiffError
};

use crate::ui::components::game_card::{
    GameCardComponent,
    GameCardComponentInput,
    CardVariant
};

use crate::ui::components::factory::game_card_tasks::GameCardFactory;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// All the possible tasks statuses in one enum
pub enum UnifiedTaskStatus {
    PreparingTransition,
    Downloading,
    Unpacking,
    FinishingTransition,
    ApplyingHdiffPatches,
    DeletingObsoleteFiles,
    Finished
}

pub enum Task {
    DownloadGenshinDiff {
        updater: GenshinDiffUpdater
    }
}

impl std::fmt::Debug for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DownloadGenshinDiff { .. } => {
                f.debug_struct("DownloadGenshinDiff")
                    .finish()
            }
        }
    }
}

impl Task {
    pub fn get_variant(&self) -> CardVariant {
        match self {
            Self::DownloadGenshinDiff { .. } => CardVariant::Genshin
        }
    }

    /// Check if the task is finished
    pub fn is_finished(&mut self) -> bool {
        match self {
            Self::DownloadGenshinDiff { updater } => updater.is_finished()
        }
    }

    /// Get current task progress
    pub fn get_current(&self) -> usize {
        match self {
            Self::DownloadGenshinDiff { updater } => updater.current()
        }
    }

    /// Get total task progress
    pub fn get_total(&self) -> usize {
        match self {
            Self::DownloadGenshinDiff { updater } => updater.total()
        }
    }

    /// Get task completion progress
    pub fn get_progress(&self) -> f64 {
        match self {
            Self::DownloadGenshinDiff { updater } => updater.progress()
        }
    }

    /// Get task status
    pub fn get_unified_status(&mut self) -> anyhow::Result<UnifiedTaskStatus> {
        match self {
            Self::DownloadGenshinDiff { updater } => {
                match updater.status() {
                    Ok(status) => Ok(match status {
                        GenshinDiffStatus::PreparingTransition   => UnifiedTaskStatus::PreparingTransition,
                        GenshinDiffStatus::Downloading           => UnifiedTaskStatus::Downloading,
                        GenshinDiffStatus::Unpacking             => UnifiedTaskStatus::Unpacking,
                        GenshinDiffStatus::FinishingTransition   => UnifiedTaskStatus::FinishingTransition,
                        GenshinDiffStatus::ApplyingHdiffPatches  => UnifiedTaskStatus::ApplyingHdiffPatches,
                        GenshinDiffStatus::DeletingObsoleteFiles => UnifiedTaskStatus::DeletingObsoleteFiles,
                        GenshinDiffStatus::Finished              => UnifiedTaskStatus::Finished
                    }),

                    Err(err) => anyhow::bail!(err.to_string())
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct TasksQueueComponent {
    pub current_task_card: AsyncController<GameCardComponent>,
    pub current_task_status: String,
    pub current_task_progress_start: Instant,
    pub current_task: Option<Task>,

    pub queued_tasks_factory: FactoryVecDeque<GameCardFactory>,
    pub queued_tasks: VecDeque<Task>,

    pub progress_bar: gtk::ProgressBar,

    pub updater: Option<JoinHandle<()>>
}

#[derive(Debug)]
pub enum TasksQueueComponentInput {
    AddTask(Task),
    UpdateCurrentTask,
    StartUpdater,
    StopUpdater
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TasksQueueComponentOutput {
    GameDownloaded(CardVariant),

    ShowToast {
        title: String,
        message: Option<String>
    }
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for TasksQueueComponent {
    type Init = CardVariant;
    type Input = TasksQueueComponentInput;
    type Output = TasksQueueComponentOutput;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            set_margin_all: 24,

            gtk::Box {
                set_halign: gtk::Align::Center,

                #[watch]
                set_visible: model.current_task.is_some(),

                model.current_task_card.widget(),
            },

            adw::Clamp {
                set_maximum_size: 200,

                #[watch]
                set_visible: model.current_task.is_none(),

                gtk::Picture {
                    set_filename: Some("images/raiden.png")
                }
            },

            gtk::Label {
                set_halign: gtk::Align::Center,

                set_margin_top: 24,

                add_css_class: "title-4",

                #[watch]
                set_label: &match &model.current_task {
                    Some(task) => format!("Downloading {}", task.get_variant().get_title()),
                    None => String::from("Nothing to do")
                }
            },

            gtk::Label {
                set_halign: gtk::Align::Start,

                set_margin_top: 8,

                #[watch]
                set_visible: model.current_task.is_some(),

                #[watch]
                set_label: &model.current_task_status
            },

            #[local_ref]
            progress_bar -> gtk::ProgressBar {
                set_margin_top: 16,

                #[watch]
                set_visible: model.current_task.is_some(),
            },

            gtk::Label {
                set_halign: gtk::Align::Start,

                set_margin_top: 16,

                #[watch]
                set_visible: model.current_task.is_some(),

                set_label: "Download speed: 20 MB/s"

                // #[watch]
                // set_label: &match &model.current_task {
                //     Some(task) => format!("Download speed: {} bytes/s", (task.get_current() as f64 / (Instant::now() - model.current_task_progress_start).as_secs_f64()).ceil()),
                //     None => String::new()
                // }
            },

            gtk::Label {
                set_halign: gtk::Align::Start,

                set_margin_top: 8,

                #[watch]
                set_visible: model.current_task.is_some(),

                set_label: "ETA: 7 minutes"
            },

            gtk::ScrolledWindow {
                set_margin_top: 64,

                model.queued_tasks_factory.widget(),
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let flow_box = gtk::FlowBox::new();

        flow_box.set_valign(gtk::Align::End);
        flow_box.set_selection_mode(gtk::SelectionMode::None);

        flow_box.set_vexpand(true);
        flow_box.set_homogeneous(true);

        let model = Self {
            current_task_card: GameCardComponent::builder()
                .launch(init)
                .detach(),

            current_task_status: String::new(),
            current_task_progress_start: Instant::now(),
            current_task: None,

            queued_tasks_factory: FactoryVecDeque::new(flow_box, sender.input_sender()),
            queued_tasks: VecDeque::new(),

            progress_bar: gtk::ProgressBar::new(),

            updater: None
        };

        model.current_task_card.emit(GameCardComponentInput::SetWidth(160));
        model.current_task_card.emit(GameCardComponentInput::SetHeight(224));

        model.current_task_card.emit(GameCardComponentInput::SetClickable(false));
        model.current_task_card.emit(GameCardComponentInput::SetDisplayTitle(false));

        let progress_bar = &model.progress_bar;

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            TasksQueueComponentInput::AddTask(task) => {
                if self.current_task.is_none() {
                    self.current_task_card.emit(GameCardComponentInput::SetVariant(task.get_variant()));

                    self.current_task = Some(task);
                    self.current_task_progress_start = Instant::now();
                }

                else {
                    self.queued_tasks_factory.guard().push_back(task.get_variant());

                    self.queued_tasks.push_back(task);
                }

                sender.input(TasksQueueComponentInput::StartUpdater);
            }

            TasksQueueComponentInput::UpdateCurrentTask => {
                if let Some(task) = &mut self.current_task {
                    if task.is_finished() {
                        if let Err(err) = task.get_unified_status() {
                            sender.output(TasksQueueComponentOutput::ShowToast {
                                title: format!("Failed to download {}", task.get_variant().get_title()),
                                message: Some(err.to_string())
                            }).unwrap();
                        }

                        let mut is_task_queued = false;

                        for queued_task in &self.queued_tasks {
                            if queued_task.get_variant() == task.get_variant() {
                                is_task_queued = true;

                                break;
                            }
                        }

                        if !is_task_queued {
                            sender.output(TasksQueueComponentOutput::GameDownloaded(task.get_variant())).unwrap();
                        }

                        if let Some(queued_task) = self.queued_tasks.pop_front() {
                            self.queued_tasks_factory.guard().pop_front();

                            self.current_task_card.emit(GameCardComponentInput::SetVariant(queued_task.get_variant()));

                            self.current_task = Some(queued_task);
                        }

                        else {
                            self.current_task = None;

                            sender.input(TasksQueueComponentInput::StopUpdater);
                        }

                        self.current_task_status.clear();
                    }

                    else {
                        self.progress_bar.set_fraction(task.get_progress());

                        if let Ok(status) = task.get_unified_status() {
                            let title = match status {
                                UnifiedTaskStatus::PreparingTransition   => String::from("Preparing transition..."),
                                UnifiedTaskStatus::Downloading           => String::from("Downloading..."),
                                UnifiedTaskStatus::Unpacking             => String::from("Unpacking..."),
                                UnifiedTaskStatus::FinishingTransition   => String::from("Finishing transition..."),
                                UnifiedTaskStatus::ApplyingHdiffPatches  => String::from("Applying hdiff patches..."),
                                UnifiedTaskStatus::DeletingObsoleteFiles => String::from("Deleting obsolete files..."),
                                UnifiedTaskStatus::Finished              => String::from("Finished")
                            };

                            self.current_task_status = title;
                        }
                    }
                }
            }

            TasksQueueComponentInput::StartUpdater => {
                self.updater = Some(std::thread::spawn(move || {
                    loop {
                        sender.input(TasksQueueComponentInput::UpdateCurrentTask);
    
                        std::thread::sleep(std::time::Duration::from_millis(20));
                    }
                }));
            }

            TasksQueueComponentInput::StopUpdater => {
                self.updater = None;
            }
        }
    }
}
