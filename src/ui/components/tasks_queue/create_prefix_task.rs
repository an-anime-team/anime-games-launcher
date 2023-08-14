use std::cell::Cell;
use std::thread::JoinHandle;

use anime_game_core::updater::UpdaterExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    CreatingPrefix,
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
