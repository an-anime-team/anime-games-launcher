pub mod lazy_picture;
pub mod card;
pub mod cards_list;
pub mod cards_grid;

pub mod hardware_requirements;

// FIXME: NOT REFACTORED
pub mod downloads_row;
pub mod game_details;
pub mod game_tags;
pub mod graph;
pub mod maintainers_row;
pub mod picture_carousel;

pub use lazy_picture::{
    ImagePath,
    LazyPictureComponent,
    LazyPictureComponentMsg
};

pub use card::{
    CardSize,
    CardComponent,
    CardComponentInput,
    CardComponentOutput
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
