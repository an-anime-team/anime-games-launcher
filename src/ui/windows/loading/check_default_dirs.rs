use crate::{
    LAUNCHER_FOLDER,
    COMPONENTS_FOLDER
};

#[inline]
pub fn check_default_dirs() -> anyhow::Result<()> {
    if !LAUNCHER_FOLDER.exists() {
        std::fs::create_dir_all(LAUNCHER_FOLDER.as_path())?;

        std::fs::create_dir_all(COMPONENTS_FOLDER.join("wine"))?;
        std::fs::create_dir_all(COMPONENTS_FOLDER.join("dxvk"))?;
    }

    Ok(())
}
