use serde_json::{json, Value as Json};

use crate::core::prelude::*;
use crate::packages::prelude::*;

use super::cpu::CpuHardwareRequirements;
use super::gpu::GpuHardwareRequirements;
use super::ram::RamHardwareRequirements;
use super::disk::DiskHardwareRequirements;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct HardwareRequirements {
    pub cpu: Option<CpuHardwareRequirements>,
    pub gpu: Option<GpuHardwareRequirements>,
    pub ram: Option<RamHardwareRequirements>,
    pub disk: Option<DiskHardwareRequirements>
}

impl AsJson for HardwareRequirements {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "cpu": self.cpu.as_ref()
                .map(CpuHardwareRequirements::to_json)
                .transpose()?,

            "gpu": self.gpu.as_ref()
                .map(GpuHardwareRequirements::to_json)
                .transpose()?,

            "ram": self.ram.as_ref()
                .map(RamHardwareRequirements::to_json)
                .transpose()?,

            "disk": self.disk.as_ref()
                .map(DiskHardwareRequirements::to_json)
                .transpose()?
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            cpu: {
                match json.get("cpu") {
                    Some(cpu) => {
                        if cpu.is_null() {
                            None
                        } else {
                            CpuHardwareRequirements::from_json(cpu)
                                .map(Some)?
                        }
                    }

                    None => None
                }
            },

            gpu: {
                match json.get("gpu") {
                    Some(gpu) => {
                        if gpu.is_null() {
                            None
                        } else {
                            GpuHardwareRequirements::from_json(gpu)
                                .map(Some)?
                        }
                    }

                    None => None
                }
            },

            ram: {
                match json.get("ram") {
                    Some(ram) => {
                        if ram.is_null() {
                            None
                        } else {
                            RamHardwareRequirements::from_json(ram)
                                .map(Some)?
                        }
                    }

                    None => None
                }
            },

            disk: {
                match json.get("disk") {
                    Some(disk) => {
                        if disk.is_null() {
                            None
                        } else {
                            DiskHardwareRequirements::from_json(disk)
                                .map(Some)?
                        }
                    }

                    None => None
                }
            }
        })
    }
}

impl AsHash for HardwareRequirements {
    fn hash(&self) -> Hash {
        self.cpu.hash()
            .chain(self.gpu.hash())
            .chain(self.ram.hash())
            .chain(self.disk.hash())
    }
}
