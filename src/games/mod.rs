use std::collections::HashMap;
use std::process::Command;
use std::path::PathBuf;

use anime_game_core::game::diff::DiffExt;
use anime_game_core::updater::UpdaterExt;

use crate::components::wine::Wine;

use crate::ui::components::game_card::CardVariant;

use crate::ui::components::tasks_queue::{
    QueuedTask,
    ResolvedTask,
    TaskStatus
};

pub mod integrations;
pub mod genshin;

pub trait RunGameExt {
    /// Get game binary path
    fn get_game_binary(&self) -> &'static str;

    /// Deploy game folder and return path to it using game files driver
    fn deploy_game_folder(&self) -> anyhow::Result<PathBuf>;

    /// Dismantle deployed game folder using game files driver
    fn dismantle_game_folder(&self) -> anyhow::Result<()>;

    /// Get user-defined environment values
    fn get_user_environment(&self) -> HashMap<String, String>;

    /// Run the game in current thread and wait until it's closed
    fn run(&self) -> anyhow::Result<()> {
        let command = [
            format!("{:?}", Wine::from_config()?.get_executable()),
            self.get_game_binary().to_string()
        ];

        let game_folder = self.deploy_game_folder()?;

        Command::new("bash")
            .arg("-c")
            .arg(command.join(" "))
            .envs(self.get_user_environment())
            .current_dir(game_folder)
            .spawn()?
            .wait()?;

        self.dismantle_game_folder()?;

        Ok(())
    }
}

type GetStatusFn<Updater> = dyn Fn(
    Result<<Updater as UpdaterExt>::Status,
    &<Updater as UpdaterExt>::Error>
) -> anyhow::Result<TaskStatus> + Send + 'static;

pub struct DownloadDiffQueuedTask<Diff: DiffExt, Updater: UpdaterExt> {
    pub variant: CardVariant,
    pub diff: Diff,
    pub get_status: Box<GetStatusFn<Updater>>
}

impl<Diff: DiffExt, Updater: UpdaterExt> std::fmt::Debug for DownloadDiffQueuedTask<Diff, Updater> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DownloadDiffQueuedTask")
            .field("variant", &self.variant.get_title())
            .finish()
    }
}

impl<Diff, Updater> QueuedTask for DownloadDiffQueuedTask<Diff, Updater>
where
    Diff: DiffExt<Updater = Updater> + Send,
    Updater: UpdaterExt + Send + 'static
{
    #[inline]
    fn get_variant(&self) -> CardVariant {
        self.variant.clone()
    }

    #[inline]
    fn get_title(&self) -> &str {
        self.variant.get_title()
    }

    #[inline]
    fn get_author(&self) -> &str {
        self.variant.get_author()
    }

    fn resolve(self: Box<Self>) -> anyhow::Result<Box<dyn ResolvedTask>> {
        let Some(updater) = self.diff.install() else {
            anyhow::bail!("Queued diff cannot be installed");
        };

        Ok(Box::new(DownloadDiffResolvedTask {
            variant: self.variant,
            updater,
            get_status: self.get_status
        }))
    }
}

pub struct DownloadDiffResolvedTask<Updater: UpdaterExt> {
    pub variant: CardVariant,
    pub updater: Updater,
    pub get_status: Box<GetStatusFn<Updater>>
}

impl<Updater: UpdaterExt> std::fmt::Debug for DownloadDiffResolvedTask<Updater> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DownloadDiffResolvedTask")
            .field("variant", &self.variant.get_title())
            .finish()
    }
}

impl<Updater: UpdaterExt + Send> ResolvedTask for DownloadDiffResolvedTask<Updater> {
    #[inline]
    fn get_variant(&self) -> CardVariant {
        self.variant.clone()
    }

    #[inline]
    fn get_title(&self) -> &str {
        self.variant.get_title()
    }

    #[inline]
    fn get_author(&self) -> &str {
        self.variant.get_author()
    }

    #[inline]
    fn is_finished(&mut self) -> bool {
        self.updater.is_finished()
    }

    #[inline]
    fn get_current(&self) -> u64 {
        self.updater.current()
    }

    #[inline]
    fn get_total(&self) -> u64 {
        self.updater.total()
    }

    #[inline]
    fn get_progress(&self) -> f64 {
        self.updater.progress()
    }

    #[inline]
    fn get_status(&mut self) -> anyhow::Result<TaskStatus> {
        (self.get_status)(self.updater.status())
    }
}
