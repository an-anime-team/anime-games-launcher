use relm4::prelude::*;

use crate::components::dxvk::Dxvk;
use crate::components::wine::Wine;

use crate::config;
use crate::config::components::wine::prefix::Prefix;

use super::*;

const TOTAL_STEPS: f64 = 11.0;

#[derive(Debug)]
pub struct LoadingResult {
    pub download_wine: Option<Wine>,
    pub download_dxvk: Option<Dxvk>,
    pub apply_dxvk: Option<Dxvk>,
    pub create_prefix: Option<Prefix>,
    pub download_addons: Vec<check_addons::AddonsListEntry>,

    pub games_list: init_games::GamesList
}

pub fn load_app(sender: &AsyncComponentSender<LoadingApp>) -> Result<LoadingResult, LoadingAppMsg> {
    let begin = std::time::Instant::now();

    sender.input(LoadingAppMsg::SetProgress(0.0));
    sender.input(LoadingAppMsg::SetActiveStage(String::from("Preparing default folders")));

    check_default_dirs::check_default_dirs().map_err(|err| LoadingAppMsg::DisplayError {
        title: String::from("Failed to prepare default folders"),
        message: err.to_string()
    })?;

    sender.input(LoadingAppMsg::SetProgress(1.0 / TOTAL_STEPS));
    sender.input(LoadingAppMsg::SetActiveStage(String::from("Preparing config file")));

    init_config::init_config().map_err(|err| LoadingAppMsg::DisplayError {
        title: String::from("Failed to prepare config file"),
        message: err.to_string()
    })?;

    sender.input(LoadingAppMsg::SetProgress(2.0 / TOTAL_STEPS));
    sender.input(LoadingAppMsg::SetActiveStage(String::from("Updating integration scripts")));

    let config = config::get();

    let pool = rusty_pool::Builder::new()
        .name(String::from("load_app"))
        .core_size(config.general.threads.number as usize)
        .build();

    update_integrations::update_integrations(&pool).map_err(|err| LoadingAppMsg::DisplayError {
        title: String::from("Failed to update integration scripts"),
        message: err.to_string()
    })?;

    sender.input(LoadingAppMsg::SetProgress(3.0 / TOTAL_STEPS));
    sender.input(LoadingAppMsg::SetActiveStage(String::from("Preparing games")));

    init_games::init_games().map_err(|err| LoadingAppMsg::DisplayError {
        title: String::from("Failed to prepare games"),
        message: err.to_string()
    })?;

    sender.input(LoadingAppMsg::SetProgress(4.0 / TOTAL_STEPS));
    sender.input(LoadingAppMsg::SetActiveStage(String::from("Preparing games list")));

    let games_list = init_games::get_games_list().map_err(|err| LoadingAppMsg::DisplayError {
        title: String::from("Failed to prepare games list"),
        message: err.to_string()
    })?;

    sender.input(LoadingAppMsg::SetProgress(5.0 / TOTAL_STEPS));
    sender.input(LoadingAppMsg::SetActiveStage(String::from("Registering games styles")));

    init_games::register_games_styles().map_err(|err| LoadingAppMsg::DisplayError {
        title: String::from("Failed to register games styles"),
        message: err.to_string()
    })?;

    sender.input(LoadingAppMsg::SetProgress(6.0 / TOTAL_STEPS));
    sender.input(LoadingAppMsg::SetActiveStage(String::from("Checking wine version")));

    let download_wine = check_wine::get_download().map_err(|err| LoadingAppMsg::DisplayError {
        title: String::from("Failed to check wine version"),
        message: err.to_string()
    })?;

    sender.input(LoadingAppMsg::SetProgress(7.0 / TOTAL_STEPS));
    sender.input(LoadingAppMsg::SetActiveStage(String::from("Checking dxvk version")));

    let download_dxvk = check_dxvk::get_download().map_err(|err| LoadingAppMsg::DisplayError {
        title: String::from("Failed to check dxvk version"),
        message: err.to_string()
    })?;

    sender.input(LoadingAppMsg::SetProgress(8.0 / TOTAL_STEPS));
    sender.input(LoadingAppMsg::SetActiveStage(String::from("Checking applied dxvk version")));

    let apply_dxvk = check_dxvk::get_apply().map_err(|err| LoadingAppMsg::DisplayError {
        title: String::from("Failed to check dxvk version"),
        message: err.to_string()
    })?;

    sender.input(LoadingAppMsg::SetProgress(9.0 / TOTAL_STEPS));
    sender.input(LoadingAppMsg::SetActiveStage(String::from("Checking wine prefix")));

    let create_prefix = check_wine_prefix::check_wine_prefix();

    sender.input(LoadingAppMsg::SetProgress(10.0 / TOTAL_STEPS));
    sender.input(LoadingAppMsg::SetActiveStage(String::from("Checking games addons")));

    let download_addons = check_addons::check_addons(&pool).map_err(|err| LoadingAppMsg::DisplayError {
        title: String::from("Failed to check games addons"),
        message: err.to_string()
    })?;

    sender.input(LoadingAppMsg::SetProgress(1.0));

    // TODO: pulse progress bar before it's joined
    pool.join();

    println!("Launcher loading time: {} ms", begin.elapsed().as_millis());

    Ok(LoadingResult {
        download_wine,
        download_dxvk,
        apply_dxvk,
        create_prefix,
        download_addons,

        games_list
    })
}
