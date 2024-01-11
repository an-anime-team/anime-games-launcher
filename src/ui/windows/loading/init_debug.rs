use tracing_subscriber::prelude::*;
use tracing_subscriber::filter::*;

use crate::{
    DEBUG_FILE,
    APP_DEBUG
};

#[inline]
pub fn init_debug() -> anyhow::Result<()> {
    // Prepare stdout logger
    let stdout = tracing_subscriber::fmt::layer()
        .pretty()
        .with_filter({
            if *APP_DEBUG {
                LevelFilter::TRACE
            } else {
                LevelFilter::WARN
            }
        })
        .with_filter(filter_fn(move |metadata| {
            !metadata.target().contains("rustls")
        }));

    // Prepare debug file logger
    let file = std::fs::File::create(DEBUG_FILE.as_path())?;

    let debug_log = tracing_subscriber::fmt::layer()
        .pretty()
        .with_ansi(false)
        .with_writer(std::sync::Arc::new(file))
        .with_filter(filter_fn(|metadata| {
            !metadata.target().contains("rustls")
        }));

    tracing_subscriber::registry()
        .with(stdout)
        .with(debug_log)
        .init();

    Ok(())
}
