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

/// Version of the `agl-core` library.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod rw_sync;
pub mod buffer;

pub mod tasks;

#[cfg(feature = "network")]
pub mod network;

#[cfg(feature = "archives")]
pub mod archives;

#[cfg(feature = "hashes")]
pub mod hashes;

#[cfg(feature = "compression")]
pub mod compression;

#[cfg(any(
    feature = "tasks",
    feature = "network",
    feature = "hashes",
    feature = "compression"
))]
pub mod export {
    //! Re-exports of core library dependencies.

    #[cfg(feature = "tasks")]
    pub mod tasks {
        //! Re-exports of the `tasks` feature dependencies.

        pub use tokio;
    }

    #[cfg(feature = "network")]
    pub mod network {
        //! Re-exports of the `network` feature dependencies.

        pub use reqwest;
    }

    #[cfg(feature = "hashes")]
    pub mod hashes {
        //! Re-exports of the `hashes` feature dependencies.

        #[cfg(feature = "hashes-seahash")]
        pub use seahash;

        #[cfg(feature = "hashes-crc32")]
        pub use crc32fast as crc32;

        #[cfg(feature = "hashes-crc32c")]
        pub use crc32c;

        #[cfg(feature = "hashes-xxh")]
        pub use xxhash_rust as xxh;

        #[cfg(feature = "hashes-md5")]
        pub use md5;

        #[cfg(feature = "hashes-sha1")]
        pub use sha1;

        #[cfg(feature = "hashes-sha2")]
        pub use sha2;

        #[cfg(feature = "hashes-sha3")]
        pub use sha3;

        #[cfg(feature = "hashes-blake2")]
        pub use blake2;

        #[cfg(feature = "hashes-blake3")]
        pub use blake3;
    }

    #[cfg(feature = "compression")]
    pub mod compression {
        //! Re-exports of the `compression` feature dependencies.

        #[cfg(feature = "compression-lz4")]
        pub use lz4_flex as lz4;

        #[cfg(feature = "compression-bzip2")]
        pub use bzip2;

        #[cfg(feature = "compression-deflate")]
        pub use flate2 as deflate;

        #[cfg(feature = "compression-zstd")]
        pub use zstd;
    }
}
