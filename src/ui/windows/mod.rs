mod loading_window;
mod main;
mod create_wine_profile;

pub use loading_window::LoadingWindow;

pub mod prelude {
    pub use super::main::{
        MainApp,
        MainAppMsg,
        WINDOW as MAIN_APP_WINDOW
    };

    pub use super::create_wine_profile::{
        CreateWineProfileApp,
        CreateWineProfileAppMsg,
        WINDOW as CREATE_WINE_PROFILE_WINDOW
    };
}
