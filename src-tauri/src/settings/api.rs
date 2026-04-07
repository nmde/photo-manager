use log::debug;
use tauri::State;

use crate::{ApiError, models::Setting, photos::PhotoState};

#[tauri::command]
pub async fn set_setting(
    state: State<'_, PhotoState>,
    setting: String,
    value: i32,
) -> Result<(), ApiError> {
    debug!("Setting setting {setting} to {value}");
    state.app.lock().await.set_setting(&setting, value).await?;

    Ok(())
}

#[tauri::command]
pub async fn get_setting(
    state: State<'_, PhotoState>,
    setting: String,
) -> Result<Option<Setting>, ApiError> {
    Ok(state.app.lock().await.get_setting(&setting).await?)
}
