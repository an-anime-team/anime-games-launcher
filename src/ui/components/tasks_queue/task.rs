use crate::ui::components::game_card::CardInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
/// All the possible tasks statuses in one enum
pub enum TaskStatus {
    Pending,
    PreparingTransition,
    RunPreTransitionCode,
    Downloading,
    Unpacking,
    StreamUnpacking,
    RunTransitionCode,
    FinishingTransition,
    RunPostTransitionCode,
    CreatingPrefix,
    InstallingDxvk,
    InstallingFonts,
    VerifyingFiles,
    RepairingFiles,
    DeletingFiles,
    Finished
}

pub trait QueuedTask: Send + std::fmt::Debug {
    /// Get component info
    fn get_info(&self) -> CardInfo;

    /// Resolve queued task and start downloading stuff
    fn resolve(self: Box<Self>) -> anyhow::Result<Box<dyn ResolvedTask>>;
}

pub trait ResolvedTask: Send + std::fmt::Debug {
    /// Get component info
    fn get_info(&self) -> CardInfo;

    /// Check if the task is finished
    fn is_finished(&mut self) -> bool;

    /// Get current task progress
    fn get_current(&self) -> u64;

    /// Get total task progress
    fn get_total(&self) -> u64;

    /// Get task completion progress
    fn get_progress(&self) -> f64;

    /// Get task status
    fn get_status(&mut self) -> anyhow::Result<TaskStatus>;
}
