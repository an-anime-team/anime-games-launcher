pub mod lazy_picture;
pub mod picture_carousel;
pub mod card;
pub mod cards_list;
pub mod cards_grid;
pub mod graph;

pub mod game_store_details;
pub mod hardware_requirements;

// FIXME: NOT REFACTORED

pub mod downloads_row;
pub mod game_library_details;
pub mod game_tags;
pub mod maintainers_row;

pub mod prelude {
    pub use super::lazy_picture::{
        ImagePath,
        LazyPictureComponent,
        LazyPictureComponentMsg
    };

    pub use super::picture_carousel::{
        PictureCarousel,
        PictureCarouselMsg
    };

    pub use super::card::{
        CardSize,
        CardComponent,
        CardComponentInput,
        CardComponentOutput
    };

    pub use super::cards_list::{
        CardsList,
        CardsListInit,
        CardsListInput,
        CardsListOutput
    };

    pub use super::cards_grid::{
        CardsGrid,
        CardsGridOutput
    };

    pub use super::graph::{
        Graph,
        GraphInit,
        GraphMsg
    };

    pub use super::game_store_details::{
        GameStoreDetails,
        GameStoreDetailsMsg
    };

    pub use super::game_library_details::{
        GameLibraryDetails,
        GameLibraryDetailsMsg
    };

    pub use super::hardware_requirements::{
        HardwareRequirementsComponent,
        HardwareRequirementsComponentMsg,
        HardwareRequirementsSection,
        HardwareRequirementsSectionMsg
    };

    // FIXME: NOT REFACTORED

    pub use super::game_tags::GameTagFactory;

    pub use super::downloads_row::{
        DownloadsRow,
        DownloadsRowInit,
        DownloadsRowMsg,

        DownloadsRowFactory,
        DownloadsRowFactoryOutput,
        DownloadsRowFactoryMsg
    };

    pub use super::maintainers_row::{
        MaintainersRowFactory,
        MaintainersRowFactoryMsg
    };
}
