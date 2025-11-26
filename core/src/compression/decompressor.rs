use std::io::{BufReader, Read, Write};

use crate::rw_sync::ReadWriteMutex;
use crate::buffer::Buffer;

use super::*;

pub enum Decompressor {
    #[cfg(feature = "compression-lz4")]
    Lz4(lz4_flex::frame::FrameDecoder<Buffer>),

    #[cfg(feature = "compression-bzip2")]
    Bzip2(bzip2::read::MultiBzDecoder<Buffer>),

    #[cfg(feature = "compression-deflate")]
    Deflate(flate2::read::DeflateDecoder<Buffer>),

    #[cfg(feature = "compression-deflate")]
    Gzip(flate2::write::MultiGzDecoder<Buffer>),

    #[cfg(feature = "compression-deflate")]
    Zlib(flate2::read::ZlibDecoder<Buffer>),

    #[cfg(feature = "compression-zstd")]
    Zstd {
        buf: ReadWriteMutex<Buffer>,
        decompressor: zstd::Decoder<'static, BufReader<ReadWriteMutex<Buffer>>>
    }
}

impl Decompressor {
    /// Create new decompressor for the given algorithm.
    pub fn new(
        algorithm: impl Into<CompressionAlgorithm>
    ) -> Result<Self, CompressionError> {
        let algorithm: CompressionAlgorithm = algorithm.into();
        let buf = Buffer::default();

        #[cfg(feature = "tracing")]
        tracing::trace!(?algorithm, "create decompressor");

        match algorithm {
            #[cfg(feature = "compression-lz4")]
            CompressionAlgorithm::Lz4 => {
                let decompressor = lz4_flex::frame::FrameDecoder::new(buf);

                Ok(Self::Lz4(decompressor))
            }

            #[cfg(feature = "compression-bzip2")]
            CompressionAlgorithm::Bzip2 => {
                let decompressor = bzip2::read::MultiBzDecoder::new(buf);

                Ok(Self::Bzip2(decompressor))
            }

            #[cfg(feature = "compression-deflate")]
            CompressionAlgorithm::Deflate => {
                let decompressor = flate2::read::DeflateDecoder::new(buf);

                Ok(Self::Deflate(decompressor))
            }

            #[cfg(feature = "compression-deflate")]
            CompressionAlgorithm::Gzip => {
                let decompressor = flate2::write::MultiGzDecoder::new(buf);

                Ok(Self::Gzip(decompressor))
            }

            #[cfg(feature = "compression-deflate")]
            CompressionAlgorithm::Zlib => {
                let decompressor = flate2::read::ZlibDecoder::new(buf);

                Ok(Self::Zlib(decompressor))
            }

            #[cfg(feature = "compression-zstd")]
            CompressionAlgorithm::Zstd => {
                let buf = ReadWriteMutex::new(buf);
                let decompressor = zstd::Decoder::new(buf.clone())?;

                Ok(Self::Zstd {
                    buf,
                    decompressor
                })
            }
        }
    }

    /// Get compression algorithm from the current decompressor struct.
    pub const fn algorithm(&self) -> CompressionAlgorithm {
        match self {
            #[cfg(feature = "compression-lz4")]
            Self::Lz4(_) => CompressionAlgorithm::Lz4,

            #[cfg(feature = "compression-bzip2")]
            Self::Bzip2(_) => CompressionAlgorithm::Bzip2,

            #[cfg(feature = "compression-deflate")]
            Self::Deflate(_) => CompressionAlgorithm::Deflate,

            #[cfg(feature = "compression-deflate")]
            Self::Gzip(_) => CompressionAlgorithm::Gzip,

            #[cfg(feature = "compression-deflate")]
            Self::Zlib(_) => CompressionAlgorithm::Zlib,

            #[cfg(feature = "compression-zstd")]
            Self::Zstd { .. } => CompressionAlgorithm::Zstd
        }
    }
}

impl Write for Decompressor {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            #[cfg(feature = "compression-lz4")]
            Self::Lz4(decompressor) => decompressor.get_mut().write(buf),

            #[cfg(feature = "compression-bzip2")]
            Self::Bzip2(decompressor) => decompressor.get_mut().write(buf),

            #[cfg(feature = "compression-deflate")]
            Self::Deflate(decompressor) => decompressor.get_mut().write(buf),

            #[cfg(feature = "compression-deflate")]
            Self::Gzip(decompressor) => decompressor.write(buf),

            #[cfg(feature = "compression-deflate")]
            Self::Zlib(decompressor) => decompressor.get_mut().write(buf),

            #[cfg(feature = "compression-zstd")]
            Self::Zstd { buf: decompressor_buf, .. } => decompressor_buf.write(buf)
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            #[cfg(feature = "compression-lz4")]
            Self::Lz4(decompressor) => decompressor.get_mut().flush(),

            #[cfg(feature = "compression-bzip2")]
            Self::Bzip2(decompressor) => decompressor.get_mut().flush(),

            #[cfg(feature = "compression-deflate")]
            Self::Deflate(decompressor) => decompressor.get_mut().flush(),

            #[cfg(feature = "compression-deflate")]
            Self::Gzip(decompressor) => decompressor.flush(),

            #[cfg(feature = "compression-deflate")]
            Self::Zlib(decompressor) => decompressor.get_mut().flush(),

            #[cfg(feature = "compression-zstd")]
            Self::Zstd { buf: decompressor_buf, .. } => decompressor_buf.flush()
        }
    }
}

impl Read for Decompressor {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            #[cfg(feature = "compression-lz4")]
            Self::Lz4(decompressor) => decompressor.read(buf),

            #[cfg(feature = "compression-bzip2")]
            Self::Bzip2(decompressor) => decompressor.read(buf),

            #[cfg(feature = "compression-deflate")]
            Self::Deflate(decompressor) => decompressor.read(buf),

            #[cfg(feature = "compression-deflate")]
            Self::Gzip(decompressor) => decompressor.get_mut().read(buf),

            #[cfg(feature = "compression-deflate")]
            Self::Zlib(decompressor) => decompressor.read(buf),

            #[cfg(feature = "compression-zstd")]
            Self::Zstd { decompressor, .. } => decompressor.read(buf)
        }
    }
}

impl std::str::FromStr for Decompressor {
    type Err = CompressionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, _) = s.split_once(":")
            .unwrap_or((s, "default"));

        let algorithm = CompressionAlgorithm::from_str(name)?;

        Self::new(algorithm)
    }
}

impl std::fmt::Debug for Decompressor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "compression-lz4")]
            Self::Lz4(decompressor) => f.debug_struct("Decompressor")
                .field("inner", decompressor)
                .finish(),

            #[cfg(feature = "compression-bzip2")]
            Self::Bzip2(_) => f.debug_struct("Decompressor")
                .field("inner", &"Bzip2" as &dyn std::fmt::Debug)
                .finish(),

            #[cfg(feature = "compression-deflate")]
            Self::Deflate(decompressor) => f.debug_struct("Decompressor")
                .field("inner", decompressor)
                .finish(),

            #[cfg(feature = "compression-deflate")]
            Self::Gzip(decompressor) => f.debug_struct("Decompressor")
                .field("inner", decompressor)
                .finish(),

            #[cfg(feature = "compression-deflate")]
            Self::Zlib(decompressor) => f.debug_struct("Decompressor")
                .field("inner", decompressor)
                .finish(),

            #[cfg(feature = "compression-zstd")]
            Self::Zstd { .. } => f.debug_struct("Decompressor")
                .field("inner", &"Zstd" as &dyn std::fmt::Debug)
                .finish()
        }
    }
}

impl AsRef<Decompressor> for Decompressor {
    #[inline(always)]
    fn as_ref(&self) -> &Decompressor {
        self
    }
}

impl AsMut<Decompressor> for Decompressor {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut Decompressor {
        self
    }
}
