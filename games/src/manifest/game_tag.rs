// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-games
// Copyright (C) 2025  Nikita Podvirnyi <krypt0nn@vk.com>
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameTag {
    /// Game has scenes of gambling or has game mechanics related to gambling
    /// (wishes, banners, etc.)
    Gambling,

    /// Game can accept real money for in-game content.
    Payments,

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

    /// Game has built-in multiplayer (cooperative) elements.
    Cooperative,

    /// Game has social features - online chat, VoIP, shared spaces, etc.
    Social,

    /// Game has controllers support.
    Controller,

    /// Game is known to have bad performance, either globally across all the
    /// platforms or on the target platform specifically.
    PerformanceIssues,

    /// Game has an anti-cheat, either server- or client-side. This tag doesn't
    /// necessary mean that this anti-cheat doesn't support the target platform.
    AntiCheat,

    /// Game cannot run on some platforms natively, but the integration package
    /// provides set of special utilities or game files modifications which make
    /// the game function. Note that this may violate its terms of service and
    /// result in taking actions on your account.
    Workarounds
}

impl std::fmt::Display for GameTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Gambling            => f.write_str("gambling"),
            Self::Payments            => f.write_str("payments"),
            Self::GraphicViolence     => f.write_str("graphic-violence"),
            Self::Cooperative         => f.write_str("cooperative"),
            Self::Social              => f.write_str("social"),
            Self::Controller          => f.write_str("controller"),
            Self::PerformanceIssues   => f.write_str("performance-issues"),
            Self::AntiCheat           => f.write_str("anti-cheat"),
            Self::Workarounds         => f.write_str("workarounds")
        }
    }
}

impl std::str::FromStr for GameTag {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "gambling"             => Ok(Self::Gambling),
            "payments"             => Ok(Self::Payments),
            "graphic-violence"     => Ok(Self::GraphicViolence),
            "cooperative"          => Ok(Self::Cooperative),
            "social"               => Ok(Self::Social),
            "performance-issues"   => Ok(Self::PerformanceIssues),
            "anti-cheat"           => Ok(Self::AntiCheat),
            "workarounds"          => Ok(Self::Workarounds),

            _ => Err(())
        }
    }
}
