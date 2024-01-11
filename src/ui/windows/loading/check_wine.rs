use crate::components::wine::Wine;

#[inline]
pub fn get_download() -> anyhow::Result<Option<Wine>> {
    let wine = Wine::from_config()?;

    if wine.is_downloaded() {
        return Ok(None);
    }

    Ok(Some(wine))
}
