use std::io::{Read, Write};

use super::*;

const LEVELS: &[CompressionLevel] = &[
    CompressionLevel::Quick,
    CompressionLevel::Fast,
    CompressionLevel::Balanced,
    CompressionLevel::Good,
    CompressionLevel::Best,
    CompressionLevel::Default
];

#[allow(unused)]
fn test(algorithm: CompressionAlgorithm) -> Result<(), CompressionError> {
    for level in LEVELS {
        let mut compressor = Compressor::new(algorithm, *level)?;
        let mut decompressor = Decompressor::new(algorithm)?;

        compressor.write_all(b"AAAAAAAAAAAAAAAAAAAA")?;
        compressor.write_all(b"AAAAAAAAAAAAAAAAAAAA")?;
        compressor.write_all(b"AAAAAAAAAAAAAAAAAAAA")?;
        compressor.write_all(b"AAAAAAAAAAAAAAAAAAAA")?;
        compressor.write_all(b"AAAAAAAAAAAAAAAAAAAA")?;
        compressor.flush()?;
        compressor.try_finish()?;

        let mut compressed = Vec::new();

        compressor.read_to_end(&mut compressed)?;

        assert!(compressed.len() < 100);

        decompressor.write_all(&compressed)?;
        decompressor.flush()?;

        let mut decompressed = Vec::new();

        decompressor.read_to_end(&mut decompressed)?;

        assert_eq!(decompressed, b"AAAAAAAAAAAAAAAAAAAA".repeat(5));
    }

    Ok(())
}

#[cfg(feature = "compression-lz4")]
#[test]
fn lz4() -> Result<(), CompressionError> {
    test(CompressionAlgorithm::Lz4)
}

#[cfg(feature = "compression-bzip2")]
#[test]
fn bzip2() -> Result<(), CompressionError> {
    test(CompressionAlgorithm::Bzip2)
}

#[cfg(feature = "compression-deflate")]
#[test]
fn deflate() -> Result<(), CompressionError> {
    test(CompressionAlgorithm::Deflate)
}

#[cfg(feature = "compression-deflate")]
#[test]
fn gzip() -> Result<(), CompressionError> {
    test(CompressionAlgorithm::Gzip)
}

#[cfg(feature = "compression-deflate")]
#[test]
fn zlib() -> Result<(), CompressionError> {
    test(CompressionAlgorithm::Zlib)
}

#[cfg(feature = "compression-zstd")]
#[test]
fn zstd() -> Result<(), CompressionError> {
    test(CompressionAlgorithm::Zstd)
}
