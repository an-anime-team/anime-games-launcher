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

use std::io::{Read, Write};
use std::sync::{Arc, Mutex, MutexGuard};

/// Shared read-write interface hidden behind `Arc<Mutex<T>>`. Can be used if
/// you need to read and write data from multiple places.
#[derive(Default, Debug, Clone)]
pub struct ReadWriteMutex<T: Read + Write>(Arc<Mutex<T>>);

impl<T: Read + Write> ReadWriteMutex<T> {
    #[inline]
    pub fn new(inner: T) -> Self {
        Self(Arc::new(Mutex::new(inner)))
    }

    pub fn inner(&mut self) -> std::io::Result<MutexGuard<'_, T>> {
        self.0.lock()
            .map_err(|err| {
                std::io::Error::other(format!("failed to lock mutex: {err}"))
            })
    }
}

impl<T: Read + Write> Read for ReadWriteMutex<T> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner()?.read(buf)
    }

    #[inline]
    fn read_vectored(
        &mut self,
        bufs: &mut [std::io::IoSliceMut<'_>]
    ) -> std::io::Result<usize> {
        self.inner()?.read_vectored(bufs)
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        self.inner()?.read_to_end(buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> std::io::Result<usize> {
        self.inner()?.read_to_string(buf)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        self.inner()?.read_exact(buf)
    }
}

impl<T: Read + Write> Write for ReadWriteMutex<T> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner()?.write(buf)
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.inner()?.write_all(buf)
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner()?.flush()
    }

    #[inline]
    fn write_vectored(
        &mut self,
        bufs: &[std::io::IoSlice<'_>]
    ) -> std::io::Result<usize> {
        self.inner()?.write_vectored(bufs)
    }

    #[inline]
    fn write_fmt(
        &mut self,
        args: std::fmt::Arguments<'_>
    ) -> std::io::Result<()> {
        self.inner()?.write_fmt(args)
    }
}
