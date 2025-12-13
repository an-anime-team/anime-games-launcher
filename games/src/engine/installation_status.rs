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
pub enum InstallationStatus {
    /// Latest game version is installed.
    Installed,

    /// Game is installed but there's an optional update available.
    UpdateAvailable,

    /// Game is installed but there's an update available that must be installed.
    UpdateRequired,

    /// Game is not installed.
    NotInstalled
}

impl std::fmt::Display for InstallationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Installed       => f.write_str("installed"),
            Self::UpdateAvailable => f.write_str("update-available"),
            Self::UpdateRequired  => f.write_str("update-required"),
            Self::NotInstalled    => f.write_str("not-installed")
        }
    }
}

impl std::str::FromStr for InstallationStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "installed"        => Ok(Self::Installed),
            "update-available" => Ok(Self::UpdateAvailable),
            "update-required"  => Ok(Self::UpdateRequired),
            "not-installed"    => Ok(Self::NotInstalled),

            _ => Err(())
        }
    }
}
