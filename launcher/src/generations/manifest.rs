use std::collections::HashMap;

use serde_json::{json, Value as Json};

use crate::core::prelude::*;
use crate::packages::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    /// Format of the generations list.
    pub format: u64,

    /// Table of generations.
    pub generations: HashMap<Hash, u64>
}

impl Default for Manifest {
    #[inline]
    fn default() -> Self {
        Self {
            format: 1,
            generations: HashMap::new()
        }
    }
}

impl AsJson for Manifest {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "format": self.format,

            "generations": self.generations.iter()
                .map(|(k, v)| (k.to_base32(), v))
                .collect::<HashMap<_, _>>()
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            format: json.get("format")
                .ok_or_else(|| AsJsonError::FieldNotFound("format"))?
                .as_u64()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("format"))?,

            generations: json.get("generations")
                .ok_or_else(|| AsJsonError::FieldNotFound("generations"))?
                .as_object()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("generations"))?
                .iter()
                .map(|(k, v)| {
                    Hash::from_base32(k)
                        .and_then(|k| {
                            v.as_u64()
                                .map(|v| (k, v))
                        })
                })
                .collect::<Option<HashMap<_, _>>>()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("generations"))?
        })
    }
}

impl AsHash for Manifest {
    #[inline]
    fn hash(&self) -> Hash {
        self.format.hash().chain(self.generations.hash())
    }
}
