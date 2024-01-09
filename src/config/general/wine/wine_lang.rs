use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

lazy_static::lazy_static! {
    static ref LANGS_INFO: HashMap<WineLang, (&'static str, &'static str)> = HashMap::from([
        (WineLang::System,     ("System",     "")),
        (WineLang::English,    ("English",    "en_US.UTF-8")),
        (WineLang::Russian,    ("Russian",    "ru_RU.UTF-8")),
        (WineLang::German,     ("German",     "de_DE.UTF-8")),
        (WineLang::Portuguese, ("Portuguese", "pt_PT.UTF-8")),
        (WineLang::Polish,     ("Polish",     "pl_PL.UTF-8")),
        (WineLang::French,     ("French",     "fr_FR.UTF-8")),
        (WineLang::Spanish,    ("Spanish",    "es_ES.UTF-8")),
        (WineLang::Chinese,    ("Chinese",    "zh_CN.UTF-8")),
        (WineLang::Japanese,   ("Japanese",   "ja_JP.UTF-8")),
        (WineLang::Korean,     ("Korean",     "ko_KR.UTF-8"))
    ]);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WineLang {
    System,
    English,
    Russian,
    German,
    Portuguese,
    Polish,
    French,
    Spanish,
    Chinese,
    Japanese,
    Korean
}

impl Default for WineLang {
    #[inline]
    fn default() -> Self {
        Self::System
    }
}

impl From<&Json> for WineLang {
    #[inline]
    fn from(value: &Json) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }
}

impl WineLang {
    /// Get available wine languages
    pub fn list() -> Vec<Self> {
        let mut langs = LANGS_INFO.keys()
            .copied()
            .collect::<Vec<_>>();

        langs.sort_by(|a, b| {
            if a.name() == "System" {
                std::cmp::Ordering::Less
            } else if b.name() == "System" {
                std::cmp::Ordering::Greater
            } else {
                a.name().cmp(b.name())
            }
        });

        langs
    }

    /// Get language name
    /// 
    /// `WineLang::English -> English`
    pub fn name(&self) -> &'static str {
        LANGS_INFO[self].0
    }

    /// Get language code
    /// 
    /// `WineLang::English -> en_US.UTF-8`
    pub fn code(&self) -> &'static str {
        LANGS_INFO[self].1
    }

    /// Get environment variables corresponding to used wine language
    pub fn get_env_vars(&self) -> HashMap<&str, &str> {
        if self == &Self::System {
            return HashMap::new();
        }

        HashMap::from([
            ("LANG", self.code()),
            ("LC_ALL", self.code())
        ])
    }
}
