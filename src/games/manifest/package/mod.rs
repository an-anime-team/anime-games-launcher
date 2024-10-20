use std::collections::HashSet;
use std::str::FromStr;

use serde::{Serialize, Deserialize};
use serde_json::{json, Value as Json};

use crate::core::prelude::*;
use crate::packages::prelude::*;

pub mod target_platform;
pub mod platform_feature;

use target_platform::TargetPlatform;
use platform_feature::PlatformFeature;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Package {
    pub url: String,
    pub output: String,
    pub runtime: PackageRuntime
}

impl AsJson for Package {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "url": self.url,
            "output": self.output,
            "runtime": self.runtime.to_json()?
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            url: json.get("url")
                .ok_or_else(|| AsJsonError::FieldNotFound("package.url"))?
                .as_str()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("package.url"))?
                .to_string(),

            output: json.get("output")
                .ok_or_else(|| AsJsonError::FieldNotFound("package.output"))?
                .as_str()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("package.output"))?
                .to_string(),

            runtime: json.get("runtime")
                .ok_or_else(|| AsJsonError::FieldNotFound("package.runtime"))
                .and_then(PackageRuntime::from_json)?
        })
    }
}

impl AsHash for Package {
    #[inline]
    fn hash(&self) -> Hash {
        self.url.hash().chain(self.output.hash())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageRuntime {
    pub platform: TargetPlatform,
    pub features: Option<HashSet<PlatformFeature>>,
    pub supported: Option<Vec<PackageSupportedRuntime>>
}

impl AsJson for PackageRuntime {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "platform": self.platform.to_string(),

            "features": self.features.as_ref()
                .map(|features| {
                    features.iter()
                        .map(|feature| feature.to_string())
                        .collect::<Vec<_>>()
                }),

            "supported": self.supported.as_ref()
                .map(|runtimes| {
                    runtimes.iter()
                        .map(|runtime| runtime.to_json())
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()?
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            platform: json.get("platform")
                .ok_or_else(|| AsJsonError::FieldNotFound("package.runtime.platform"))?
                .as_str()
                .map(TargetPlatform::from_str)
                .ok_or_else(|| AsJsonError::InvalidFieldValue("package.runtime.platform"))?
                .map_err(|err| AsJsonError::Other(err.into()))?,

            features: match json.get("features") {
                Some(features) if features.is_null() => None,

                Some(features) => {
                    features.as_array()
                        .ok_or_else(|| AsJsonError::InvalidFieldValue("package.runtime.features"))?
                        .iter()
                        .map(|feature| {
                            feature.as_str()
                                .ok_or_else(|| anyhow::anyhow!("Invalid target platform feature format"))
                                .and_then(PlatformFeature::from_str)
                        })
                        .collect::<Result<HashSet<_>, _>>()
                        .map_err(|err| AsJsonError::Other(err.into()))
                        .map(Some)?
                }

                None => None
            },

            supported: match json.get("supported") {
                Some(supported) if supported.is_null() => None,

                Some(supported) => {
                    supported.as_array()
                        .ok_or_else(|| AsJsonError::InvalidFieldValue("package.runtime.supported"))?
                        .iter()
                        .map(PackageSupportedRuntime::from_json)
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|err| AsJsonError::Other(err.into()))
                        .map(Some)?
                }

                None => None
            }
        })
    }
}

impl AsHash for PackageRuntime {
    fn hash(&self) -> Hash {
        self.platform.hash()
            .chain(self.features.hash())
            .chain(self.supported.hash())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageSupportedRuntime {
    pub platform: TargetPlatform,
    pub features: Option<HashSet<PlatformFeature>>
}

impl AsJson for PackageSupportedRuntime {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "platform": self.platform.to_string(),

            "features": self.features.as_ref()
                .map(|features| {
                    features.iter()
                        .map(|feature| feature.to_string())
                        .collect::<Vec<_>>()
                })
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            platform: json.get("platform")
                .ok_or_else(|| AsJsonError::FieldNotFound("package.runtime.supported[].platform"))?
                .as_str()
                .map(TargetPlatform::from_str)
                .ok_or_else(|| AsJsonError::InvalidFieldValue("package.runtime.supported[].platform"))?
                .map_err(|err| AsJsonError::Other(err.into()))?,

            features: match json.get("features") {
                Some(features) if features.is_null() => None,

                Some(features) => {
                    features.as_array()
                        .ok_or_else(|| AsJsonError::InvalidFieldValue("package.runtime.supported[].features"))?
                        .iter()
                        .map(|feature| {
                            feature.as_str()
                                .ok_or_else(|| anyhow::anyhow!("Invalid target platform feature format"))
                                .and_then(PlatformFeature::from_str)
                        })
                        .collect::<Result<HashSet<_>, _>>()
                        .map_err(|err| AsJsonError::Other(err.into()))
                        .map(Some)?
                }

                None => None
            }
        })
    }
}

impl AsHash for PackageSupportedRuntime {
    #[inline]
    fn hash(&self) -> Hash {
        self.platform.hash().chain(self.features.hash())
    }
}
