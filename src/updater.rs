pub enum UpdaterStatus<Output, Error, Status> {
    Pending,
    Working(Status),
    Finished(Result<Output, Error>)
}

pub struct Updater<Output, Error, Status> {
    receiver: flume::Receiver<Status>,
    status: UpdaterStatus<Output, Error, Status>,
    handle: Option<tokio::task::JoinHandle<Result<Output, Error>>>
}

impl<Output, Error, Status> Updater<Output, Error, Status>
where
    Output: Send + Sync + 'static,
    Error: Send + Sync + 'static,
    Status: Send + Sync + 'static
{
    /// Spawn given future in the global runtime
    pub fn spawn<F>(callback: impl FnOnce(flume::Sender<Status>) -> F) -> Self
    where
        F: std::future::Future<Output = Result<Output, Error>> + Send + Sync + 'static
    {
        let (sender, receiver) = flume::unbounded();

        Self {
            receiver,
            status: UpdaterStatus::Pending,
            handle: Some(tokio::spawn(callback(sender)))
        }
    }

    /// Spawn given future in the runtime
    pub fn spawn_in<F>(runtime: &tokio::runtime::Runtime, callback: impl FnOnce(flume::Sender<Status>) -> F) -> Self
    where
        F: std::future::Future<Output = Result<Output, Error>> + Send + Sync + 'static
    {
        let (sender, receiver) = flume::unbounded();

        Self {
            receiver,
            status: UpdaterStatus::Pending,
            handle: Some(runtime.spawn(callback(sender)))
        }
    }

    async fn update(&mut self) -> Result<(), tokio::task::JoinError> {
        if let Some(handle) = self.handle.take() {
            while let Ok(status) = self.receiver.try_recv() {
                self.status = UpdaterStatus::Working(status);
            }

            if handle.is_finished() {
                self.status = UpdaterStatus::Finished(handle.await?);
            }

            else {
                self.handle = Some(handle);
            }
        }

        Ok(())
    }

    /// Fetch latest updater status
    pub async fn status(&mut self) -> Result<&UpdaterStatus<Output, Error, Status>, tokio::task::JoinError> {
        self.update().await?;

        Ok(&self.status)
    }

    /// Check if updater has finished its job
    pub async fn is_finished(&mut self) -> Result<bool, tokio::task::JoinError> {
        self.update().await?;

        Ok(self.handle.is_none())
    }

    /// Join updater, returning its output
    pub async fn join(mut self) -> Result<Result<Output, Error>, tokio::task::JoinError> {
        if let Some(handle) = self.handle.take() {
            handle.await
        }

        else if let UpdaterStatus::Finished(result) = self.status {
            Ok(result)
        }

        else {
            unreachable!("Updater future finished, but its status wasn't saved");
        }
    }

    /// Abort updater future execution
    pub fn abort(mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}
