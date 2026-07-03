// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-locale
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

use std::str::FromStr;
use std::sync::RwLock;

pub use unic_langid;

use unic_langid::LanguageIdentifier;

pub mod string;
pub mod bundle;

lazy_static::lazy_static! {
    pub static ref ENGLISH_LANG: LanguageIdentifier = LanguageIdentifier::from_str("en")
        .expect("failed to parse English language code");

    pub static ref SYSTEM_LANG: LanguageIdentifier = {
        let lang = std::env::var("AGL_LOCALE").ok()
            .or_else(|| std::env::var("AGL_LANG").ok())
            .or_else(|| std::env::var("AGL_LANGUAGE").ok())
            .or_else(|| std::env::var("LANG").ok())
            .or_else(|| std::env::var("LANGUAGE").ok())
            .or_else(|| std::env::var("LC_MESSAGES").ok())
            .or_else(|| std::env::var("LC_ALL").ok())
            .unwrap_or_else(|| String::from("en"));

        // Strip down unnecessary info ("en_US.UTF-8").
        let (lang, _) = lang.split_once('.')
            .unwrap_or((lang.as_str(), ""));

        lang.parse::<LanguageIdentifier>()
            .or_else(|_| {
                // "en-us", "en_US", ..
                lang.chars()
                    .take(5)
                    .collect::<String>()
                    .parse::<LanguageIdentifier>()
            })
            .or_else(|_| {
                // "en"
                lang.chars()
                    .take(2)
                    .collect::<String>()
                    .parse::<LanguageIdentifier>()
            })
            .unwrap_or_else(|_| ENGLISH_LANG.clone())
    };

    pub static ref DEFAULT_BUNDLE: RwLock<bundle::LocalizationBundle> = {
        RwLock::new(bundle::LocalizationBundle::default())
    };
}

/// Include translations file in the `DEFAULT_BUNDLE`.
#[macro_export]
macro_rules! include_i18n {
    ($($path:expr $(,)*)+) => {
        let mut lock = $crate::DEFAULT_BUNDLE.write()
            .expect("failed to lock default translations bundle");

        $(
            lock.load_str(include_str!($path))
                .expect("failed to load embedded translations TOML file");
        )+

        drop(lock);
    };
}

/// Get translation string from default translations bundle.
///
/// - `i18n("string_key") -> Option<&str>`
/// - `i18n("string_key", { arg => "value", ... }) -> Option<String>`
/// - `i18n(lang, "string_key") -> Option<&str>`
/// - `i18n(lang, "string_key", { arg => "value", ... }) -> Option<String>`
#[macro_export]
macro_rules! i18n {
    ($key:expr) => {
        $crate::DEFAULT_BUNDLE.read()
            .expect("failed to lock default translations bundle")
            .get_translation(
                $key,
                $crate::SYSTEM_LANG.as_ref()
            )
    };

    ($key:expr, {$( $arg_key:expr => $arg_value:expr $(,)* )+}) => {
        {
            let mut args = std::collections::HashMap::new();

            $( args.insert(stringify!($arg_key), $arg_value.to_string()); )+

            $crate::DEFAULT_BUNDLE.read()
                .expect("failed to lock default translations bundle")
                .get_translation_with_args(
                    $key,
                    $crate::SYSTEM_LANG.as_ref(),
                    args
                )
        }
    };

    ($lang:expr, $key:expr) => {
        $crate::DEFAULT_BUNDLE.read()
            .expect("failed to lock default translations bundle")
            .get_translation($key, $lang)
    };

    ($lang:expr, $key:expr, {$( $arg_key:expr => $arg_value:expr $(,)* )+}) => {
        {
            let mut args = std::collections::HashMap::new();

            $( args.insert(stringify!($arg_key), $arg_value.to_string()); )+

            $crate::DEFAULT_BUNDLE.read()
                .expect("failed to lock default translations bundle")
                .get_translation_with_args($key, $lang, args)
        }
    };
}
