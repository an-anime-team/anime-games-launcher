use crate::config;

#[inline]
pub fn init_config() -> anyhow::Result<config::Config> {
    let config = config::get();

    config::update(&config)?;

    Ok(config)
}
