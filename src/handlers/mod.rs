use crate::updater::Updater;

pub mod http;
pub mod file;

pub trait HandlerExt {
    type Output;
    type Error;
    type Status;

    fn protocol(&self) -> &'static str;

    /// Handle given URI
    /// 
    /// This method doesn't check if this URI
    /// can be handled by current handler.
    /// It's expected that this is already done
    /// by the user (developer) (you)
    fn handle(&self, uri: &str) -> Updater<Self::Output, Self::Error, Self::Status>;

    #[inline]
    /// Check if current handler can be applied to the given URI
    fn can_handle(&self, uri: &str) -> bool {
        uri.starts_with(&format!("{}://", self.protocol()))
    }
}

pub struct UriHandler {
    http: http::HttpHandler,
    https: http::HttpsHandler,
    file: file::FileHandler
}

impl UriHandler {
    /// Create universal URI handler
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            // TODO: respect proxy and timeout settings
            http: http::HttpHandler::new(None, None)?,
            https: http::HttpsHandler::new(None, None)?,

            file: file::FileHandler
        })
    }

    /// Check if given URI is supported
    pub fn can_handle(&self, uri: impl AsRef<str>) -> bool {
        let uri = uri.as_ref();

        self.https.can_handle(uri) ||
        self.http.can_handle(uri) ||
        self.file.can_handle(uri)
    }

    /// Handle given URI
    pub fn handle(&self, uri: impl AsRef<str>) -> anyhow::Result<Updater<Vec<u8>, anyhow::Error, ()>> {
        let uri = uri.as_ref();

        if self.https.can_handle(uri) {
            let updater = self.https.handle(uri);

            Ok(Updater::spawn(|_| async move {
                Ok(updater.join().await??.bytes().await?.to_vec())
            }))
        }

        else if self.http.can_handle(uri) {
            let updater = self.https.handle(uri);

            Ok(Updater::spawn(|_| async move {
                Ok(updater.join().await??.bytes().await?.to_vec())
            }))
        }

        else if self.file.can_handle(uri) {
            let updater = self.file.handle(uri);

            Ok(Updater::spawn(|_| async move {
                Ok(updater.join().await??)
            }))
        }

        else {
            anyhow::bail!("Can't handle given URI: {uri}");
        }
    }
}
