/// Version of the wineyard runtime library.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod hash;
pub mod packages;

#[cfg(feature = "runtime")]
pub mod runtime;
