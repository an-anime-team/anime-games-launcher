#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DiskType {
    Hdd,
    Ssd,
    Nvme
}

impl std::fmt::Display for DiskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hdd  => write!(f, "hdd"),
            Self::Ssd  => write!(f, "ssd"),
            Self::Nvme => write!(f, "nvme")
        }
    }
}

impl std::str::FromStr for DiskType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hdd"  => Ok(Self::Hdd),
            "ssd"  => Ok(Self::Ssd),
            "nvme" => Ok(Self::Nvme),

            _ => anyhow::bail!("Unsupported disk type: {s}")
        }
    }
}
