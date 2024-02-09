pub mod card;
pub mod cards_list;
pub mod game_details;

pub mod prelude {
    pub use super::card::{
        CardComponent,
        CardComponentInput,
        CardComponentOutput
    };

    pub use super::cards_list::{
        CardsListFactory,
        CardsListFactoryInit,
        CardsListFactoryInput,
        CardsListFactoryOutput
    };

    pub use super::game_details::{
        GameDetails,
        GameDetailsInput,
        GameDetailsOutput
    };
}
