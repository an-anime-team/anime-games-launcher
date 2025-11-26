pub use tokio::{fs, io, net, sync};
pub use tokio::task::{JoinHandle, JoinError};

use tokio::runtime::{Runtime, Builder};

lazy_static::lazy_static! {
    pub static ref RUNTIME: Runtime = Builder::new_multi_thread()
        .thread_name("wineyard_core")
        .enable_all()
        .build()
        .expect("failed to initialize tokio runtime");
}

/// Spawn future in the shared tokio runtime.
#[inline(always)]
pub fn spawn<T: Send + 'static>(
    future: impl Future<Output = T> + Send + 'static
) -> JoinHandle<T> {
    RUNTIME.spawn(future)
}

/// Block current thread to execute the future.
#[inline(always)]
pub fn block_on<T>(future: impl Future<Output = T>) -> T {
    RUNTIME.block_on(future)
}
