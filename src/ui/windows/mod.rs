mod main_window;
mod profiles_window;
mod download_manager;
mod create_wine_profile;

pub use main_window::{
    MainWindow,
    MainWindowMsg,
    WINDOW as MAIN_WINDOW
};

pub use main_window::library_page::SyncGameCommand;

pub use download_manager::{
    DownloadManagerWindow,
    DownloadManagerWindowMsg
};
