use std::str::FromStr;

use serde::{Serialize, Deserialize};

use crate::prelude::*;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TargetPlatform {
    X86_64_windows_native,
    X86_64_linux_native,
    X86_64_linux_wine32,
    X86_64_linux_wine64
}

impl TargetPlatform {
    /// Try to get current platform.
    pub fn current() -> Option<Self> {
        let info = os_info::get();

        let arch = info.architecture()?;

        if info.os_type() == os_info::Type::Windows {
            Self::from_str(&format!("{arch}-windows-native")).ok()
        } else {
            Self::from_str(&format!("{arch}-linux-native")).ok()
        }
    }
}

impl std::fmt::Display for TargetPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::X86_64_windows_native => write!(f, "x86_64-windows-native"),
            Self::X86_64_linux_native   => write!(f, "x86_64-linux-native"),
            Self::X86_64_linux_wine32   => write!(f, "x86_64-linux-wine32"),
            Self::X86_64_linux_wine64   => write!(f, "x86_64-linux-wine64")
        }
    }
}

impl FromStr for TargetPlatform {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "x86_64-windows-native" => Ok(Self::X86_64_windows_native),
            "x86_64-linux-native"   => Ok(Self::X86_64_linux_native),
            "x86_64-linux-wine32"   => Ok(Self::X86_64_linux_wine32),
            "x86_64-linux-wine64"   => Ok(Self::X86_64_linux_wine64),

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
        assert_eq!(TargetPlatform::X86_64_windows_native.hash(), "x86_64-windows-native".hash());
        assert_eq!(TargetPlatform::X86_64_linux_wine64.hash(), "x86_64-linux-wine64".hash());

        Ok(())
    }
}
