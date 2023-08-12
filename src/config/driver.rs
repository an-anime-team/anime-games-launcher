use std::path::PathBuf;

use serde::{Serialize, Deserialize};

use anime_game_core::filesystem::DriverExt;
use anime_game_core::filesystem::physical::Driver as PhysicalFsDriver;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Driver {
    PhysicalFsDriver {
        base_folder: PathBuf
    }
}

impl Driver {
    pub fn to_dyn_trait(&self) -> impl DriverExt {
        match self {
            Self::PhysicalFsDriver { base_folder } => {
                PhysicalFsDriver::new(base_folder)
            }
        }
    }
}

// TODO: proper Serialize / Deserialize implementation

// impl TryFrom<&Json> for Driver {
//     type Error = anyhow::Error;

//     fn try_from(value: &Json) -> Result<Self, Self::Error> {
//         let Some(driver) = value.get("driver").and_then(Json::as_str) else {
//             anyhow::bail!("Wrong driver description: no 'driver' name given");
//         };

//         let Some(params) = value.get("params") else {
//             anyhow::bail!("Wrong driver description: no 'params' given");
//         };

//         match driver {
//             "physical_filesystem_driver" => {
//                 let Some(base_folder) = params.get("base_folder").and_then(Json::as_str) else {
//                     anyhow::bail!("Wrong driver params description: no 'base_folder' given");
//                 };

//                 Ok(Driver::PhysicalFsDriver {
//                     base_folder: PathBuf::from(base_folder)
//                 })
//             },

//             driver => anyhow::bail!("Unsupported driver used: '{driver}'")
//         }
//     }
// }
