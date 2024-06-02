use crate::updater::Updater;

use super::HandlerExt;

#[derive(Default, Debug, Clone, Copy)]
pub struct FileHandler;

impl HandlerExt for FileHandler {
    type Output = Vec<u8>;
    type Error = std::io::Error;
    type Status = ();

    fn protocol(&self) -> &'static str {
        "file"
    }

    fn handle(&self, uri: &str) -> Updater<Self::Output, Self::Error, Self::Status> {
        let path = uri.strip_prefix("file://")
            .unwrap_or(uri)
            .to_string();

        Updater::spawn(|_| async move {
            tokio::fs::read(path).await
        })
    }
}
