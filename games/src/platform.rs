// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-games
// Copyright (C) 2025  Nikita Podvirnyi <krypt0nn@vk.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::str::FromStr;

lazy_static::lazy_static! {
    static ref SYSTEM_INFO: os_info::Info = os_info::get();

    static ref CPU_ARCH: Option<Arch> = SYSTEM_INFO.architecture()
        .and_then(|arch| Arch::from_str(arch).ok());

    static ref OS_FAMILY: Option<System> = match SYSTEM_INFO.os_type() {
        os_info::Type::Windows => Some(System::Windows),
        os_info::Type::Macos => Some(System::Darwin),

        os_info::Type::Android |
        os_info::Type::Ios |
        os_info::Type::DragonFly |
        os_info::Type::Unknown => None,

        // Not very accurate.
        _ => Some(System::Linux)
    };

    static ref PLATFORM: Option<Platform> = {
        if let (Some(arch), Some(system)) = (*CPU_ARCH, *OS_FAMILY) {
            Some(Platform { arch, system })
        } else {
            None
        }
    };
}

/// CPU architecture.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Arch {
    /// `x86_64`
    X86_64,

    /// `aarch64`
    Aarch64
}

impl Arch {
    /// Try to get current system's CPU architecture.
    #[inline]
    pub fn current() -> Option<Self> {
        *CPU_ARCH
    }
}

impl std::fmt::Display for Arch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::X86_64  => f.write_str("x86_64"),
            Self::Aarch64 => f.write_str("aarch64")
        }
    }
}

impl FromStr for Arch {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "x86_64" | "amd64" | "x64" => Ok(Self::X86_64),
            "aarch64" | "arm64"        => Ok(Self::Aarch64),

            _ => Err(())
        }
    }
}

/// OS family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum System {
    /// `windows`
    Windows,

    /// `linux`
    Linux,

    /// `darwin` (MacOS)
    Darwin
}

impl System {
    /// Try to get current system's OS family.
    #[inline]
    pub fn current() -> Option<Self> {
        *OS_FAMILY
    }
}

impl std::fmt::Display for System {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Windows => f.write_str("windows"),
            Self::Linux   => f.write_str("linux"),
            Self::Darwin  => f.write_str("darwin")
        }
    }
}

impl FromStr for System {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "windows" | "nt"   => Ok(Self::Windows),
            "linux"            => Ok(Self::Linux),
            "darwin" | "macos" => Ok(Self::Darwin),

            _ => Err(())
        }
    }
}

/// Information about a host system platform (CPU architecture + OS family).
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Platform {
    pub arch: Arch,
    pub system: System
}

impl Platform {
    /// Try to get current system's platform.
    #[inline]
    pub fn current() -> Option<Self> {
        *PLATFORM
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.arch, self.system)
    }
}

impl FromStr for Platform {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((arch, system)) = s.split_once('-') else {
            return Err(());
        };

        Ok(Self {
            arch: Arch::from_str(arch)?,
            system: System::from_str(system)?
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arch_display() {
        assert_eq!(Arch::X86_64.to_string(), "x86_64");
        assert_eq!(Arch::Aarch64.to_string(), "aarch64");

        assert_eq!(Arch::from_str("x86_64"), Ok(Arch::X86_64));
        assert_eq!(Arch::from_str("aarch64"), Ok(Arch::Aarch64));
    }

    #[test]
    fn system_display() {
        assert_eq!(System::Windows.to_string(), "windows");
        assert_eq!(System::Linux.to_string(), "linux");
        assert_eq!(System::Darwin.to_string(), "darwin");

        assert_eq!(System::from_str("windows"), Ok(System::Windows));
        assert_eq!(System::from_str("linux"), Ok(System::Linux));
        assert_eq!(System::from_str("darwin"), Ok(System::Darwin));
    }

    #[test]
    fn platform_display() {
        let platform = Platform {
            arch: Arch::X86_64,
            system: System::Linux
        };

        assert_eq!(platform.to_string(), "x86_64-linux");
        assert_eq!(Platform::from_str("x86_64-linux"), Ok(platform));
    }
}
