use unic_langid::{langid, LanguageIdentifier};

fluent_templates::static_loader! {
    pub static LOCALES = {
        locales: "./assets/locales",
        core_locales: "./assets/locales/common.ftl",
        fallback_language: "en"
    };
}

lazy_static::lazy_static! {
    /// Current system language code.
    ///
    /// Parsed from the environment variables.
    static ref SYSTEM_LANGUAGE: String = std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_else(|_| String::from("en_us"))
        .to_ascii_lowercase();

    /// Get system language or default language
    /// if system one is not supported.
    static ref DEFAULT_LANGUAGE: LanguageIdentifier = SUPPORTED_LANGUAGES.iter()
        .find(|lang| SYSTEM_LANGUAGE.starts_with(lang.language.as_str()))
        .unwrap_or_else(|| unsafe { get_lang() })
        .clone();
}

/// Map of supported languages
pub const SUPPORTED_LANGUAGES: &[LanguageIdentifier] = &[
    langid!("en-us"),
    langid!("ru-ru"),
    langid!("de-de"),
    langid!("pt-br"),
    langid!("zh-cn")
];

static mut LANG: LanguageIdentifier = langid!("en-us");

/// Set launcher language
pub fn set_language(lang: LanguageIdentifier) -> anyhow::Result<()> {
    if SUPPORTED_LANGUAGES.iter().any(|item| item.language == lang.language) {
        unsafe {
            LANG = lang
        }

        return Ok(());
    }

    anyhow::bail!("Language '{lang}' is not supported");
}

#[allow(clippy::missing_safety_doc)]
/// Get launcher language
pub unsafe fn get_lang<'a>() -> &'a LanguageIdentifier {
    LANG.as_ref()
}

/// Format given language to `<language>-<country>` format
///
/// Example: `en-us`, `ru-ru`
pub fn format_language(lang: &LanguageIdentifier) -> String {
    format!("{}-{}", lang.language, match lang.region {
        Some(region) => region.to_string().to_ascii_lowercase(),
        None => lang.language.to_string()
    })
}

#[macro_export]
/// Get translated message by key, with optional translation parameters
///
/// # Examples:
///
/// Without parameters:
///
/// ```no_run
/// println!("Translated message: {}", tr!("launch"));
/// ```
///
/// With parameters:
///
/// ```no_run
/// println!("Translated message: {}", tr!("game-outdated", {
///     "latest" = "3.3.0"
/// }));
/// ```
macro_rules! tr {
    ($id:expr) => {
        {
            use fluent_templates::Loader;

            #[allow(unused_unsafe)]
            $crate::i18n::LOCALES
                .lookup(unsafe { $crate::i18n::get_lang() }, $id)
                .expect(&format!("Failed to find a message with given id: {}", stringify!($id)))
        }
    };

    ($id:expr, { $($key:literal = $value:expr),* }) => {
        {
            use std::collections::HashMap;

            use fluent_templates::Loader;
            use fluent_templates::fluent_bundle::FluentValue;

            let mut args = HashMap::new();

            $(
                args.insert($key, FluentValue::from($value));
            )*

            #[allow(unused_unsafe)]
            $crate::i18n::LOCALES
                .lookup_complete(unsafe { $crate::i18n::get_lang() }, $id, Some(&args))
                .expect(&format!("Failed to find a message with given id: {}", stringify!($id)))
        }
    };
}
