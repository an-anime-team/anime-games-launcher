use std::str::FromStr;

use serde::{Serialize, Deserialize};

use crate::prelude::*;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TargetPlatform {
    X86_64_windows,
    X86_64_linux
}

impl TargetPlatform {
    #[inline]
    /// Get list of all available platforms.
    pub const fn list() -> &'static [Self] {
        &[
            Self::X86_64_windows,
            Self::X86_64_linux
        ]
    }

    /// Try to get current platform.
    pub fn current() -> Option<Self> {
        let info = os_info::get();
        let arch = info.architecture()?;

        if info.os_type() == os_info::Type::Windows {
            Self::from_str(&format!("{arch}-windows")).ok()
        } else {
            Self::from_str(&format!("{arch}-linux")).ok()
        }
    }
}

impl std::fmt::Display for TargetPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::X86_64_windows => f.write_str("x86_64-windows"),
            Self::X86_64_linux   => f.write_str("x86_64-linux")
        }
    }
}

impl FromStr for TargetPlatform {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "x86_64-windows" => Ok(Self::X86_64_windows),
            "x86_64-linux"   => Ok(Self::X86_64_linux),

            _ => anyhow::bail!("Unsupported target platform: {s}")
        }
    }
}

impl AsHash for TargetPlatform {
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
        assert_eq!(TargetPlatform::X86_64_windows.hash(), "x86_64-windows".hash());
        assert_eq!(TargetPlatform::X86_64_linux.hash(), "x86_64-linux".hash());

        Ok(())
    }
}
