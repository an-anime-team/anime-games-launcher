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
use std::path::Path;

use unic_langid::LanguageIdentifier;

use crate::string::LocalizableString;

type Err = Box<dyn std::error::Error>;

/// A table of string keys and associated translatable strings.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct LocalizationBundle(HashMap<String, LocalizableString>);

impl LocalizationBundle {
    /// Load bundle from a TOML file.
    pub fn load_file(&mut self, path: impl AsRef<Path>) -> Result<(), Err> {
        self.load_str(std::fs::read_to_string(path)?)
    }

    /// Load bundle from a TOML string.
    pub fn load_str(&mut self, str: impl AsRef<str>) -> Result<(), Err> {
        let translations = toml::from_str::<HashMap<String, toml::Value>>(str.as_ref())?;

        for (key, translation) in translations {
            if let Some(translation) = LocalizableString::from_toml(&translation) {
                // TODO: merge translations.
                self.0.insert(key, translation);
            }
        }

        Ok(())
    }

    /// Get localizable string from a bundle.
    pub fn get_str(&self, key: impl AsRef<str>) -> Option<&LocalizableString> {
        self.0.get(key.as_ref())
    }

    /// Get translation of a string with provided key and language.
    pub fn get_translation(
        &self,
        key: impl AsRef<str>,
        lang: impl AsRef<LanguageIdentifier>
    ) -> Option<&'_ str> {
        self.0.get(key.as_ref())
            .map(|str| str.translate(lang))
    }

    /// Get default translation of a string with provided key.
    pub fn get_default_translation(
        &self,
        key: impl AsRef<str>
    ) -> Option<&'_ str> {
        self.0.get(key.as_ref())
            .map(|str| str.default_translation())
    }

    /// Get translation of a string with provided key, language and args.
    pub fn get_translation_with_args<K: std::fmt::Display, V: AsRef<str>>(
        &self,
        key: impl AsRef<str>,
        lang: impl AsRef<LanguageIdentifier>,
        args: impl IntoIterator<Item = (K, V)>
    ) -> Option<String> {
        self.0.get(key.as_ref())
            .map(|str| str.translate_with_args(lang, args))
    }

    /// Get default translation of a string with provided key and args.
    pub fn get_default_translation_with_args<K, V>(
        &self,
        key: impl AsRef<str>,
        args: impl IntoIterator<Item = (K, V)>
    ) -> Option<String>
    where
        K: std::fmt::Display,
        V: AsRef<str>
    {
        self.0.get(key.as_ref())
            .map(|str| str.default_translation_with_args(args))
    }
}
