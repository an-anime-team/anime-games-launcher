pub mod lazy_picture;
pub mod picture_carousel;
pub mod card;
pub mod cards_list;
pub mod cards_grid;
pub mod graph;

pub mod game_store_details;
pub mod hardware_requirements;

pub use lazy_picture::{
    ImagePath,
    LazyPictureComponent,
    LazyPictureComponentMsg
};

pub use picture_carousel::{
    PictureCarousel,
    PictureCarouselMsg
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

pub use graph::{
    Graph,
    GraphInit,
    GraphMsg
};

pub use game_store_details::{
    GameStoreDetails,
    GameStoreDetailsMsg
};

pub use hardware_requirements::{
    HardwareRequirementsComponent,
    HardwareRequirementsComponentMsg,
    HardwareRequirementsSection,
    HardwareRequirementsSectionMsg
};

// FIXME: NOT REFACTORED
pub mod downloads_row;
pub mod game_library_details;
pub mod game_tags;
pub mod maintainers_row;

pub use game_tags::GameTagFactory;

pub use downloads_row::{
    DownloadsRow,
    DownloadsRowInit,
    DownloadsRowMsg,

    DownloadsRowFactory,
    DownloadsRowFactoryOutput,
    DownloadsRowFactoryMsg
};

pub use game_library_details::{
    GameLibraryDetails,
    GameLibraryDetailsMsg
};

pub use maintainers_row::{
    MaintainersRowFactory,
    MaintainersRowFactoryMsg
};
