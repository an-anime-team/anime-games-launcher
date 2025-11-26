use std::io::{Read, Write};

use crate::buffer::Buffer;

use super::*;

pub enum Compressor {
    #[cfg(feature = "compression-lz4")]
    Lz4(lz4_flex::frame::FrameEncoder<Buffer>),

    #[cfg(feature = "compression-bzip2")]
    Bzip2(bzip2::write::BzEncoder<Buffer>),

    #[cfg(feature = "compression-deflate")]
    Deflate(flate2::write::DeflateEncoder<Buffer>),

    #[cfg(feature = "compression-deflate")]
    Gzip(flate2::write::GzEncoder<Buffer>),

    #[cfg(feature = "compression-deflate")]
    Zlib(flate2::write::ZlibEncoder<Buffer>),

    #[cfg(feature = "compression-zstd")]
    Zstd(zstd::Encoder<'static, Buffer>)
}

impl Compressor {
    /// Create new compressor from the given algorithm and compression level.
    pub fn new(
        algorithm: impl Into<CompressionAlgorithm>,
        level: impl Into<CompressionLevel>
    ) -> Result<Self, CompressionError> {
        let algorithm: CompressionAlgorithm = algorithm.into();
        let level: CompressionLevel = level.into();

        #[cfg(feature = "tracing")]
        tracing::trace!(?algorithm, ?level, "create compressor");

        let buf = Buffer::default();

        match algorithm {
            #[cfg(feature = "compression-lz4")]
            CompressionAlgorithm::Lz4 => {
                let compressor = lz4_flex::frame::FrameEncoder::new(buf);

                Ok(Self::Lz4(compressor))
            }

            #[cfg(feature = "compression-bzip2")]
            CompressionAlgorithm::Bzip2 => {
                let compressor = bzip2::write::BzEncoder::new(buf, level.into());

                Ok(Self::Bzip2(compressor))
            }

            #[cfg(feature = "compression-deflate")]
            CompressionAlgorithm::Deflate => {
                let compressor = flate2::write::DeflateEncoder::new(buf, level.into());

                Ok(Self::Deflate(compressor))
            }

            #[cfg(feature = "compression-deflate")]
            CompressionAlgorithm::Gzip => {
                let compressor = flate2::write::GzEncoder::new(buf, level.into());

                Ok(Self::Gzip(compressor))
            }

            #[cfg(feature = "compression-deflate")]
            CompressionAlgorithm::Zlib => {
                let compressor = flate2::write::ZlibEncoder::new(buf, level.into());

                Ok(Self::Zlib(compressor))
            }

            #[cfg(feature = "compression-zstd")]
            CompressionAlgorithm::Zstd => {
                let compressor = zstd::Encoder::new(buf, level.zstd_level())?;

                Ok(Self::Zstd(compressor))
            }
        }
    }

    /// Get compression algorithm from the current compressor struct.
    pub const fn algorithm(&self) -> CompressionAlgorithm {
        match self {
            #[cfg(feature = "compression-lz4")]
            Self::Lz4 { .. } => CompressionAlgorithm::Lz4,

            #[cfg(feature = "compression-bzip2")]
            Self::Bzip2 { .. } => CompressionAlgorithm::Bzip2,

            #[cfg(feature = "compression-deflate")]
            Self::Deflate { .. } => CompressionAlgorithm::Deflate,

            #[cfg(feature = "compression-deflate")]
            Self::Gzip { .. } => CompressionAlgorithm::Gzip,

            #[cfg(feature = "compression-deflate")]
            Self::Zlib { .. } => CompressionAlgorithm::Zlib,

            #[cfg(feature = "compression-zstd")]
            Self::Zstd { .. } => CompressionAlgorithm::Zstd
        }
    }

    /// Get compressor variant which will automatically finish stream on drop.
    #[inline(always)]
    pub const fn auto_finish(self) -> AutoFinishCompressor {
        AutoFinishCompressor(self)
    }

    /// Append stream termination bytes to the compressed data Bufferfer.
    /// This method is called automatically if compressor is dropped.
    pub fn try_finish(&mut self) -> std::io::Result<()> {
        match self {
            #[cfg(feature = "compression-lz4")]
            Self::Lz4(compressor) => compressor.try_finish()?,

            #[cfg(feature = "compression-bzip2")]
            Self::Bzip2(compressor) => compressor.try_finish()?,

            #[cfg(feature = "compression-deflate")]
            Self::Deflate(compressor) => compressor.try_finish()?,

            #[cfg(feature = "compression-deflate")]
            Self::Gzip(compressor) => compressor.try_finish()?,

            #[cfg(feature = "compression-deflate")]
            Self::Zlib(compressor) => compressor.try_finish()?,

            #[cfg(feature = "compression-zstd")]
            Self::Zstd(compressor) => compressor.do_finish()?
        }

        Ok(())
    }
}

impl Write for Compressor {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            #[cfg(feature = "compression-lz4")]
            Self::Lz4(compressor) => compressor.write(buf),

            #[cfg(feature = "compression-bzip2")]
            Self::Bzip2(compressor) => compressor.write(buf),

            #[cfg(feature = "compression-deflate")]
            Self::Deflate(compressor) => compressor.write(buf),

            #[cfg(feature = "compression-deflate")]
            Self::Gzip(compressor) => compressor.write(buf),

            #[cfg(feature = "compression-deflate")]
            Self::Zlib(compressor) => compressor.write(buf),

            #[cfg(feature = "compression-zstd")]
            Self::Zstd(compressor) => compressor.write(buf)
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            #[cfg(feature = "compression-lz4")]
            Self::Lz4(compressor) => compressor.flush(),

            #[cfg(feature = "compression-bzip2")]
            Self::Bzip2(compressor) => compressor.flush(),

            #[cfg(feature = "compression-deflate")]
            Self::Deflate(compressor) => compressor.flush(),

            #[cfg(feature = "compression-deflate")]
            Self::Gzip(compressor) => compressor.flush(),

            #[cfg(feature = "compression-deflate")]
            Self::Zlib(compressor) => compressor.flush(),

            #[cfg(feature = "compression-zstd")]
            Self::Zstd(compressor) => compressor.flush()
        }
    }
}

impl Read for Compressor {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            #[cfg(feature = "compression-lz4")]
            Self::Lz4(compressor) => compressor.get_mut().read(buf),

            #[cfg(feature = "compression-bzip2")]
            Self::Bzip2(compressor) => compressor.get_mut().read(buf),

            #[cfg(feature = "compression-deflate")]
            Self::Deflate(compressor) => compressor.get_mut().read(buf),

            #[cfg(feature = "compression-deflate")]
            Self::Gzip(compressor) => compressor.get_mut().read(buf),

            #[cfg(feature = "compression-deflate")]
            Self::Zlib(compressor) => compressor.get_mut().read(buf),

            #[cfg(feature = "compression-zstd")]
            Self::Zstd(compressor) => compressor.get_mut().read(buf)
        }
    }
}

impl std::str::FromStr for Compressor {
    type Err = CompressionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, level) = s.split_once(":")
            .unwrap_or((s, "default"));

        let algorithm = CompressionAlgorithm::from_str(name)?;
        let level = CompressionLevel::from_str(level)?;

        Self::new(algorithm, level)
    }
}

impl std::fmt::Debug for Compressor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "compression-lz4")]
            Self::Lz4(compressor) => f.debug_struct("Compressor")
                .field("inner", compressor)
                .finish(),

            #[cfg(feature = "compression-bzip2")]
            Self::Bzip2(_) => f.debug_struct("Compressor")
                .field("inner", &"Bzip2" as &dyn std::fmt::Debug)
                .finish(),

            #[cfg(feature = "compression-deflate")]
            Self::Deflate(compressor) => f.debug_struct("Compressor")
                .field("inner", compressor)
                .finish(),

            #[cfg(feature = "compression-deflate")]
            Self::Gzip(compressor) => f.debug_struct("Compressor")
                .field("inner", compressor)
                .finish(),

            #[cfg(feature = "compression-deflate")]
            Self::Zlib(compressor) => f.debug_struct("Compressor")
                .field("inner", compressor)
                .finish(),

            #[cfg(feature = "compression-zstd")]
            Self::Zstd(_) => f.debug_struct("Compressor")
                .field("inner", &"Zstd" as &dyn std::fmt::Debug)
                .finish()
        }
    }
}

impl AsRef<Compressor> for Compressor {
    #[inline(always)]
    fn as_ref(&self) -> &Compressor {
        self
    }
}

impl AsMut<Compressor> for Compressor {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut Compressor {
        self
    }
}

/// Wrapper around the compressor struct which will automatically call
/// `try_finish` method when dropped.
#[derive(Debug)]
pub struct AutoFinishCompressor(Compressor);

impl Write for AutoFinishCompressor {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }

    #[inline(always)]
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

impl Read for AutoFinishCompressor {
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }
}

impl Drop for AutoFinishCompressor {
    #[inline(always)]
    fn drop(&mut self) {
        let _ = self.0.try_finish();
    }
}

impl From<Compressor> for AutoFinishCompressor {
    #[inline(always)]
    fn from(value: Compressor) -> Self {
        Self(value)
    }
}

impl AsRef<AutoFinishCompressor> for AutoFinishCompressor {
    #[inline(always)]
    fn as_ref(&self) -> &AutoFinishCompressor {
        self
    }
}

impl AsMut<AutoFinishCompressor> for AutoFinishCompressor {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut AutoFinishCompressor {
        self
    }
}
