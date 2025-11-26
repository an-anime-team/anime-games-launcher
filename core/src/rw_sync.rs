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
    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
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
    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        self.inner()?.write_vectored(bufs)
    }

    #[inline]
    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        self.inner()?.write_fmt(args)
    }
}
