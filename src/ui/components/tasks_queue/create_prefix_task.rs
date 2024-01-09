use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{cell::Cell, sync::Arc};
use std::path::PathBuf;
use std::thread::JoinHandle;

use wincompatlib::prelude::*;

use anime_game_core::updater::UpdaterExt;

use crate::components::wine::Wine;
use crate::components::dxvk::Dxvk;

use crate::ui::components::game_card::CardInfo;

use super::{QueuedTask, ResolvedTask, TaskStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    CreatingPrefix,
    InstallingDxvk,
    InstallingFonts,
    Finished
}

#[derive(Debug)]
pub struct Updater {
    pub status: Cell<Status>,
    pub current: Cell<u64>,
    pub total: Cell<u64>,

    pub worker: Option<JoinHandle<Result<(), anyhow::Error>>>,
    pub worker_result: Option<Result<(), anyhow::Error>>,
    pub updater: flume::Receiver<(Status, u64, u64)>
}

impl Updater {
    fn update(&self) {
        while let Ok((status, current, total)) = self.updater.try_recv() {
            self.status.set(status);
            self.current.set(current);
            self.total.set(total);
        }
    }
}

impl UpdaterExt for Updater {
    type Error = anyhow::Error;
    type Status = Status;
    type Result = ();

    fn status(&mut self) -> Result<Self::Status, &Self::Error> {
        self.update();

        if let Some(worker) = self.worker.take() {
            if !worker.is_finished() {
                self.worker = Some(worker);

                return Ok(self.status.get());
            }

            self.worker_result = Some(worker.join().expect("Failed to join prefix creation thread"));
        }

        match &self.worker_result {
            Some(Ok(_)) => Ok(self.status.get()),
            Some(Err(err)) => Err(err),

            None => unreachable!()
        }
    }

    fn wait(mut self) -> Result<Self::Result, Self::Error> {
        if let Some(worker) = self.worker.take() {
            return worker.join().expect("Failed to join prefix creation thread");
        }

        else if let Some(result) = self.worker_result.take() {
            return result;
        }

        unreachable!()
    }

    #[inline]
    fn is_finished(&mut self) -> bool {
        matches!(self.status(), Ok(Status::Finished) | Err(_))
    }

    #[inline]
    fn current(&self) -> u64 {
        self.update();

        self.current.get()
    }

    #[inline]
    fn total(&self) -> u64 {
        self.update();

        self.total.get()
    }
}

#[derive(Debug)]
pub struct CreatePrefixQueuedTask {
    pub path: PathBuf,
    pub install_corefonts: bool
}

impl QueuedTask for CreatePrefixQueuedTask {
    fn get_info(&self) -> CardInfo {
        CardInfo::Component {
            name: String::from("wine-prefix"),
            title: String::from("Wine prefix"),
            developer: String::new()
        }
    }

    fn resolve(self: Box<Self>) -> anyhow::Result<Box<dyn ResolvedTask>> {
        let (sender, receiver) = flume::unbounded();

        let Some(wine) = Wine::from_config()?.to_wincompatlib() else {
            anyhow::bail!("Failed to resolve wincompatlib wine descriptor");
        };

        Ok(Box::new(CreatePrefixResolvedTask {
            updater: Updater {
                status: Cell::new(Status::CreatingPrefix),
                current: Cell::new(0),
                total: Cell::new(1), // To prevent division by 0

                worker_result: None,
                updater: receiver,

                worker: Some(std::thread::spawn(move || -> anyhow::Result<()> {
                    // Specify basic wine params

                    let wine = wine.with_arch(WineArch::Win64)
                        .with_prefix(&self.path)
                        .with_loader(WineLoader::Current);

                    // Create wine prefix

                    sender.send((Status::CreatingPrefix, 0, 1))?;

                    if self.path.exists() {
                        wine.update_prefix(None::<&str>)?;
                    } else {
                        wine.init_prefix(None::<&str>)?;
                    }

                    sender.send((Status::CreatingPrefix, 1, 1))?;

                    // Apply DXVK

                    sender.send((Status::InstallingDxvk, 0, 1))?;

                    let dxvk = Dxvk::from_config()?;

                    wine.install_dxvk(dxvk.get_folder(), InstallParams {
                        repair_dlls: false,
                        ..InstallParams::default()
                    })?;

                    sender.send((Status::InstallingDxvk, 1, 1))?;

                    // Install fonts

                    if self.install_corefonts {
                        let wine_arc = Arc::new(wine);

                        let fonts = Font::iterator().into_iter().collect::<Vec<Font>>();
                        let total_fonts = fonts.len() as u64;

                        let font_queue = Arc::new(Mutex::new(fonts));
                        let installed_fonts = Arc::new(AtomicU64::new(0));

                        // Spawn maximum 8 threads to install all the fonts
                        let threads_count = std::cmp::min(total_fonts, 8);
                        let mut threads = Vec::with_capacity(threads_count as usize);

                        sender.send((Status::InstallingFonts, 0, total_fonts))?;

                        for _ in 0..threads_count {
                            let path = self.path.clone();

                            let wine_arc_copy = wine_arc.clone();
                            let font_queue_copy = font_queue.clone();
                            let installed_fonts_copy = installed_fonts.clone();

                            let sender_copy = sender.clone();

                            threads.push(std::thread::spawn(move || -> anyhow::Result<()> {
                                // Using "while let" here will lead to the first thread locking the queue
                                // for it's entire lifetime, making parallelization useless
                                loop {
                                    let Some(font) = font_queue_copy.lock().unwrap().pop() else {
                                        break;
                                    };

                                    if !font.is_installed(&path) {
                                        wine_arc_copy.as_ref().install_font(font)?;
                                    }

                                    sender_copy.send((
                                        Status::InstallingFonts, 
                                        installed_fonts_copy.fetch_add(1, Ordering::Relaxed) + 1, 
                                        total_fonts
                                    ))?;
                                }

                                Ok(())
                            }));
                        }

                        for thread in threads {
                            thread.join().expect("Failed to join font installing thread")?;
                        }

                        sender.send((Status::InstallingFonts, total_fonts, total_fonts))?;
                    }

                    // Finish downloading

                    sender.send((Status::Finished, 1, 1))?;

                    Ok(())
                }))
            }
        }))
    }
}

#[derive(Debug)]
pub struct CreatePrefixResolvedTask {
    pub updater: Updater
}

impl ResolvedTask for CreatePrefixResolvedTask {
    fn get_info(&self) -> CardInfo {
        CardInfo::Component {
            name: String::from("wine-prefix"),
            title: String::from("Wine prefix"),
            developer: String::new()
        }
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

    fn get_status(&mut self) -> anyhow::Result<TaskStatus> {
        match self.updater.status() {
            Ok(status) => Ok(match status {
                Status::CreatingPrefix  => TaskStatus::CreatingPrefix,
                Status::InstallingDxvk  => TaskStatus::InstallingDxvk,
                Status::InstallingFonts => TaskStatus::InstallingFonts,
                Status::Finished        => TaskStatus::Finished
            }),

            Err(err) => anyhow::bail!(err.to_string())
        }
    }
}
