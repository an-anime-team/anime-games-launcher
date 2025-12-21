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

use std::collections::HashMap;
use std::str::FromStr;

pub use unic_langid;

#[cfg(feature = "json")]
use serde_json::{json, Value as Json};

#[cfg(feature = "mlua")]
use mlua::prelude::*;

use unic_langid::LanguageIdentifier;

lazy_static::lazy_static! {
    static ref EN_LANGID: LanguageIdentifier = LanguageIdentifier::from_str("en")
        .expect("failed to parse `en` language identifier");
}

// Get lowercase string from the language identifier.
fn lang_code(lang: &LanguageIdentifier) -> String {
    let language = lang.language.to_string()
        .to_ascii_lowercase();

    match lang.region {
        Some(region) => format!("{language}-{}", region.as_str().to_ascii_lowercase()),
        None => language
    }
}

/// A string variant which can contain translations for different languages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalizableString {
    Raw(String),
    Translatable(HashMap<LanguageIdentifier, String>)
}

impl LocalizableString {
    /// Create new string with no translations.
    pub fn raw(value: impl ToString) -> Self {
        Self::Raw(value.to_string())
    }

    /// Create new string with provided translations table.
    pub fn translatable<T>(
        iter: impl IntoIterator<Item = (LanguageIdentifier, T)>
    ) -> Self
    where
        T: ToString
    {
        let translations = iter.into_iter()
            .map(|(lang, locale)| (lang, locale.to_string()))
            .collect::<HashMap<_, _>>();

        Self::Translatable(translations)
    }

    /// Get translated value of the string.
    ///
    /// This function will try to translate the value using provided target
    /// language, fallback to English or return a stub string if both failed.
    pub fn translate(&self, locale: &LanguageIdentifier) -> &str {
        match self {
            Self::Raw(str) => str,

            Self::Translatable(values) => {
                if let Some(value) = values.get(locale) {
                    return value;
                }

                let mut english_value = "<no translation available>";

                for (key, value) in values {
                    if key.language == locale.language {
                        return value;
                    }

                    if key.language == "en" {
                        english_value = value;
                    }
                }

                english_value
            }
        }
    }

    /// Get default translation of the string.
    ///
    /// Will either return the raw value, English variant or a stub string.
    #[inline]
    pub fn default_translation(&self) -> &str {
        self.translate(EN_LANGID.as_ref())
    }
}

impl LocalizableString {
    #[cfg(feature = "json")]
    pub fn to_json(&self) -> Json {
        match self {
            LocalizableString::Raw(str) => json!(str),

            LocalizableString::Translatable(values) => {
                let values = values.iter()
                    .map(|(k, v)| (lang_code(k), v))
                    .collect::<HashMap<String, &String>>();

                json!(values)
            }
        }
    }

    #[cfg(feature = "json")]
    pub fn from_json(value: &Json) -> Option<Self> {
        if value.is_string() {
            let str = value.as_str()?
                .to_string();

            return Some(Self::Raw(str));
        }

        else if value.is_object() {
            let raw_translations = value.as_object()?;

            let mut translations = HashMap::with_capacity(raw_translations.len());

            for (lang, value) in raw_translations {
                let lang = lang.parse::<LanguageIdentifier>().ok()?;

                let value = value.as_str()?
                    .to_string();

                translations.insert(lang, value);
            }

            return Some(Self::Translatable(translations));
        }

        None
    }

    #[cfg(feature = "mlua")]
    pub fn to_lua(&self, lua: &Lua) -> Result<LuaValue, LuaError> {
        match self {
            Self::Raw(string) => Ok(LuaValue::String(lua.create_string(string)?)),

            Self::Translatable(translations) => {
                let table = lua.create_table_with_capacity(0, translations.len())?;

                for (lang, translation) in translations {
                    table.set(lang.to_string(), lua.create_string(translation)?)?;
                }

                Ok(LuaValue::Table(table))
            }
        }
    }

    #[cfg(feature = "mlua")]
    pub fn from_lua(value: &LuaValue) -> Result<Self, LuaError> {
        if let Some(translations) = value.as_table().cloned() {
            let mut table = HashMap::new();

            for pair in translations.pairs::<String, String>() {
                let (lang, translation) = pair?;

                let lang = lang.parse::<LanguageIdentifier>()
                    .map_err(LuaError::external)?;

                table.insert(lang, translation);
            }

            Ok(Self::Translatable(table))
        }

        else {
            Ok(Self::Raw(value.to_string()?))
        }
    }
}

#[test]
fn test_translate() {
    let text = LocalizableString::raw("Hello, World!");

    let en_us = LanguageIdentifier::from_str("en-us").unwrap();
    let ru_ru = LanguageIdentifier::from_str("ru-ru").unwrap();
    let en_gb = LanguageIdentifier::from_str("en-gb").unwrap();
    let zh_cn = LanguageIdentifier::from_str("zh-cn").unwrap();

    let en = LanguageIdentifier::from_str("en").unwrap();
    let ru = LanguageIdentifier::from_str("ru").unwrap();
    let zh = LanguageIdentifier::from_str("zh").unwrap();
    let fr = LanguageIdentifier::from_str("fr").unwrap();

    assert_eq!(text.translate(&en_us), "Hello, World!");

    let text = LocalizableString::translatable([
        (en_us.clone(), "Test"),
        (ru_ru.clone(), "Тест"),
        (zh.clone(), "测试"),
    ]);

    assert_eq!(text.translate(&en_us), "Test");
    assert_eq!(text.translate(&ru_ru), "Тест");
    assert_eq!(text.translate(&zh), "测试");

    assert_eq!(text.translate(&en_gb), "Test");
    assert_eq!(text.translate(&en), "Test");
    assert_eq!(text.translate(&ru), "Тест");
    assert_eq!(text.translate(&zh_cn), "测试");
    assert_eq!(text.translate(&fr), "Test");
}
