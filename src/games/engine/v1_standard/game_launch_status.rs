use mlua::prelude::*;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameLaunchStatus {
    #[default]
    Normal,

    Warning,
    Dangerous,
    Disabled
}

impl std::fmt::Display for GameLaunchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal    => write!(f, "normal"),
            Self::Warning   => write!(f, "warning"),
            Self::Dangerous => write!(f, "dangerous"),
            Self::Disabled  => write!(f, "disabled")
        }
    }
}

impl std::str::FromStr for GameLaunchStatus {
    type Err = LuaError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "normal"    => Ok(Self::Normal),
            "warning"   => Ok(Self::Warning),
            "dangerous" => Ok(Self::Dangerous),
            "disabled"  => Ok(Self::Disabled),

            _ => Err(LuaError::external(format!("invalid game launch status: {s}")))
        }
    }
}
