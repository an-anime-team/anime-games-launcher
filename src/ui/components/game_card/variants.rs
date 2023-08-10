#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardVariant {
    Genshin,
    Honkai,
    StarRail,
    PGR,

    Component {
        title: &'static str,
        author: &'static str
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
    pub fn get_image(&self) -> &'static str {
        match self {
            Self::Genshin  => "images/genshin-cropped.jpg",
            Self::Honkai   => "images/honkai-cropped.jpg",
            Self::StarRail => "images/star-rail-cropped.jpg",
            Self::PGR      => "images/pgr-cropped.jpg",

            Self::Component { .. } => "images/component.png"
        }
    }

    #[inline]
    pub fn get_title(&self) -> &'static str {
        match self {
            Self::Genshin  => "Genshin Impact",
            Self::Honkai   => "Honkai Impact 3rd",
            Self::StarRail => "Honkai: Star Rail",
            Self::PGR      => "Punishing: Gray Raven",

            Self::Component { title, .. } => title
        }
    }

    #[inline]
    pub fn get_author(&self) -> &'static str {
        match self {
            Self::Genshin |
            Self::Honkai |
            Self::StarRail => "Hoyoverse",

            Self::PGR => "Kuro Game",

            Self::Component { author, .. } => author
        }
    }

    #[inline]
    pub fn get_details_style(&self) -> &'static str {
        match self {
            Self::Genshin  => "game-details--genshin",
            Self::Honkai   => "game-details--honkai",
            Self::StarRail => "game-details--star-rail",
            Self::PGR      => "game-details--pgr",

            Self::Component { .. } => ""
        }
    }
}
