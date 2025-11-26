mod error;
mod algorithm;
mod level;
mod compressor;
mod decompressor;

#[cfg(test)]
mod tests;

pub use error::CompressionError;
pub use level::CompressionLevel;
pub use algorithm::CompressionAlgorithm;
pub use compressor::Compressor;
pub use decompressor::Decompressor;

// TODO: compressor is done, but decompressor is BROKEN !
