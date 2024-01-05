use crate::config;

#[inline]
pub fn init_config() -> anyhow::Result<()> {
    config::update(&config::get())
}
