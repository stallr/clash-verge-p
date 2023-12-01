use crate::{
    config::*,
    core::*,
    feat,
    utils::{dirs, help},
};
use crate::{ret_err, wrap_err};
use anyhow::{Context, Result};
use serde_yaml::Mapping;
use std::{collections::{HashMap, VecDeque}, env, process::Command};
use std::path::{Path, PathBuf};
use sysproxy::Sysproxy;


type CmdResult<T = ()> = Result<T, String>;

#[tauri::command]
pub fn check_if_installed_in_applications() -> Result<bool, String> {
    let current_exe = env::current_exe().map_err(|e| e.to_string())?;
    let app_bundle_path = current_exe
        .ancestors()
        .find(|path| path.extension().and_then(|ext| ext.to_str()) == Some("app"))
        .ok_or("Not running from an .app bundle.")?;

    // 检查程序是否已安装在 /Applications
    if app_bundle_path.starts_with("/Applications/") {
        Ok(true) // 已安装在 /Applications
    } else {
        Ok(false) // 未安装在 /Applications
    }
}

fn escape_for_apple_script(path: &Path) -> Result<String, String> {
    path.to_str()
        .map(|s| s.replace("\"", "\\\""))
        .ok_or_else(|| "Failed to convert path to string.".to_string())
}

#[tauri::command]
pub fn move_to_applications() -> Result<bool, String> {
    let current_exe = env::current_exe().map_err(|e| e.to_string())?;
    let app_bundle_path = current_exe
        .ancestors()
        .find(|path| path.extension().and_then(|ext| ext.to_str()) == Some("app"))
        .ok_or("Not running from an .app bundle.")?;

    let app_bundle_path_str = escape_for_apple_script(&app_bundle_path)?;
    let app_name = app_bundle_path.file_name().ok_or("Failed to get app bundle name")?.to_str().ok_or("Failed to convert app bundle name to string")?;
    let target_path = format!("/Applications/{}", app_name);

    let apple_script_command = if PathBuf::from(&target_path).exists() {
        format!(
            "do shell script \"rm -R '{}' && cp -R '{}' '{}' && rm -R '{}'\" with administrator privileges",
            target_path, app_bundle_path_str, target_path, app_bundle_path_str
        )
    } else {
        format!(
            "do shell script \"cp -R '{}' '{}' && rm -R '{}'\" with administrator privileges",
            app_bundle_path_str, target_path, app_bundle_path_str
        )
    };


    let output = Command::new("osascript")
        .arg("-e")
        .arg(apple_script_command)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(true)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
pub fn get_profiles() -> CmdResult<IProfiles> {
    Ok(Config::profiles().data().clone())
}

#[tauri::command]
pub async fn enhance_profiles() -> CmdResult {
    wrap_err!(CoreManager::global().update_config().await)?;
    handle::Handle::refresh_clash();
    Ok(())
}

#[tauri::command]
pub async fn import_profile(url: String, option: Option<PrfOption>) -> CmdResult {
    let item = wrap_err!(PrfItem::from_url(&url, None, None, option).await)?;
    wrap_err!(Config::profiles().data().append_item(item))
}

#[tauri::command]
pub async fn create_profile(item: PrfItem, file_data: Option<String>) -> CmdResult {
    let item = wrap_err!(PrfItem::from(item, file_data).await)?;
    wrap_err!(Config::profiles().data().append_item(item))
}

#[tauri::command]
pub async fn update_profile(index: String, option: Option<PrfOption>) -> CmdResult {
    wrap_err!(feat::update_profile(index, option).await)
}

#[tauri::command]
pub async fn delete_profile(index: String) -> CmdResult {
    let should_update = wrap_err!({ Config::profiles().data().delete_item(index) })?;
    if should_update {
        wrap_err!(CoreManager::global().update_config().await)?;
        handle::Handle::refresh_clash();
    }

    Ok(())
}

/// 修改profiles的
#[tauri::command]
pub async fn patch_profiles_config(profiles: IProfiles) -> CmdResult {
    wrap_err!({ Config::profiles().draft().patch_config(profiles) })?;

    match CoreManager::global().update_config().await {
        Ok(_) => {
            handle::Handle::refresh_clash();
            Config::profiles().apply();
            wrap_err!(Config::profiles().data().save_file())?;
            Ok(())
        }
        Err(err) => {
            Config::profiles().discard();
            log::error!(target: "app", "{err}");
            Err(format!("{err}"))
        }
    }
}

/// 修改某个profile item的
#[tauri::command]
pub fn patch_profile(index: String, profile: PrfItem) -> CmdResult {
    wrap_err!(Config::profiles().data().patch_item(index, profile))?;
    wrap_err!(timer::Timer::global().refresh())
}

#[tauri::command]
pub fn view_profile(index: String) -> CmdResult {
    let file = {
        wrap_err!(Config::profiles().latest().get_item(&index))?
            .file
            .clone()
            .ok_or("the file field is null")
    }?;

    let path = wrap_err!(dirs::app_profiles_dir())?.join(file);
    if !path.exists() {
        ret_err!("the file not found");
    }

    wrap_err!(help::open_file(path))
}

#[tauri::command]
pub fn read_profile_file(index: String) -> CmdResult<String> {
    let profiles = Config::profiles();
    let profiles = profiles.latest();
    let item = wrap_err!(profiles.get_item(&index))?;
    let data = wrap_err!(item.read_file())?;
    Ok(data)
}

#[tauri::command]
pub fn save_profile_file(index: String, file_data: Option<String>) -> CmdResult {
    if file_data.is_none() {
        return Ok(());
    }

    let profiles = Config::profiles();
    let profiles = profiles.latest();
    let item = wrap_err!(profiles.get_item(&index))?;
    wrap_err!(item.save_file(file_data.unwrap()))
}

#[tauri::command]
pub fn get_clash_info() -> CmdResult<ClashInfo> {
    Ok(Config::clash().latest().get_client_info())
}

#[tauri::command]
pub fn get_runtime_config() -> CmdResult<Option<Mapping>> {
    Ok(Config::runtime().latest().config.clone())
}

#[tauri::command]
pub fn get_runtime_yaml() -> CmdResult<String> {
    let runtime = Config::runtime();
    let runtime = runtime.latest();
    let config = runtime.config.as_ref();
    wrap_err!(config
        .ok_or(anyhow::anyhow!("failed to parse config to yaml file"))
        .and_then(
            |config| serde_yaml::to_string(config).context("failed to convert config to yaml")
        ))
}

#[tauri::command]
pub fn get_runtime_exists() -> CmdResult<Vec<String>> {
    Ok(Config::runtime().latest().exists_keys.clone())
}

#[tauri::command]
pub fn get_runtime_logs() -> CmdResult<HashMap<String, Vec<(String, String)>>> {
    Ok(Config::runtime().latest().chain_logs.clone())
}

#[tauri::command]
pub async fn patch_clash_config(payload: Mapping) -> CmdResult {
    wrap_err!(feat::patch_clash(payload).await)
}

#[tauri::command]
pub fn get_verge_config() -> CmdResult<IVerge> {
    Ok(Config::verge().data().clone())
}

#[tauri::command]
pub async fn patch_verge_config(payload: IVerge) -> CmdResult {
    wrap_err!(feat::patch_verge(payload).await)
}

#[tauri::command]
pub async fn change_clash_core(clash_core: Option<String>) -> CmdResult {
    wrap_err!(CoreManager::global().change_core(clash_core).await)
}

/// restart the sidecar
#[tauri::command]
pub async fn restart_sidecar() -> CmdResult {
    wrap_err!(CoreManager::global().run_core().await)
}

#[tauri::command]
pub fn grant_permission(_core: String) -> CmdResult {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    return wrap_err!(manager::grant_permission(_core));

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    return Err("Unsupported target".into());
}

/// get the system proxy
#[tauri::command]
pub fn get_sys_proxy() -> CmdResult<Mapping> {
    let current = wrap_err!(Sysproxy::get_system_proxy())?;

    let mut map = Mapping::new();
    map.insert("enable".into(), current.enable.into());
    map.insert(
        "server".into(),
        format!("{}:{}", current.host, current.port).into(),
    );
    map.insert("bypass".into(), current.bypass.into());

    Ok(map)
}

#[tauri::command]
pub fn get_clash_logs() -> CmdResult<VecDeque<String>> {
    Ok(logger::Logger::global().get_log())
}

#[tauri::command]
pub fn open_app_dir() -> CmdResult<()> {
    let app_dir = wrap_err!(dirs::app_home_dir())?;
    wrap_err!(open::that(app_dir))
}

#[tauri::command]
pub fn open_core_dir() -> CmdResult<()> {
    let core_dir = wrap_err!(tauri::utils::platform::current_exe())?;
    let core_dir = core_dir.parent().ok_or(format!("failed to get core dir"))?;
    wrap_err!(open::that(core_dir))
}

#[tauri::command]
pub fn open_logs_dir() -> CmdResult<()> {
    let log_dir = wrap_err!(dirs::app_logs_dir())?;
    wrap_err!(open::that(log_dir))
}

#[tauri::command]
pub fn open_web_url(url: String) -> CmdResult<()> {
    wrap_err!(open::that(url))
}


#[cfg(windows)]
pub mod uwp {
    use super::*;
    use crate::core::win_uwp;

    #[tauri::command]
    pub async fn invoke_uwp_tool() -> CmdResult {
        wrap_err!(win_uwp::invoke_uwptools().await)
    }
}

#[tauri::command]
pub async fn clash_api_get_proxy_delay(
    name: String,
    url: Option<String>,
) -> CmdResult<clash_api::DelayRes> {
    match clash_api::get_proxy_delay(name, url).await {
        Ok(res) => Ok(res),
        Err(err) => Err(format!("{}", err.to_string())),
    }
}

#[cfg(windows)]
pub mod service {
    use super::*;
    use crate::core::win_service;

    #[tauri::command]
    pub async fn check_service() -> CmdResult<win_service::JsonResponse> {
        wrap_err!(win_service::check_service().await)
    }

    #[tauri::command]
    pub fn install_service() -> CmdResult {
        wrap_err!(win_service::install_service())
    }

    #[tauri::command]
    pub fn uninstall_service() -> CmdResult {
        wrap_err!(win_service::uninstall_service())
    }
}

#[cfg(not(windows))]
pub mod service {
    use super::*;

    #[tauri::command]
    pub async fn check_service() -> CmdResult {
        Ok(())
    }
    #[tauri::command]
    pub async fn install_service() -> CmdResult {
        Ok(())
    }
    #[tauri::command]
    pub async fn uninstall_service() -> CmdResult {
        Ok(())
    }
}

#[cfg(not(windows))]
pub mod uwp {
    use super::*;

    #[tauri::command]
    pub async fn invoke_uwp_tool() -> CmdResult {
        Ok(())
    }
}