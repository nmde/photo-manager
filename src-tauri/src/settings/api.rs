use log::debug;

use crate::{
    app::ApiError,
    models::Setting,
    settings::{get_setting as _get_setting, set_setting as _set_setting},
};

#[tauri::command]
pub async fn set_setting(setting: String, value: i32) -> Result<(), ApiError> {
    debug!("Setting setting {setting} to {value}");
    _set_setting(&setting, value).await?;

    Ok(())
}

#[tauri::command]
pub async fn get_setting(setting: String) -> Result<Option<Setting>, ApiError> {
    Ok(_get_setting(&setting).await?)
}
