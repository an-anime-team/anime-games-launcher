use std::collections::HashMap;

use serde_json::{json, Value as Json};
use mlua::prelude::*;

use unic_langid::{langid, LanguageIdentifier};

use crate::prelude::*;

// Get lowercase string from the language identifier.
fn lang_code(lang: &LanguageIdentifier) -> String {
    let language = lang.language.to_string()
        .to_ascii_lowercase();

    match lang.region {
        Some(region) => format!("{language}-{}", region.as_str().to_ascii_lowercase()),
        None => language
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalizableString {
    Raw(String),
    Translatable(HashMap<LanguageIdentifier, String>)
}

impl LocalizableString {
    /// Create new raw localizable string.
    ///
    /// ```
    /// let string = LocalizableString::raw("Hello, World!");
    /// ```
    pub fn raw(value: impl ToString) -> Self {
        Self::Raw(value.to_string())
    }

    /// Create new translatable string.
    ///
    /// ```
    /// use unic_langid::langid;
    ///
    /// let string = LocalizableString::translatable([
    ///     (langid!("en"), String::from("Hello, World!")),
    ///     (langid!("ru"), String::from("Привет, Мир!"))
    /// ]);
    /// ```
    pub fn translatable(iter: impl IntoIterator<Item = (LanguageIdentifier, String)>) -> Self {
        Self::Translatable(HashMap::from_iter(iter))
    }

    /// Get translated value of the string.
    ///
    /// This function will try to translate the value using
    /// provided target language, fallback to English or
    /// return a stub string if both failed.
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

    #[inline]
    /// Get default translation of the string.
    ///
    /// Will either return the raw value, English
    /// variant or a stub string.
    pub fn default_translation(&self) -> &str {
        self.translate(&langid!("en"))
    }
}

impl AsJson for LocalizableString {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        match self {
            Self::Raw(str) => Ok(json!(str)),

            Self::Translatable(values) => {
                let values = values.iter()
                    .map(|(k, v)| (lang_code(k), v))
                    .collect::<HashMap<String, &String>>();

                Ok(json!(values))
            }
        }
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        if json.is_string() {
            let str = json.as_str()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("<localizable string>"))?
                .to_string();

            return Ok(Self::Raw(str));
        }

        else if json.is_object() {
            let raw_translations = json.as_object()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("<localizable string>"))?;

            let mut translations = HashMap::with_capacity(raw_translations.len());

            for (lang, value) in raw_translations {
                let lang = lang.parse::<LanguageIdentifier>()
                    .map_err(|err| AsJsonError::Other(err.into()))?;

                let value = value.as_str()
                    .ok_or_else(|| AsJsonError::InvalidFieldValue("<localizable string>"))?
                    .to_string();

                translations.insert(lang, value);
            }

            return Ok(Self::Translatable(translations));
        }

        Err(AsJsonError::InvalidFieldValue("<localizable string>"))
    }
}

impl AsLua for LocalizableString {
    fn to_lua(&self, lua: &Lua) -> Result<LuaValue, AsLuaError> {
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

    fn from_lua(value: &LuaValue) -> Result<Self, AsLuaError> where Self: Sized {
        if let Some(translations) = value.as_table().cloned() {
            let mut table = HashMap::new();

            for pair in translations.pairs::<LuaString, LuaString>() {
                let (lang, translation) = pair?;

                let lang = lang.to_string_lossy()
                    .parse::<LanguageIdentifier>()
                    .map_err(|err| AsLuaError::Other(err.into()))?;

                table.insert(lang, translation.to_string_lossy().to_string());
            }

            Ok(Self::Translatable(table))
        }

        else {
            Ok(Self::Raw(value.to_string()?))
        }
    }
}

impl AsHash for LocalizableString {
    fn hash(&self) -> Hash {
        match self {
            Self::Raw(str) => str.hash(),

            Self::Translatable(values) => {
                let mut hash = Hash::default();

                for (key, value) in values {
                    hash ^= Hash::for_slice(key.to_string());
                    hash ^= value.hash();
                }

                hash
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate() {
        let text = LocalizableString::raw("Hello, World!");

        assert_eq!(text.translate(&langid!("en-us")), "Hello, World!");

        let text = LocalizableString::translatable([
            (langid!("en-us"), String::from("Test")),
            (langid!("ru-ru"), String::from("Тест")),
            (langid!("zh"), String::from("测试")),
        ]);

        assert_eq!(text.translate(&langid!("en-us")), "Test");
        assert_eq!(text.translate(&langid!("ru-ru")), "Тест");
        assert_eq!(text.translate(&langid!("zh")), "测试");

        assert_eq!(text.translate(&langid!("en-gb")), "Test");
        assert_eq!(text.translate(&langid!("en")), "Test");
        assert_eq!(text.translate(&langid!("ru")), "Тест");
        assert_eq!(text.translate(&langid!("zh-cn")), "测试");
        assert_eq!(text.translate(&langid!("fr")), "Test");
    }
}
