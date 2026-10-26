// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-games
// Copyright (C) 2026  Nikita Podvirnyi <krypt0nn@dawn.wine>
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

use std::collections::HashMap;
use std::sync::LazyLock;
use std::process::{Command, Stdio};
use std::path::PathBuf;

pub static BWRAP_BINARY: LazyLock<Option<&'static str>> = LazyLock::new(|| {
    Command::new("bwrap")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .is_ok()
        .then_some("bwrap")
});

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct SandboxOptions {
    /// List of filesystem mounts within the sandbox.
    pub mounts: Vec<SandboxMountOption>,

    /// Table of environment variables values available inside of the sandbox.
    /// If value starts with `$`, then host system variable is read.
    ///
    /// Example: `HOME=$HOME` will read host system `HOME` variable.
    pub env: HashMap<String, String>
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SandboxMountOption {
    /// Original file or directory path.
    pub host_path: PathBuf,

    /// Path within the sandbox.
    pub sandbox_path: PathBuf,

    /// Make file or directory read-only.
    pub read_only: bool
}

/// Wrap existing command with sandbox options. Return `Ok` if command was
/// wrapped, or `Err` if no sandbox utility available.
#[allow(clippy::result_large_err)]
pub fn wrap<'a>(
    command: &'a Command,
    options: &SandboxOptions
) -> Result<Command, &'a Command> {
    let Some(bwrap) = &*BWRAP_BINARY else {
        return Err(command);
    };

    // Build bwrap command.
    let mut sb_command = Command::new(bwrap);

    // Standard virtual filesystems.
    sb_command.args(["--dev", "/dev"]);
    sb_command.args(["--proc", "/proc"]);
    sb_command.args(["--tmpfs", "/tmp"]);

    if matches!(std::fs::exists("/dev/dri"), Ok(true)) {
        sb_command.args(["--dev-bind", "/dev/dri", "/dev/dri"]);
    }

    // Standard paths.
    for path in ["/usr/lib", "/usr/lib64", "/usr/bin", "/usr/share", "/etc"] {
        if matches!(std::fs::exists(path), Ok(true)) {
            sb_command.args(["--ro-bind", path, path]);
        }
    }

    // Wayland, pipewire, pulse audio.
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let wayland_dir = format!("{runtime_dir}/wayland-1");
        let pipewire_dir = format!("{runtime_dir}/pipewire-0");
        let pulse_dir = format!("{runtime_dir}/pulse");

        sb_command.args(["--dir", &runtime_dir]);

        for path in [wayland_dir, pipewire_dir, pulse_dir] {
            if matches!(std::fs::exists(&path), Ok(true)) {
                sb_command.args(["--ro-bind", &path, &path]);
            }
        }
    }

    // Add mount options.
    for mount in &options.mounts {
        if matches!(std::fs::exists(&mount.host_path), Ok(true)) {
            if mount.read_only {
                sb_command.arg("--ro-bind");
                sb_command.arg(&mount.host_path);
                sb_command.arg(&mount.sandbox_path);
            } else {
                sb_command.arg("--bind");
                sb_command.arg(&mount.host_path);
                sb_command.arg(&mount.sandbox_path);
            }
        }
    }

    // Copy original command options.
    sb_command.arg(command.get_program());
    sb_command.args(command.get_args());

    for (k, v) in command.get_envs() {
        if let Some(v) = v {
            sb_command.env(k, v);
        }
    }

    // Overwrite environment variables.
    for (k, v) in &options.env {
        sb_command.env(k, v);
    }

    Ok(sb_command)
}
