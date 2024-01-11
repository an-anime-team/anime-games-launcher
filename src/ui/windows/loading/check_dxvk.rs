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
    let prefix = config::get().components.wine.prefix.path;

    if !prefix.exists() {
        // We don't need to apply DXVK because it's done during prefix creation
        return Ok(None);
    }

    let selected_dxvk = Dxvk::from_config()?;

    let installed_dxvk = WincompatlibDxvk::get_version(prefix)?;

    if let Some(version) = installed_dxvk {
        if selected_dxvk.name.contains(&version) || selected_dxvk.version.contains(&version) {
            return Ok(None);
        }
    }

    Ok(Some(selected_dxvk))
}
