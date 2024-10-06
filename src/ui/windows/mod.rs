mod loading_window;
mod main_window;
mod create_wine_profile;

pub use loading_window::{
    LoadingWindow,
    LoadingWindowMsg
};

pub use main_window::{
    MainWindow,
    MainWindowMsg
};

pub mod prelude {
    pub use super::create_wine_profile::{
        CreateWineProfileApp,
        CreateWineProfileAppMsg,
        WINDOW as CREATE_WINE_PROFILE_WINDOW
    };
}
