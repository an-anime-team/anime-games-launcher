use std::collections::VecDeque;
use std::thread::JoinHandle;
use std::time::Instant;

use relm4::factory::FactoryVecDeque;
use relm4::prelude::*;
use relm4::component::*;

use gtk::prelude::*;

use crate::ui::components::game_card::{
    GameCardComponent,
    GameCardComponentInput,
    CardVariant
};

use crate::ui::components::factory::game_card_tasks::GameCardFactory;

pub mod queued_task;
pub mod resolved_task;
pub mod create_prefix_task;

pub use queued_task::QueuedTask;

pub use resolved_task::{
    ResolvedTask,
    TaskStatus
};

#[derive(Debug)]
pub struct TasksQueueComponent {
    pub current_task: Option<ResolvedTask>,
    pub current_task_card: AsyncController<GameCardComponent>,
    pub current_task_status: String,
    pub current_task_progress_start: Instant,
    pub current_task_progress_pulse: bool,

    pub queued_tasks_factory: FactoryVecDeque<GameCardFactory>,
    pub queued_tasks: VecDeque<QueuedTask>,

    pub progress_bar: gtk::ProgressBar,

    pub updater: Option<JoinHandle<()>>
}

#[derive(Debug)]
pub enum TasksQueueComponentInput {
    AddTask(QueuedTask),
    UpdateCurrentTask,
    StartUpdater,
    StopUpdater
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TasksQueueComponentOutput {
    GameDownloaded(CardVariant),

    HideTasksFlap,

    ShowToast {
        title: String,
        message: Option<String>
    }
}

#[relm4::component(pub, async)]
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
                    Some(task) => task.get_title(),
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
            current_task: None,

            current_task_card: GameCardComponent::builder()
                .launch(init)
                .detach(),

            current_task_status: String::new(),
            current_task_progress_start: Instant::now(),
            current_task_progress_pulse: false,

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
                    match task.resolve() {
                        Ok(task) => {
                            self.current_task_card.emit(GameCardComponentInput::SetVariant(task.get_variant()));

                            self.current_task = Some(task);
                            self.current_task_progress_start = Instant::now();
                        }

                        Err(err) => {
                            sender.output(TasksQueueComponentOutput::ShowToast {
                                title: String::from("Failed to resolve queued task"),
                                message: Some(err.to_string())
                            }).unwrap();
                        }
                    }
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
                        if let Err(err) = task.get_status() {
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

                            match queued_task.resolve() {
                                Ok(task) => {
                                    self.current_task_card.emit(GameCardComponentInput::SetVariant(task.get_variant()));

                                    self.current_task = Some(task);
                                }

                                Err(err) => {
                                    sender.output(TasksQueueComponentOutput::ShowToast {
                                        title: String::from("Failed to resolve queued task"),
                                        message: Some(err.to_string())
                                    }).unwrap();
                                }
                            }
                        }

                        else {
                            self.current_task = None;

                            sender.input(TasksQueueComponentInput::StopUpdater);
                            sender.output(TasksQueueComponentOutput::HideTasksFlap).unwrap();
                        }

                        self.current_task_status.clear();
                    }

                    else {
                        if self.current_task_progress_pulse {
                            self.progress_bar.pulse();
                        }

                        else {
                            self.progress_bar.set_fraction(task.get_progress());
                        }

                        if let Ok(status) = task.get_status() {
                            let (pulse, title) = match status {
                                TaskStatus::PreparingTransition => (true, String::from("Preparing transition...")),
                                TaskStatus::FinishingTransition => (true, String::from("Finishing transition...")),

                                TaskStatus::Downloading => (false, String::from("Downloading...")),
                                TaskStatus::Unpacking   => (false, String::from("Unpacking...")),

                                TaskStatus::ApplyingHdiffPatches  => (false, String::from("Applying hdiff patches...")),
                                TaskStatus::DeletingObsoleteFiles => (false, String::from("Deleting obsolete files...")),

                                TaskStatus::CreatingPrefix             => (true, String::from("Creating prefix...")),
                                TaskStatus::InstallingFont(font) => (false, format!("Installing font: {}...", font.name())),

                                TaskStatus::Finished => (true, String::from("Finished"))
                            };

                            self.current_task_progress_pulse = pulse;
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
