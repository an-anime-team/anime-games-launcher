use serde::{Serialize, Deserialize};

use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlatformFeature {
    WineDxvk
}

impl std::fmt::Display for PlatformFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WineDxvk => write!(f, "wine-dxvk")
        }
    }
}

impl std::str::FromStr for PlatformFeature {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "wine-dxvk" => Ok(Self::WineDxvk),

            _ => anyhow::bail!("Unsupported target platform feature: {s}")
        }
    }
}

impl AsHash for PlatformFeature {
    #[inline]
    fn hash(&self) -> Hash {
        self.to_string().hash()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash() -> anyhow::Result<()> {
        assert_eq!(PlatformFeature::WineDxvk.hash(), "wine-dxvk".hash());

        Ok(())
    }
}
