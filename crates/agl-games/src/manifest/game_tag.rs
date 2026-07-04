// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-games
// Copyright (C) 2025 - 2026  Nikita Podvirnyi <krypt0nn@dawn.wine>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GameTag {
    /// Game is free to play.
    FreeToPlay,

    /// Game has built-in multiplayer (cooperative) elements.
    Cooperative,

    /// Game has social features such as text, voice or video chats.
    SocialFeatures,

    /// Game has controllers support.
    ControllerSupport,

    /// Game has scenes of gambling or has game mechanics related to gambling.
    Gambling,

    /// Game accepts real money for in-game content.
    InGamePurchases,

    /// Graphic violence generally consists of any clear and uncensored
    /// depiction of various violent acts. Commonly included depictions include
    /// murder, assault with a deadly weapon, accidents which result in death or
    /// severe injury, suicide, and torture. In all cases, it is the
    /// explicitness of the violence and the injury inflicted which results in
    /// it being labeled "graphic".
    ///
    /// In fictional depictions, appropriately realistic plot elements are
    /// usually included to heighten the sense of realism (i.e. blood effects,
    /// prop weapons, CGI).
    ///
    /// Source: https://en.wikipedia.org/wiki/Graphic_violence
    GraphicViolence,

    /// Game has adult content.
    AdultContent,

    /// Another tag.
    Other(String)
}

impl std::fmt::Display for GameTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FreeToPlay        => f.write_str("free-to-play"),
            Self::Cooperative       => f.write_str("cooperative"),
            Self::SocialFeatures    => f.write_str("social-features"),
            Self::ControllerSupport => f.write_str("controller-support"),
            Self::Gambling          => f.write_str("gambling"),
            Self::InGamePurchases   => f.write_str("in-game-purchases"),
            Self::GraphicViolence   => f.write_str("graphic-violence"),
            Self::AdultContent      => f.write_str("adult-content"),

            Self::Other(tag) => f.write_str(tag)
        }
    }
}

impl std::str::FromStr for GameTag {
    type Err = ();

    fn from_str(tag: &str) -> Result<Self, Self::Err> {
        match tag {
            "free-to-play" | "free" | "f2p" => Ok(Self::FreeToPlay),

            "cooperative" | "coop" => Ok(Self::Cooperative),

            "social-features" | "social" => Ok(Self::SocialFeatures),

            "controller-support" | "controller" => Ok(Self::ControllerSupport),

            "gambling" => Ok(Self::Gambling),

            "in-game-purchases" | "purchases" | "payments" => Ok(Self::InGamePurchases),

            "graphic-violence" | "violence" => Ok(Self::GraphicViolence),

            "adult-content" | "adult" | "R18" | "18+" => Ok(Self::AdultContent),

            _ => Err(())
        }
    }
}

impl From<String> for GameTag {
    fn from(tag: String) -> Self {
        tag.parse().unwrap_or(Self::Other(tag))
    }
}

impl From<&'_ str> for GameTag {
    fn from(tag: &str) -> Self {
        tag.parse().unwrap_or_else(|_| Self::Other(tag.to_string()))
    }
}
