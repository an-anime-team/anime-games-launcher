// SPDX-License-Identifier: GPL-3.0-or-later
//
// anime-games-launcher
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

/// Generate pretty time output from given seconds.
///
/// ```
/// assert_eq!(pretty_seconds(1),    "00:00:01");
/// assert_eq!(pretty_seconds(60),   "00:01:00");
/// assert_eq!(pretty_seconds(3600), "01:00:00");
/// ```
pub fn pretty_seconds(mut seconds: u64) -> String {
    let hours = seconds / 3600;

    seconds -= hours * 3600;

    let hours = if hours < 10 {
        format!("0{hours}")
    } else {
        hours.to_string()
    };

    let minutes = seconds / 60;

    seconds -= minutes * 60;

    let minutes = if minutes < 10 {
        format!("0{minutes}")
    } else {
        minutes.to_string()
    };

    let seconds = if seconds < 10 {
        format!("0{seconds}")
    } else {
        seconds.to_string()
    };

    format!("{hours}:{minutes}:{seconds}")
}
