use crate::i18n;
use crate::config;

pub fn init_locales(config: &config::Config) -> anyhow::Result<()> {
    i18n::set_language(config.general.language.parse()?)?;

    Ok(())
}
