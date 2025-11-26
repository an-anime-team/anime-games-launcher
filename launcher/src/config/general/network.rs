use std::time::Duration;
use std::str::FromStr;

use serde::{Serialize, Deserialize};
use serde_json::{json, Value as Json};

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Network {
    pub proxy: Option<Proxy>,
    pub timeout: u64
}

impl Default for Network {
    fn default() -> Self {
        Self {
            proxy: None,
            timeout: 5000
        }
    }
}

impl Network {
    #[inline]
    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout)
    }

    pub fn builder(&self) -> reqwest::Result<reqwest::ClientBuilder> {
        let mut builder = reqwest::Client::builder()
            .connect_timeout(self.timeout());

        if let Some(proxy) = &self.proxy {
            builder = builder.proxy(proxy.proxy()?);
        }

        Ok(builder)
    }
}

impl AsJson for Network {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "proxy": self.proxy.as_ref()
                .map(Proxy::to_json)
                .transpose()?,

            "timeout": self.timeout
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        let default = Self::default();

        Ok(Self {
            proxy: json.get("proxy")
                .ok_or_else(|| AsJsonError::FieldNotFound("general.network.proxy"))
                .and_then(|proxy| {
                    if proxy.is_null() {
                        Ok(None)
                    } else {
                        Proxy::from_json(proxy).map(Some)
                    }
                })
                .unwrap_or(default.proxy),

            timeout: json.get("timeout")
                .and_then(Json::as_u64)
                .unwrap_or(default.timeout)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Proxy {
    pub address: String,
    pub mode: ProxyMode
}

impl Default for Proxy {
    #[inline]
    fn default() -> Self {
        Self {
            address: String::from("socks5://127.0.0.1:9050"),
            mode: ProxyMode::All
        }
    }
}

impl Proxy {
    pub fn proxy(&self) -> reqwest::Result<reqwest::Proxy> {
        match self.mode {
            ProxyMode::All   => reqwest::Proxy::all(&self.address),
            ProxyMode::Http  => reqwest::Proxy::http(&self.address),
            ProxyMode::Https => reqwest::Proxy::https(&self.address)
        }
    }
}

impl AsJson for Proxy {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "address": self.address,
            "mode": self.mode
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        let default = Self::default();

        Ok(Self {
            address: json.get("address")
                .and_then(Json::as_str)
                .map(String::from)
                .unwrap_or(default.address),

            mode: json.get("mode")
                .and_then(Json::as_str)
                .ok_or_else(|| AsJsonError::InvalidFieldValue("general.network.proxy.mode"))
                .and_then(ProxyMode::from_str)
                .unwrap_or(default.mode)
        })
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProxyMode {
    #[default]
    All,
    Http,
    Https
}

impl std::fmt::Display for ProxyMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::All   => write!(f, "all"),
            Self::Http  => write!(f, "http"),
            Self::Https => write!(f, "https")
        }
    }
}

impl FromStr for ProxyMode {
    type Err = AsJsonError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "all"   => Ok(Self::All),
            "http"  => Ok(Self::Http),
            "https" => Ok(Self::Https),

            _ => Err(AsJsonError::InvalidFieldValue("<proxy mode>"))
        }
    }
}
