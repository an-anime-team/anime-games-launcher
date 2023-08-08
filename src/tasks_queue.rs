use std::collections::VecDeque;
use std::rc::Rc;

use anime_game_core::game::diff::DiffExt;

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

pub struct TasksQueue {
    queue: VecDeque<Task>
}

impl TasksQueue {
    #[inline]
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new()
        }
    }

    #[inline]
    pub fn push(&mut self, task: impl Into<Task>) {
        self.queue.push_back(task.into());
    }

    #[inline]
    pub fn get_current(&self) -> Option<&Task> {
        self.queue.front()
    }
}
