use crate::components::dxvk::Dxvk;

#[inline]
pub fn is_downloaded() -> anyhow::Result<Option<Dxvk>> {
    let dxvk = Dxvk::from_config()?;

    if dxvk.is_downloaded() {
        return Ok(None);
    }

    Ok(Some(dxvk))
}
