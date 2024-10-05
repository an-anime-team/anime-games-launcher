use std::collections::HashMap;

use serde_json::{json, Value as Json};
use unic_langid::LanguageIdentifier;
use mlua::prelude::*;

use crate::core::prelude::*;
use crate::packages::prelude::*;

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
}

impl TryFrom<&LuaValue<'_>> for LocalizableString {
    type Error = LuaError;

    fn try_from(value: &LuaValue) -> Result<Self, Self::Error> {
        if let Some(translations) = value.as_table() {
            let mut result = HashMap::with_capacity(translations.raw_len());

            for pair in translations.clone().pairs::<LuaString, LuaString>() {
                let (lang, value) = pair?;

                let lang = lang.to_string_lossy()
                    .parse::<LanguageIdentifier>()
                    .map_err(LuaError::external)?;

                let value = value.to_string_lossy()
                    .to_string();

                result.insert(lang, value);
            }

            Ok(Self::translatable(result))
        }

        else if let Some(value) = value.as_string_lossy() {
            Ok(Self::raw(value))
        }

        else {
            Err(LuaError::external("failed to parse localizable string"))
        }
    }
}

impl AsJson for LocalizableString {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        match self {
            Self::Raw(str) => Ok(json!(str)),

            Self::Translatable(values) => {
                let values = values.iter()
                    .map(|(k, v)| {
                        let language = k.language.to_string()
                            .to_ascii_lowercase();

                        let locale = match k.region {
                            Some(region) => format!("{language}-{}", region.as_str().to_ascii_lowercase()),
                            None => language
                        };

                        (locale, v)
                    })
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
    use unic_langid::langid;

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
