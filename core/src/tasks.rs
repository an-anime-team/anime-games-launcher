// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-core
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

pub use tokio::{fs, io, net, sync};
pub use tokio::task::{JoinHandle, JoinError};

use tokio::runtime::{Runtime, Builder};

lazy_static::lazy_static! {
    pub static ref RUNTIME: Runtime = Builder::new_multi_thread()
        .thread_name("anime_games_launcher_core")
        .enable_all()
        .build()
        .expect("failed to initialize tokio runtime");
}

/// Spawn future in the shared tokio runtime.
#[inline(always)]
pub fn spawn<T: Send + 'static>(
    future: impl Future<Output = T> + Send + 'static
) -> JoinHandle<T> {
    RUNTIME.spawn(future)
}

/// Block current thread to execute the future.
#[inline(always)]
pub fn block_on<T>(future: impl Future<Output = T>) -> T {
    RUNTIME.block_on(future)
}
