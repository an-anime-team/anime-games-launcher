pub mod card;
pub mod cards_list;
pub mod cards_grid;

// FIXME: NOT REFACTORED
pub mod downloads_row;
pub mod game_details;
pub mod game_tags;
pub mod graph;
pub mod maintainers_row;
pub mod picture_carousel;
pub mod requirements;

pub use card::{
    CardComponent,
    CardComponentInput,
    CardComponentOutput,

    DEFAULT_SIZE as CARD_DEFAULT_SIZE,
    SMALL_SIZE as CARD_SMALL_SIZE,
    MEDIUM_SIZE as CARD_MEDIUM_SIZE
};

pub use cards_list::{
    CardsList,
    CardsListInit,
    CardsListInput,
    CardsListOutput
};

pub use cards_grid::{
    CardsGrid,
    CardsGridOutput
};

// FIXME: NOT REFACTORED
pub use downloads_row::{
    DownloadsRow,
    DownloadsRowInit,
    DownloadsRowMsg,

    DownloadsRowFactory,
    DownloadsRowFactoryOutput,
    DownloadsRowFactoryMsg
};

pub use game_details::{
    GameDetails,
    GameDetailsInit,
    GameDetailsInput
};
