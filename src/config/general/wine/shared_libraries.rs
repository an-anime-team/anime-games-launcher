use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

// https://github.com/bottlesdevs/Bottles/blob/8d4cb54e4645166e21fec7b0686dbdb89e0fd2c2/bottles/backend/wine/winecommand.py#L228

const WINE_LIBS: &[&str] = &[
    "lib",
    "lib64",
    "lib/wine/x86_64-unix",
    "lib32/wine/x86_64-unix",
    "lib64/wine/x86_64-unix",
    "lib/wine/i386-unix",
    "lib32/wine/i386-unix",
    "lib64/wine/i386-unix"
];

const GST_LIBS: &[&str] = &[
    "lib64/gstreamer-1.0",
    "lib/gstreamer-1.0",
    "lib32/gstreamer-1.0"
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SharedLibraries {
    /// Set `LD_LIBRARY_PATH` variable with paths to the wine shared libraries
    pub wine: bool,

    /// Set `GST_PLUGIN_PATH` variable with paths to gstreamer shared libraries
    /// 
    /// https://gstreamer.freedesktop.org/documentation/gstreamer/gstregistry.html?gi-language=c
    pub gstreamer: bool
}

impl Default for SharedLibraries {
    #[inline]
    fn default() -> Self {
        Self {
            wine: true,
            gstreamer: true
        }
    }
}

impl From<&Json> for SharedLibraries {
    #[inline]
    fn from(value: &Json) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }
}

impl SharedLibraries {
    /// Get environment variables corresponding to selected shared variables options
    pub fn get_env_vars(&self, wine_folder: impl Into<PathBuf>) -> HashMap<&str, String> {
        let mut env = HashMap::new();
        let wine_folder: PathBuf = wine_folder.into();

        // Setup `LD_LIBRARY_PATH`

        if self.wine {
            let mut ld_libs = Vec::with_capacity(WINE_LIBS.len());

            for folder in WINE_LIBS {
                let folder = wine_folder.join(folder);

                if folder.exists() {
                    ld_libs.push(folder.to_string_lossy().to_string());
                }
            }

            env.insert("LD_LIBRARY_PATH", ld_libs.join(":"));
        }

        // Setup `GST_PLUGIN_PATH`

        if self.gstreamer {
            let mut gst_libs = Vec::with_capacity(GST_LIBS.len());

            for folder in GST_LIBS {
                let folder = wine_folder.join(folder);

                if folder.exists() {
                    gst_libs.push(folder.to_string_lossy().to_string());
                }
            }

            env.insert("GST_PLUGIN_PATH", gst_libs.join(":"));
        }

        env
    }
}
