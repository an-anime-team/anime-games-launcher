pub mod card;
pub mod cards_list;
pub mod cards_row;
pub mod downloads_row;
pub mod game_details;
pub mod game_tags;
pub mod graph;
pub mod maintainers_row;
pub mod requirements;

pub mod prelude {
    pub use super::card::{
        CardComponent, CardComponentInput, CardComponentOutput, DEFAULT_SIZE as CARD_DEFAULT_SIZE,
        MEDIUM_SIZE as CARD_MEDIUM_SIZE, SMALL_SIZE as CARD_SMALL_SIZE,
    };

    pub use super::cards_list::{
        CardsListFactory, CardsListFactoryInit, CardsListFactoryInput, CardsListFactoryOutput,
    };

    pub use super::game_details::{GameDetails, GameDetailsInput, GameDetailsOutput};
}
