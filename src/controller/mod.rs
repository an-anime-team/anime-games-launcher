use std::collections::VecDeque;
use std::thread::JoinHandle;

use crate::ui::windows::{
    main::MainAppMsg,
    games_manager::GamesManagerAppMsg
};

pub mod task;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VariantStatus {
    NotQueued,
    Queued,
    Resolved(task::TaskStatus)
}

// What I do here is perhaps not the most clever approach
// but it was the simplest way to do what I want

static mut CONTROLLER: Option<Controller> = None;

pub struct Controller {
    pub queued_tasks: VecDeque<Box<dyn task::QueuedTask>>,
    pub current_task: Option<Box<dyn task::ResolvedTask>>,

    pub worker: JoinHandle<()>,

    pub main_window_sender: Option<relm4::Sender<MainAppMsg>>,
    pub games_manager_window_sender: Option<relm4::Sender<GamesManagerAppMsg>>
}

impl Default for Controller {
    fn default() -> Self {
        Self {
            queued_tasks: VecDeque::new(),
            current_task: None,

            main_window_sender: None,
            games_manager_window_sender: None,

            worker: std::thread::spawn(|| {
                loop {
                    // Yes I understand how bad this is. Please rewrite it in
                    // appropriate rust approach if you know how
                    let controller = Controller::get_mut();

                    if let Some(task) = &mut controller.current_task {
                        if task.is_finished() {
                            controller.current_task = None;
                        }
                    }

                    else if let Some(task) = controller.queued_tasks.pop_front() {
                        // TODO: send messages to update tasks UI

                        match task.resolve() {
                            Ok(task) => {
                                controller.current_task = Some(task);
                            }

                            Err(err) => {
                                // TODO

                                dbg!(err);
                            }
                        }
                    }
                }
            })
        }
    }
}

impl Controller {
    pub fn get<'a>() -> &'a Self {
        unsafe {
            if CONTROLLER.is_none() {
                CONTROLLER = Some(Self::default());
            }

            CONTROLLER.as_ref().unwrap_unchecked()
        }
    }

    pub fn get_mut<'a>() -> &'a mut Self {
        unsafe {
            if CONTROLLER.is_none() {
                CONTROLLER = Some(Self::default());
            }

            CONTROLLER.as_mut().unwrap_unchecked()
        }
    }

    pub fn current_task<'a>() -> Option<&'a dyn task::ResolvedTask> {
        Self::get().current_task.as_deref()
    }

    pub fn current_task_mut<'a>() -> Option<&'a mut Box<dyn task::ResolvedTask>> {
        Self::get_mut().current_task.as_mut()
    }

    pub fn add_task(task: Box<dyn task::QueuedTask>) {
        Self::get_mut().queued_tasks.push_back(task);
    }

    pub fn get_status(variant: task::TaskVariant) -> anyhow::Result<VariantStatus> {
        let controller = Self::get_mut();

        if let Some(task) = &mut controller.current_task {
            if task.get_variant() == &variant {
                return Ok(VariantStatus::Resolved(task.get_status()?));
            }
        }

        for task in &controller.queued_tasks {
            if task.get_variant() == &variant {
                return Ok(VariantStatus::Queued);
            }
        }

        Ok(VariantStatus::NotQueued)
    }

    pub fn register_main_sender(sender: relm4::Sender<MainAppMsg>) {
        Self::get_mut().main_window_sender = Some(sender);
    }

    pub fn register_games_manager_sender(sender: relm4::Sender<GamesManagerAppMsg>) {
        Self::get_mut().games_manager_window_sender = Some(sender);
    }
}
