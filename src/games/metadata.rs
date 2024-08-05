use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use chrono::Datelike;

use crate::{tr, LAUNCHER_FOLDER};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LauncherMetadata {
    pub launches: GameLaunchesMetadata,
}

impl Default for LauncherMetadata {
    #[inline]
    fn default() -> Self {
        Self {
            launches: GameLaunchesMetadata::default(),
        }
    }
}

impl From<&Json> for LauncherMetadata {
    fn from(value: &Json) -> Self {
        let default = Self::default();

        Self {
            launches: value
                .get("launches")
                .map(GameLaunchesMetadata::from)
                .unwrap_or(default.launches),
        }
    }
}

impl LauncherMetadata {
    pub fn load_for_game(game: impl AsRef<str>, edition: impl AsRef<str>) -> anyhow::Result<Self> {
        let path = LAUNCHER_FOLDER
            .join("games")
            .join(game.as_ref())
            .join(edition.as_ref())
            .join("launcher_metadata.json");

        if !path.exists() {
            return Ok(Self::default());
        }

        let value = serde_json::from_slice::<Json>(&std::fs::read(&path)?)?;

        Ok(Self::from(&value))
    }

    pub fn save_for_game(
        &self,
        game: impl AsRef<str>,
        edition: impl AsRef<str>,
    ) -> anyhow::Result<()> {
        let folder_path = LAUNCHER_FOLDER
            .join("games")
            .join(game.as_ref())
            .join(edition.as_ref());

        let file_path = folder_path.join("launcher_metadata.json");

        if !folder_path.exists() {
            std::fs::create_dir_all(&folder_path)?;
        }

        std::fs::write(file_path, serde_json::to_string_pretty(&self)?)?;

        Ok(())
    }

    pub fn get_last_played_text(&self) -> String {
        let Some(last_launch) = self.launches.last_launch else {
            return tr!("details-never");
        };

        let last_launch = chrono::DateTime::from_timestamp(last_launch.stopped_at, 0).unwrap();

        let today = chrono::Utc::now().num_days_from_ce();
        let last_run = last_launch.num_days_from_ce();

        match today - last_run {
            1 => tr!("details-yesterday"),
            0 => tr!("details-today"),

            _ => format!("{}", last_launch.format("%d/%m/%Y")),
        }
    }

    // FIXME: make this more efficient
    pub fn get_total_playtime_text(&self) -> String {
        let mut seconds = self.launches.total_playtime;
        let mut minutes = seconds / 60;

        seconds -= minutes * 60;

        let hours = minutes / 60;

        minutes -= hours * 60;

        if hours > 0 {
            if hours < 999 {
                return format!("{hours} {}", &tr!("details-hours"));
            }

            let hours = hours
                .to_string()
                .as_bytes()
                .rchunks(3)
                .rev()
                .map(std::str::from_utf8)
                .collect::<Result<Vec<&str>, _>>()
                .unwrap()
                .join(",");

            return format!("{hours} {}", &tr!("details-hours"));
        } else if minutes > 0 {
            return format!("{minutes} {}", &tr!("details-minutes"));
        } else if seconds > 0 {
            return format!("{seconds} {}", &tr!("details-seconds"));
        }

        tr!("details-never")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GameLaunchesMetadata {
    pub last_launch: Option<GameLastLaunchMetadata>,
    pub total_playtime: u64,
}

impl Default for GameLaunchesMetadata {
    #[inline]
    fn default() -> Self {
        Self {
            last_launch: None,
            total_playtime: 0,
        }
    }
}

impl From<&Json> for GameLaunchesMetadata {
    fn from(value: &Json) -> Self {
        let default = Self::default();

        Self {
            last_launch: value
                .get("last_launch")
                .map(|value| {
                    if value.is_null() {
                        None
                    } else {
                        Some(GameLastLaunchMetadata::from(value))
                    }
                })
                .unwrap_or(default.last_launch),

            total_playtime: value
                .get("total_playtime")
                .and_then(Json::as_u64)
                .unwrap_or(default.total_playtime),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GameLastLaunchMetadata {
    pub started_at: i64,
    pub stopped_at: i64,
}

impl Default for GameLastLaunchMetadata {
    #[inline]
    fn default() -> Self {
        let now = chrono::Utc::now().timestamp();

        Self {
            started_at: now,
            stopped_at: now,
        }
    }
}

impl From<&Json> for GameLastLaunchMetadata {
    fn from(value: &Json) -> Self {
        let default = Self::default();

        Self {
            started_at: value
                .get("started_at")
                .and_then(Json::as_i64)
                .unwrap_or(default.started_at),

            stopped_at: value
                .get("stopped_at")
                .and_then(Json::as_i64)
                .unwrap_or(default.stopped_at),
        }
    }
}
