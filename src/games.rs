use std::path::PathBuf;

use crate::config;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameVariant {
    Genshin,
    Honkai,
    StarRail,
    PGR
}

impl GameVariant {
    #[inline]
    pub fn list() -> &'static [Self] {
        &[
            Self::Genshin,
            Self::Honkai,
            Self::StarRail,
            Self::PGR
        ]
    }

    #[inline]
    pub fn get_image(&self) -> &'static str {
        match self {
            Self::Genshin  => "images/genshin-cropped.jpg",
            Self::Honkai   => "images/honkai-cropped.jpg",
            Self::StarRail => "images/star-rail-cropped.jpg",
            Self::PGR      => "images/pgr-cropped.jpg"
        }
    }

    #[inline]
    pub fn get_title(&self) -> &'static str {
        match self {
            Self::Genshin  => "Genshin Impact",
            Self::Honkai   => "Honkai Impact 3rd",
            Self::StarRail => "Honkai: Star Rail",
            Self::PGR      => "Punishing: Gray Raven"
        }
    }

    #[inline]
    pub fn get_publisher(&self) -> &'static str {
        match self {
            Self::Genshin | Self::Honkai | Self::StarRail => "Hoyoverse",
            Self::PGR => "Kuro Game"
        }
    }

    #[inline]
    /// Get game code name
    /// 
    /// Example: `honkai-star-rail`
    pub fn get_name(&self) -> &'static str {
        match self {
            Self::Genshin  => "genshin-impact",
            Self::Honkai   => "honkai-impact-3rd",
            Self::StarRail => "honkai-star-rail",
            Self::PGR      => "punishing-gray-raven"
        }
    }

    #[inline]
    pub fn get_details_style(&self) -> &'static str {
        match self {
            Self::Genshin  => "game-details--genshin",
            Self::Honkai   => "game-details--honkai",
            Self::StarRail => "game-details--star-rail",
            Self::PGR      => "game-details--pgr"
        }
    }

    #[inline]
    pub fn get_folder(&self) -> PathBuf {
        let config = config::get();

        config.games.genshin.paths
            .for_edition(config.games.genshin.edition)
            .to_path_buf()
    }
}
