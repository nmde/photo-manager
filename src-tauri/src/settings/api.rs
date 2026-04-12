use std::str::FromStr;

use crate::{
    app::ApiError,
    settings::{
        add_color as _add_color, get_colors as _get_colors, get_setting,
        promote_color as _promote_color, set_setting, Settings, ThemeSetting,
    },
};

#[tauri::command]
pub async fn get_theme() -> Result<ThemeSetting, ApiError> {
    Ok(ThemeSetting::from_str(
        &get_setting(Settings::Theme).await?,
    )?)
}

#[tauri::command]
pub async fn set_theme(value: String) -> Result<(), ApiError> {
    // The from_str().to_string() conversion here checks the validity of the supplied value
    set_setting(Settings::Theme, ThemeSetting::from_str(&value)?.to_string()).await?;
    Ok(())
}

#[tauri::command]
pub async fn get_colors() -> Result<Vec<String>, ApiError> {
    Ok(_get_colors().await?)
}

#[tauri::command]
pub async fn promote_color(color: String) -> Result<Vec<String>, ApiError> {
    Ok(_promote_color(&color).await?)
}

#[tauri::command]
pub async fn add_color(color: String) -> Result<Vec<String>, ApiError> {
    Ok(_add_color(&color).await?)
}
