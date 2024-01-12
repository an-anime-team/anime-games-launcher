use relm4::prelude::*;

use crate::tr;

use crate::components::dxvk::Dxvk;
use crate::components::wine::Wine;

use crate::config::components::wine::prefix::Prefix;

use super::*;

const TOTAL_STEPS: f64 = 13.0;

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
    sender.input(LoadingAppMsg::SetActiveStage(tr!("loading-preparing-folders")));

    check_default_dirs::check_default_dirs().map_err(|err| LoadingAppMsg::DisplayError {
        title: tr!("loading-preparing-folders-failed"),
        message: err.to_string()
    })?;

    sender.input(LoadingAppMsg::SetProgress(1.0));
    sender.input(LoadingAppMsg::SetActiveStage(tr!("loading-initializing-debug")));

    init_debug::init_debug().map_err(|err| LoadingAppMsg::DisplayError {
        title: tr!("loading-initializing-debug-failed"),
        message: err.to_string()
    })?;

    sender.input(LoadingAppMsg::SetProgress(2.0 / TOTAL_STEPS));
    sender.input(LoadingAppMsg::SetActiveStage(tr!("loading-preparing-config")));

    let config = init_config::init_config().map_err(|err| LoadingAppMsg::DisplayError {
        title: tr!("loading-preparing-config-failed"),
        message: err.to_string()
    })?;

    sender.input(LoadingAppMsg::SetProgress(3.0));
    sender.input(LoadingAppMsg::SetActiveStage(tr!("loading-initializing-locales")));

    init_locales::init_locales(&config).map_err(|err| LoadingAppMsg::DisplayError {
        title: tr!("loading-initializing-locales-failed"),
        message: err.to_string()
    })?;

    sender.input(LoadingAppMsg::SetProgress(4.0 / TOTAL_STEPS));
    sender.input(LoadingAppMsg::SetActiveStage(tr!("loading-updating-integrations")));

    let pool = rusty_pool::Builder::new()
        .name(String::from("load_app"))
        .core_size(config.general.threads.number as usize)
        .build();

    update_integrations::update_integrations(&pool).map_err(|err| LoadingAppMsg::DisplayError {
        title: tr!("loading-updating-integrations-failed"),
        message: err.to_string()
    })?;

    sender.input(LoadingAppMsg::SetProgress(5.0 / TOTAL_STEPS));
    sender.input(LoadingAppMsg::SetActiveStage(tr!("loading-preparing-games")));

    init_games::init_games().map_err(|err| LoadingAppMsg::DisplayError {
        title: tr!("loading-preparing-games-failed"),
        message: err.to_string()
    })?;

    sender.input(LoadingAppMsg::SetProgress(6.0 / TOTAL_STEPS));
    sender.input(LoadingAppMsg::SetActiveStage(tr!("loading-preparing-games-list")));

    let games_list = init_games::get_games_list().map_err(|err| LoadingAppMsg::DisplayError {
        title: tr!("loading-preparing-games-list-failed"),
        message: err.to_string()
    })?;

    sender.input(LoadingAppMsg::SetProgress(7.0 / TOTAL_STEPS));
    sender.input(LoadingAppMsg::SetActiveStage(tr!("loading-registering-styles")));

    init_games::register_games_styles().map_err(|err| LoadingAppMsg::DisplayError {
        title: tr!("loading-registering-styles-failed"),
        message: err.to_string()
    })?;

    sender.input(LoadingAppMsg::SetProgress(8.0 / TOTAL_STEPS));
    sender.input(LoadingAppMsg::SetActiveStage(tr!("loading-checking-wine-version")));

    let download_wine = check_wine::get_download().map_err(|err| LoadingAppMsg::DisplayError {
        title: tr!("loading-checking-wine-version-failed"),
        message: err.to_string()
    })?;

    sender.input(LoadingAppMsg::SetProgress(9.0 / TOTAL_STEPS));
    sender.input(LoadingAppMsg::SetActiveStage(tr!("loading-checking-dxvk-version")));

    let download_dxvk = check_dxvk::get_download().map_err(|err| LoadingAppMsg::DisplayError {
        title: tr!("loading-checking-dxvk-version-failed"),
        message: err.to_string()
    })?;

    sender.input(LoadingAppMsg::SetProgress(10.0 / TOTAL_STEPS));
    sender.input(LoadingAppMsg::SetActiveStage(tr!("loading-checking-applied-dxvk")));

    let apply_dxvk = check_dxvk::get_apply().map_err(|err| LoadingAppMsg::DisplayError {
        title: tr!("loading-checking-applied-dxvk-failed"),
        message: err.to_string()
    })?;

    sender.input(LoadingAppMsg::SetProgress(11.0 / TOTAL_STEPS));
    sender.input(LoadingAppMsg::SetActiveStage(tr!("loading-checking-wine-prefix")));

    let create_prefix = check_wine_prefix::check_wine_prefix();

    sender.input(LoadingAppMsg::SetProgress(12.0 / TOTAL_STEPS));
    sender.input(LoadingAppMsg::SetActiveStage(tr!("loading-checking-games-addons")));

    let download_addons = check_addons::get_download(&pool).map_err(|err| LoadingAppMsg::DisplayError {
        title: tr!("loading-checking-games-addons-failed"),
        message: err.to_string()
    })?;

    sender.input(LoadingAppMsg::SetProgress(1.0));

    // TODO: pulse progress bar before it's joined
    pool.join();

    tracing::info!("Launcher loaded in {} ms", begin.elapsed().as_millis());

    Ok(LoadingResult {
        download_wine,
        download_dxvk,
        apply_dxvk,
        create_prefix,
        download_addons,

        games_list
    })
}
