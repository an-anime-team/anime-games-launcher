use crate::packages::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameTag {
    /// Game has a scenes of gambling or has game mechanics
    /// related to gambling (wishes, banners, etc.)
    Gambling,

    /// Game can accept real money for in-game content.
    Payments,

    /// Game contains scenes of violence.
    Violence,

    /// Game is known to have a bad performance, either
    /// on any platform or on linux specifically
    /// (perhaps due to bad DXVK/wine/gstreamer implementation).
    PerformanceIssues,

    /// Game has an anti-cheat, either server- or client-side.
    /// This tag doesn't necessary mean that this anti-cheat
    /// doesn't support linux platform.
    AntiCheat,

    /// Game is not officially supported on linux.
    UnsupportedPlatform,

    /// Game is not runnable on linux, but the integration package
    /// provides set of special utilities or game files modifications
    /// which make the game to function. Note that this may violate its
    /// terms of service and result in taking actions on your account.
    CompatibilityLayer
}

impl std::fmt::Display for GameTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Gambling            => write!(f, "gambling"),
            Self::Payments            => write!(f, "payments"),
            Self::Violence            => write!(f, "violence"),
            Self::PerformanceIssues   => write!(f, "performance-issues"),
            Self::AntiCheat           => write!(f, "anti-cheat"),
            Self::UnsupportedPlatform => write!(f, "unsupported-platform"),
            Self::CompatibilityLayer  => write!(f, "compatibility-layer")
        }
    }
}

impl std::str::FromStr for GameTag {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "gambling"             => Ok(Self::Gambling),
            "payments"             => Ok(Self::Payments),
            "violence"             => Ok(Self::Violence),
            "performance-issues"   => Ok(Self::PerformanceIssues),
            "anti-cheat"           => Ok(Self::AntiCheat),
            "unsupported-platform" => Ok(Self::UnsupportedPlatform),
            "compatibility-layer"  => Ok(Self::CompatibilityLayer),

            _ => anyhow::bail!("Unsupported game tag: {s}")
        }
    }
}

impl AsHash for GameTag {
    #[inline]
    fn hash(&self) -> Hash {
        self.to_string().hash()
    }
}
