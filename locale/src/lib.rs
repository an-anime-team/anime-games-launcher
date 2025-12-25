// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-locale
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

use std::str::FromStr;

pub use unic_langid;

use unic_langid::LanguageIdentifier;

pub mod string;
pub mod bundle;

lazy_static::lazy_static! {
    pub static ref ENGLISH_LANG: LanguageIdentifier = LanguageIdentifier::from_str("en")
        .expect("failed to parse English language code");

    pub static ref SYSTEM_LANG: LanguageIdentifier = {
        let lang = std::env::var("AGL_LOCALE").ok()
            .or_else(|| std::env::var("LANG").ok())
            .or_else(|| std::env::var("LANGUAGE").ok())
            .or_else(|| std::env::var("LC_MESSAGES").ok())
            .or_else(|| std::env::var("LC_ALL").ok())
            .unwrap_or_else(|| String::from("en-us"));

        lang.parse::<LanguageIdentifier>()
            .unwrap_or_else(|_| ENGLISH_LANG.clone())
    };

    pub static ref DEFAULT_BUNDLE: bundle::LocalizationBundle = bundle::LocalizationBundle::default();
}

/// Get translation string from default translations bundle.
///
/// - `i18n("string_key") -> Option<&str>`
/// - `i18n("string_key", { "arg" => "value", ... }) -> Option<String>`
/// - `i18n(lang, "string_key") -> Option<&str>`
/// - `i18n(lang, "string_key", { "arg" => "value", ... }) -> Option<String>`
#[macro_export]
macro_rules! i18n {
    ($key:expr) => {
        DEFAULT_BUNDLE.get_translation(key, SYSTEM_LANG.as_ref())
    };

    ($key:expr, {$( $arg_key:expr => $arg_value:expr $(,)* )+}) => {
        let mut args = std::collections::HashMap::new();

        $( args.insert($arg_key, $arg_value); )+

        DEFAULT_BUNDLE.get_translation_with_args(key, SYSTEM_LANG.as_ref(), args)
    };

    ($lang:expr, $key:expr) => {
        DEFAULT_BUNDLE.get_translation(key, $lang)
    };

    ($lang:expr, $key:expr, {$( $arg_key:expr => $arg_value:expr $(,)* )+}) => {
        let mut args = std::collections::HashMap::new();

        $( args.insert($arg_key, $arg_value); )+

        DEFAULT_BUNDLE.get_translation_with_args(key, $lang, args)
    };
}
