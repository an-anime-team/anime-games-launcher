use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::{Instant, Duration};

use relm4::factory::FactoryVecDeque;
use relm4::prelude::*;

use gtk::prelude::*;

use crate::tr;

use crate::ui::components::game_card::{
    CardInfo,
    CardComponent,
    CardComponentInput
};

use crate::ui::components::factory::game_card_tasks::CardFactory;

use crate::utils::{
    pretty_bytes,
    pretty_seconds
};

pub mod task;
pub mod create_prefix_task;
pub mod apply_dxvk_task;
pub mod download_diff_task;
pub mod verify_integrity_task;
pub mod delete_files_task;

pub use task::*;

pub const UPDATER_TIMEOUT: Duration = Duration::from_millis(20);

#[derive(Debug)]
pub struct TasksQueueProgressUpdater {
    pub thread: JoinHandle<()>,
    pub running: Arc<AtomicBool>
}

impl Drop for TasksQueueProgressUpdater {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

#[derive(Debug)]
pub struct TasksQueueComponent {
    pub current_task: Option<Box<dyn ResolvedTask>>,
    pub current_task_card: AsyncController<CardComponent>,
    pub current_task_status: String,
    pub current_task_progress_start: Instant,
    pub current_task_progress_pulse: bool,

    pub queued_tasks_factory: FactoryVecDeque<CardFactory>,
    pub queued_tasks: VecDeque<Box<dyn QueuedTask>>,

    pub progress_label: gtk::Label,
    pub progress_bar: gtk::ProgressBar,

    pub updater: Option<TasksQueueProgressUpdater>
}

#[derive(Debug)]
pub enum TasksQueueComponentInput {
    AddTask(Box<dyn QueuedTask>),
    UpdateCurrentTask,
    StartUpdater,
    StopUpdater
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TasksQueueComponentOutput {
    TaskFinished(CardInfo),

    HideTasksFlap,

    ShowToast {
        title: String,
        message: Option<String>
    }
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for TasksQueueComponent {
    type Init = CardInfo;
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
                    set_resource: Some(&format!("{}/icons/hicolor/scalable/apps/{}.png", crate::APP_RESOURCE_PREFIX, crate::APP_ID))
                }
            },

            gtk::Label {
                set_halign: gtk::Align::Center,

                set_margin_top: 24,

                add_css_class: "title-4",

                #[watch]
                set_label: &match &model.current_task {
                    Some(task) => task.get_info().get_title().to_string(),
                    None => tr!("tasks-nothing")
                }
            },

            gtk::CenterBox {
                set_margin_top: 16,

                #[watch]
                set_visible: model.current_task.is_some(),

                #[wrap(Some)]
                set_start_widget = &gtk::Label {
                    #[watch]
                    set_label: &model.current_task_status
                },

                #[wrap(Some)]
                #[local_ref]
                set_end_widget = progress_label -> gtk::Label {
                    set_margin_start: 16,

                    #[watch]
                    set_visible: !&model.current_task_progress_pulse
                }
            },

            #[local_ref]
            progress_bar -> gtk::ProgressBar {
                set_margin_top: 16,

                #[watch]
                set_visible: model.current_task.is_some(),

                set_pulse_step: 0.25 / UPDATER_TIMEOUT.as_millis() as f64 // 0.0125
            },

            gtk::Label {
                set_halign: gtk::Align::Start,

                set_margin_top: 16,

                #[watch]
                set_visible: model.current_task.is_some(),

                // TODO: update avg-s with some timeout

                #[watch]
                set_label: &match &model.current_task {
                    Some(task) if task.get_current() > 0 => {
                        let elapsed_time  = (Instant::now() - model.current_task_progress_start).as_secs_f64();
                        let average_speed = (task.get_current() as f64 / elapsed_time).ceil() as u64;

                        if task.get_total() > 1024 * 512 {
                            tr!("tasks-avg-speed", {
                                "format" = "data",
                                "speed" = pretty_bytes(average_speed)
                            })
                        }

                        else {
                            tr!("tasks-avg-speed", {
                                "format" = "items",
                                "speed" = average_speed
                            })
                        }
                    }

                    _ => String::new()
                }
            },

            gtk::Label {
                set_halign: gtk::Align::Start,

                set_margin_top: 8,

                #[watch]
                set_visible: model.current_task.is_some(),

                #[watch]
                set_label: &match &model.current_task {
                    Some(task) if task.get_current() > 0 => {
                        let elapsed_time = (Instant::now() - model.current_task_progress_start).as_secs_f64();
                        let progress = task.get_progress();

                        let expected_total_time = (elapsed_time / progress).ceil() as u64;
                        let remaining_time = expected_total_time - elapsed_time as u64;

                        if remaining_time < 24 * 60 * 60 {
                            tr!("tasks-avg-eta", {
                                "eta" = pretty_seconds(remaining_time)
                            })
                        }

                        else {
                            String::new()
                        }
                    }

                    _ => String::new()
                }
            },

            gtk::ScrolledWindow {
                set_margin_top: 64,

                model.queued_tasks_factory.widget(),
            }
        }
    }

    async fn init(init: Self::Init, root: Self::Root, _sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let flow_box = gtk::FlowBox::new();

        flow_box.set_valign(gtk::Align::End);
        flow_box.set_selection_mode(gtk::SelectionMode::None);

        flow_box.set_vexpand(true);
        flow_box.set_homogeneous(true);

        let model = Self {
            current_task: None,

            current_task_card: CardComponent::builder()
                .launch(init)
                .detach(),

            current_task_status: String::new(),
            current_task_progress_start: Instant::now(),
            current_task_progress_pulse: false,

            queued_tasks_factory: FactoryVecDeque::builder()
                .launch(flow_box)
                .detach(),

            queued_tasks: VecDeque::new(),

            progress_label: gtk::Label::new(None),
            progress_bar: gtk::ProgressBar::new(),

            updater: None
        };

        model.current_task_card.emit(CardComponentInput::SetWidth(160));
        model.current_task_card.emit(CardComponentInput::SetHeight(224));

        model.current_task_card.emit(CardComponentInput::SetClickable(false));
        model.current_task_card.emit(CardComponentInput::SetDisplayTitle(false));

        let progress_label = &model.progress_label;
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
                            self.current_task_card.emit(CardComponentInput::SetInfo(task.get_info()));

                            self.current_task = Some(task);
                            self.current_task_progress_start = Instant::now();
                        }

                        Err(err) => {
                            sender.output(TasksQueueComponentOutput::ShowToast {
                                title: tr!("tasks-resolve-queued-failed"),
                                message: Some(err.to_string())
                            }).unwrap();
                        }
                    }
                }

                else {
                    self.queued_tasks_factory.guard().push_back(task.get_info().to_owned());

                    self.queued_tasks.push_back(task);
                }

                // This will try to start an updater even if one is already running
                // Adding a check here (e.g. is_none()) may lead to a race condition
                sender.input(TasksQueueComponentInput::StartUpdater);
            }

            TasksQueueComponentInput::UpdateCurrentTask => {
                if let Some(task) = &mut self.current_task {
                    if task.is_finished() {
                        if let Err(err) = task.get_status() {
                            sender.output(TasksQueueComponentOutput::ShowToast {
                                title: tr!("tasks-get-status-failed", {
                                    "game-title" = task.get_info().get_title().to_string()
                                }),
                                message: Some(err.to_string())
                            }).unwrap();
                        }

                        let mut is_task_queued = false;

                        for queued_task in &self.queued_tasks {
                            if queued_task.get_info() == task.get_info() {
                                is_task_queued = true;

                                break;
                            }
                        }

                        if !is_task_queued {
                            sender.output(TasksQueueComponentOutput::TaskFinished(task.get_info().to_owned())).unwrap();
                        }

                        if let Some(queued_task) = self.queued_tasks.pop_front() {
                            self.queued_tasks_factory.guard().pop_front();

                            match queued_task.resolve() {
                                Ok(task) => {
                                    self.current_task_card.emit(CardComponentInput::SetInfo(task.get_info().to_owned()));

                                    self.current_task = Some(task);
                                }

                                Err(err) => {
                                    sender.output(TasksQueueComponentOutput::ShowToast {
                                        title: tr!("tasks-resolve-queued-failed"),
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
                            let progress = task.get_progress();

                            self.progress_label.set_text(&format!("{:.2}%", progress * 100.0));
                            self.progress_bar.set_fraction(progress);
                        }

                        if let Ok(status) = task.get_status() {
                            let (pulse, title) = match status {
                                TaskStatus::Pending => (true, tr!("tasks-pending")),

                                TaskStatus::PreparingTransition => (true, tr!("tasks-preparing-transition")),
                                TaskStatus::FinishingTransition => (true, tr!("tasks-finishing-transition")),

                                TaskStatus::Downloading     => (false, tr!("tasks-downloading")),
                                TaskStatus::Unpacking       => (false, tr!("tasks-unpacking")),
                                TaskStatus::StreamUnpacking => (false, tr!("tasks-stream-unpacking")),

                                TaskStatus::DeletingFiles => (true, tr!("tasks-deleting-files")),

                                TaskStatus::RunPreTransitionCode  => (true, tr!("tasks-pre-transition-code")),
                                TaskStatus::RunTransitionCode     => (true, tr!("tasks-transition-code")),
                                TaskStatus::RunPostTransitionCode => (true, tr!("tasks-post-transition-code")),

                                TaskStatus::CreatingPrefix  => (true, tr!("tasks-creating-prefix")),
                                TaskStatus::InstallingDxvk  => (true, tr!("tasks-installing-dxvk")),
                                TaskStatus::InstallingFonts => (false, tr!("tasks-installing-fonts")),

                                TaskStatus::VerifyingFiles => (false, tr!("tasks-verifying-files")),
                                TaskStatus::RepairingFiles => (false, tr!("tasks-repairing-files")),

                                TaskStatus::Finished => (true, tr!("tasks-finished"))
                            };

                            self.current_task_progress_pulse = pulse;
                            self.current_task_status = title;
                        }
                    }
                }
            }

            TasksQueueComponentInput::StartUpdater => {
                let running = Arc::new(AtomicBool::new(true));

                self.updater = Some(TasksQueueProgressUpdater {
                    running: running.clone(),

                    thread: std::thread::spawn(move || {
                        while running.load(Ordering::Relaxed) {
                            sender.input(TasksQueueComponentInput::UpdateCurrentTask);

                            std::thread::sleep(UPDATER_TIMEOUT);
                        }
                    })
                });
            }

            TasksQueueComponentInput::StopUpdater => {
                self.updater = None;
            }
        }
    }
}
