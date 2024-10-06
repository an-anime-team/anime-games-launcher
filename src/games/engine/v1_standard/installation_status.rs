use mlua::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstallationStatus {
    /// Latest component version is installed.
    Installed,

    /// Component is installed but there's an
    /// optional update available.
    UpdateAvailable,

    /// Component is installed but there's an update
    /// available that must be installed.
    UpdateRequired,

    /// Component is installed but there's an update
    /// which cannot be installed automatically.
    UpdateUnavailable,

    /// Component is not installed.
    NotInstalled
}

impl std::fmt::Display for InstallationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Installed         => write!(f, "installed"),
            Self::UpdateAvailable   => write!(f, "update-available"),
            Self::UpdateRequired    => write!(f, "update-required"),
            Self::UpdateUnavailable => write!(f, "update-unavailable"),
            Self::NotInstalled      => write!(f, "not-installed")
        }
    }
}

impl std::str::FromStr for InstallationStatus {
    type Err = LuaError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "installed"          => Ok(Self::Installed),
            "update-available"   => Ok(Self::UpdateAvailable),
            "update-required"    => Ok(Self::UpdateRequired),
            "update-unavailable" => Ok(Self::UpdateUnavailable),
            "not-installed"      => Ok(Self::NotInstalled),

            _ => Err(LuaError::external(format!("invalid installation status: {s}")))
        }
    }
}
