use wincompatlib::dxvk::Dxvk as WincompatlibDxvk;

use crate::config;
use crate::components::dxvk::Dxvk;

#[inline]
pub fn get_download() -> anyhow::Result<Option<Dxvk>> {
    let dxvk = Dxvk::from_config()?;

    if dxvk.is_downloaded() {
        return Ok(None);
    }

    Ok(Some(dxvk))
}

#[inline]
pub fn get_apply() -> anyhow::Result<Option<Dxvk>> {
    let dxvk = Dxvk::from_config()?;

    let installed_dxvk = WincompatlibDxvk::get_version(config::get().components.wine.prefix.path)?;

    if let Some(version) = installed_dxvk {
        if dxvk.name.contains(&version) {
            return Ok(None);
        }
    }

    Ok(Some(dxvk))
}
