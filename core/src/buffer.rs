use std::io::{Read, Write};

/// Simple bytes container which appends bytes on `Write` trait use, and pops
/// them on `Read` trait use.
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Buffer(Vec<u8>);

impl Buffer {
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Read for Buffer {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = self.0.len().min(buf.len());

        buf[..len].copy_from_slice(&self.0[..len]);

        self.0 = self.0.drain(len..).collect();

        Ok(len)
    }
}

impl Write for Buffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.extend_from_slice(buf);

        Ok(buf.len())
    }

    #[inline(always)]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl AsRef<[u8]> for Buffer {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsMut<Vec<u8>> for Buffer {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut Vec<u8> {
        &mut self.0
    }
}
