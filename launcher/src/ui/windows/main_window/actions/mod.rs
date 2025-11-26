pub mod fetch_games;
pub mod serve_generation;

pub mod prelude {
    pub use super::fetch_games::fetch_games;

    pub use super::serve_generation::{
        serve_generation,
        SyncGameCommand
    };
}
