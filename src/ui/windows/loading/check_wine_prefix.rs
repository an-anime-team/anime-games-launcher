use crate::config;
use crate::config::components::wine::prefix::Prefix;

#[inline]
pub fn check_wine_prefix() -> Option<Prefix> {
    let prefix = config::get().components.wine.prefix;

    if !prefix.path.exists() {
        return Some(prefix);
    }

    None
}
