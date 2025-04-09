pub mod main_window;
pub mod download_manager;

pub mod prelude {
    pub use super::main_window::{
        MainWindow,
        MainWindowMsg
    };

    pub use super::main_window::library_page::SyncGameCommand;

    pub use super::download_manager::{
        DownloadManagerWindow,
        DownloadManagerWindowMsg
    };
}
