use crate::resource;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CardVariant {
    Genshin,
    Honkai,
    StarRail,
    PGR,

    Component {
        title: String,
        author: String
    }
}

impl CardVariant {
    #[inline]
    pub fn games() -> &'static [Self] {
        &[
            Self::Genshin,
            Self::Honkai,
            Self::StarRail,
            Self::PGR
        ]
    }

    #[inline]
    pub fn get_image(&self) -> String {
        match self {
            Self::Genshin  => resource!("images/games/genshin/card"),
            Self::Honkai   => resource!("images/games/honkai/card"),
            Self::StarRail => resource!("images/games/star-rail/card"),
            Self::PGR      => resource!("images/games/pgr/card"),

            Self::Component { .. } => resource!("images/component.png")
        }
    }

    #[inline]
    pub fn get_title(&self) -> &str {
        match self {
            Self::Genshin  => "Genshin Impact",
            Self::Honkai   => "Honkai Impact 3rd",
            Self::StarRail => "Honkai: Star Rail",
            Self::PGR      => "Punishing: Gray Raven",

            Self::Component { title, .. } => title
        }
    }

    #[inline]
    pub fn get_author(&self) -> &str {
        match self {
            Self::Genshin |
            Self::Honkai |
            Self::StarRail => "Hoyoverse",

            Self::PGR => "Kuro Game",

            Self::Component { author, .. } => author
        }
    }

    #[inline]
    pub fn get_details_style(&self) -> &str {
        match self {
            Self::Genshin  => "game-details--genshin",
            Self::Honkai   => "game-details--honkai",
            Self::StarRail => "game-details--star-rail",
            Self::PGR      => "game-details--pgr",

            Self::Component { .. } => ""
        }
    }
}
