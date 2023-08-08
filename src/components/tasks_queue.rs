use std::collections::VecDeque;

use relm4::factory::FactoryVecDeque;
use relm4::prelude::*;
use relm4::component::*;

use gtk::prelude::*;

use crate::components::game_card::{
    GameCardComponent,
    GameCardComponentInput
};

use crate::components::factory::game_card_tasks::GameCardFactory;
use crate::games::GameVariant;

#[derive(Clone)]
pub enum Task {
    DownloadGame {
        variant: GameVariant,
        // diff: Rc<Box<dyn DiffExt>>
    }
}

impl std::fmt::Debug for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DownloadGame { variant, .. } => {
                f.debug_struct("DownloadGame")
                    .field("variant", variant)
                    .finish()
            }
        }
    }
}

impl Task {
    pub fn get_variant(&self) -> GameVariant {
        match self {
            Self::DownloadGame { variant, .. } => *variant
        }
    }

    /// Get task completion progress
    pub fn get_progress(&self) -> f64 {
        0.7
    }
}

#[derive(Debug)]
pub struct TasksQueueComponent {
    pub current_task_card: AsyncController<GameCardComponent>,
    pub current_task: Option<Task>,

    pub queued_tasks_factory: FactoryVecDeque<GameCardFactory>,
    pub queued_tasks: VecDeque<Task>
}

#[derive(Debug, Clone)]
pub enum TasksQueueComponentInput {
    AddTask(Task)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TasksQueueComponentOutput {
    
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for TasksQueueComponent {
    type Init = GameVariant;
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
                set_halign: gtk::Align::Start,

                set_margin_top: 24,

                add_css_class: "title-4",

                #[watch]
                set_label: &match &model.current_task {
                    Some(task) => format!("Downloading {}", task.get_variant().get_title()),
                    None => String::from("Nothing to do")
                }
            },

            gtk::ProgressBar {
                set_margin_top: 16,
                set_fraction: 0.7,

                #[watch]
                set_visible: model.current_task.is_some(),
            },

            gtk::Label {
                set_halign: gtk::Align::Start,

                set_margin_top: 16,

                #[watch]
                set_visible: model.current_task.is_some(),

                set_label: "Download speed: 20 MB/s"
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

            current_task: None,

            queued_tasks_factory: FactoryVecDeque::new(flow_box, sender.input_sender()),
            queued_tasks: VecDeque::new()
        };

        model.current_task_card.emit(GameCardComponentInput::SetWidth(160));
        model.current_task_card.emit(GameCardComponentInput::SetHeight(224));
        model.current_task_card.emit(GameCardComponentInput::SetClickable(false));
        model.current_task_card.emit(GameCardComponentInput::SetDisplayTitle(false));

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>) {
        match msg {
            TasksQueueComponentInput::AddTask(task) => {
                if self.current_task.is_none() {
                    self.current_task_card.emit(GameCardComponentInput::SetVariant(task.get_variant()));

                    self.current_task = Some(task);
                }

                else {
                    self.queued_tasks_factory.guard().push_back(task.get_variant());

                    self.queued_tasks.push_back(task);
                }
            }
        }
    }
}
