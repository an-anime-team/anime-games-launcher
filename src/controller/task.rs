#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TaskVariant {
    Genshin,
    Honkai,
    StarRail,
    GrayRaven,

    Component {
        title: String,
        author: String
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// All the possible tasks statuses in one enum
pub enum TaskStatus {
    PreparingTransition,
    Downloading,
    Unpacking,
    FinishingTransition,
    ApplyingHdiffPatches,
    DeletingObsoleteFiles,
    CreatingPrefix,
    InstallingDxvk,
    InstallingFonts,
    Finished
}

pub trait QueuedTask: Send + Sync + std::fmt::Debug {
    /// Get component variant
    fn get_variant(&self) -> &TaskVariant;

    /// Get tasked component title
    fn get_title(&self) -> &str;

    /// Get tasked component author
    fn get_author(&self) -> &str;

    /// Resolve queued task and start downloading stuff
    fn resolve(self: Box<Self>) -> anyhow::Result<Box<dyn ResolvedTask>>;
}

pub trait ResolvedTask: Send + Sync + std::fmt::Debug {
    /// Get component variant
    fn get_variant(&self) -> &TaskVariant;

    /// Get tasked component title
    fn get_title(&self) -> &str;

    /// Get tasked component author
    fn get_author(&self) -> &str;

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
