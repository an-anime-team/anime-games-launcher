use crate::updater::Updater;

use super::HandlerExt;

macro_rules! impl_handler {
    ($name:ident, $protocol:expr) => {
        #[derive(Default, Debug, Clone)]
        pub struct $name {
            client: reqwest::Client
        }

        impl HandlerExt for $name {
            type Output = reqwest::Response;
            type Error = reqwest::Error;
            type Status = ();

            fn protocol(&self) -> &'static str {
                $protocol
            }

            fn handle(&self, uri: &str) -> Updater<Self::Output, Self::Error, Self::Status> {
                let client = self.client.clone();
                let uri = uri.to_string();

                Updater::spawn(|_| async move {
                    client.get(uri).send().await
                })
            }
        }

        impl $name {
            pub fn new(proxy: Option<reqwest::Proxy>, timeout: Option<std::time::Duration>) -> Result<Self, reqwest::Error> {
                let mut client = reqwest::Client::builder();

                if let Some(proxy) = proxy {
                    client = client.proxy(proxy);
                }

                if let Some(timeout) = timeout {
                    client = client.timeout(timeout);
                }

                Ok(Self {
                    client: client.build()?
                })
            }
        }
    }
}

impl_handler!(HttpHandler, "http");
impl_handler!(HttpsHandler, "https");
