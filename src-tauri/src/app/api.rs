use std::str::FromStr;

use anyhow::Context;
use log::debug;
use tauri::{AppHandle, Manager, Runtime};

use crate::{
    app::{
        initialize as _initialize, refresh as _refresh, remove_deleted as _remove_deleted,
        search_photos, ApiError, LoadedPhotos, Sort,
    },
    photos::PhotoDto,
};

#[tauri::command]
pub async fn initialize<R: Runtime>(
    app: AppHandle<R>,
    path: String,
) -> Result<LoadedPhotos, ApiError> {
    debug!("Initializing with path {path}");

    Ok(_initialize(&path, &app.path().app_data_dir()?)
        .await
        .with_context(|| format!("Failed to initialize Photo Manager at {path}"))?)
}

#[tauri::command]
pub async fn photo_grid(query: Vec<String>, sort: String) -> Result<Vec<PhotoDto>, ApiError> {
    Ok(search_photos(&query, Sort::from_str(&sort)?)
        .await?
        .iter()
        .map(|x| PhotoDto::from(x))
        .collect::<Vec<PhotoDto>>())
}

#[tauri::command]
pub async fn remove_deleted(deleted: Vec<String>) -> Result<(), ApiError> {
    debug!(
        "Removing {} moved or deleted photos from the database",
        deleted.len()
    );
    _remove_deleted(&deleted).await?;

    Ok(())
}

#[tauri::command]
pub async fn refresh<R: Runtime>(app: AppHandle<R>, path: String) -> Result<Vec<String>, ApiError> {
    debug!("Refreshing photos from {path}");

    Ok(_refresh(&path, &app.path().app_data_dir()?)
        .await
        .with_context(|| format!("Failed to refresh photos from {path}"))?)
}
