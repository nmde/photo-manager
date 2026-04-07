use std::str::FromStr;

use anyhow::Context;
use chrono::NaiveDate;
use log::debug;
use serde::Serialize;
use tauri::{AppHandle, Manager, Runtime, State};

use crate::{
    photos::{PhotoState, Sort},
    tags::ValidationResult,
    ApiError,
};

#[derive(Serialize)]
pub struct PhotoDto {
    pub name: String,
    pub asset_path: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub is_duplicate: bool,
    pub rating: Option<i32>,
    pub location: Option<String>,
    pub thumbnail: Option<String>,
    pub photo_group: Option<String>,
    pub date: Option<NaiveDate>,
    pub people: Vec<String>,
    pub hide_thumbnail: bool,
    pub photographer: Option<String>,
    pub is_video: bool,
    pub is_raw: bool,
    pub valid_tags: bool,
    pub validation_msg: Option<String>,
}

#[tauri::command]
pub async fn initialize<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, PhotoState>,
    path: String,
) -> Result<Vec<String>, ApiError> {
    debug!("Initializing with path {path}");

    Ok(state
        .app
        .lock()
        .await
        .initialize(&path, &app.path().app_data_dir()?)
        .await
        .with_context(|| format!("Failed to initialize Photo Manager at {path}"))?)
}

#[tauri::command]
pub async fn photo_grid(
    state: State<'_, PhotoState>,
    query: Vec<String>,
    sort: String,
) -> Result<Vec<PhotoDto>, ApiError> {
    let mut app = state.app.lock().await;

    let mut results = vec![];
    for photo in app.search_photos(&query, Sort::from_str(&sort)?).await? {
        let validation = app.validate_tags(&photo.tags()).await?;
        results.push(PhotoDto {
            name: photo.name.clone(),
            asset_path: photo.asset_path.clone(),
            title: photo.title.clone(),
            description: photo.description.clone(),
            tags: photo.tags(),
            is_duplicate: photo.is_duplicate.unwrap_or(0) == 1,
            rating: photo.rating,
            location: photo.location.clone(),
            thumbnail: photo.thumbnail.clone(),
            photo_group: photo.photo_group.clone(),
            date: photo.date(),
            people: photo.people(),
            hide_thumbnail: photo.hide_thumbnail.unwrap_or(0) == 1,
            photographer: photo.photographer.clone(),
            is_video: photo.is_video(),
            is_raw: photo.is_raw(),
            valid_tags: validation.is_valid,
            validation_msg: validation.message,
        })
    }

    Ok(results)
}

#[tauri::command]
pub async fn remove_deleted(
    state: State<'_, PhotoState>,
    deleted: Vec<String>,
) -> Result<(), ApiError> {
    debug!(
        "Removing {} moved or deleted photos from the database",
        deleted.len()
    );
    state.app.lock().await.remove_deleted(&deleted).await?;

    Ok(())
}

#[tauri::command]
pub async fn set_photo_title(
    state: State<'_, PhotoState>,
    photo: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Setting photo {photo} title to {value}");
    state
        .app
        .lock()
        .await
        .set_photo_title(&photo, &value)
        .await
        .with_context(|| format!("Could not set photo {photo} title"))?;

    Ok(())
}

#[tauri::command]
pub async fn set_photo_desc(
    state: State<'_, PhotoState>,
    photo: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Setting photo {photo} description to {value}");
    state
        .app
        .lock()
        .await
        .set_photo_desc(&photo, &value)
        .await
        .with_context(|| format!("Could not set photo {photo} description"))?;

    Ok(())
}

#[tauri::command]
pub async fn set_photographer(
    state: State<'_, PhotoState>,
    photo: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Setting photo {photo} photographer to {value}");
    state
        .app
        .lock()
        .await
        .set_photographer(&photo, &value)
        .await
        .with_context(|| format!("Could not set photo {photo} photographer"))?;

    Ok(())
}

#[tauri::command]
pub async fn set_photo_people(
    state: State<'_, PhotoState>,
    photo: String,
    value: Vec<String>,
) -> Result<(), ApiError> {
    debug!("Setting photo {photo} people to {}", value.join(","));
    state
        .app
        .lock()
        .await
        .set_photo_people(&photo, &value)
        .await
        .with_context(|| format!("Could not set photo {photo} people to {}", value.join(",")))?;

    Ok(())
}

#[tauri::command]
pub async fn set_photo_location(
    state: State<'_, PhotoState>,
    photo: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Setting photo {photo} location to {value}");
    state
        .app
        .lock()
        .await
        .set_photo_location(&photo, &value)
        .await
        .with_context(|| format!("Could not set photo {photo} location to {value}"))?;

    Ok(())
}

#[tauri::command]
pub async fn set_photo_tags(
    state: State<'_, PhotoState>,
    photo: String,
    value: Vec<String>,
) -> Result<ValidationResult, ApiError> {
    debug!("Setting photo {photo} tags to {}", value.join(","));
    Ok(state
        .app
        .lock()
        .await
        .set_photo_tags(&photo, &value)
        .await
        .with_context(|| format!("Could not set photo {photo} tags to {}", value.join(",")))?)
}

#[tauri::command]
pub async fn set_photo_date(
    state: State<'_, PhotoState>,
    photo: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Setting photo {photo} date to {value}");
    state
        .app
        .lock()
        .await
        .set_photo_date(&photo, &value)
        .await
        .with_context(|| format!("Could not set photo {photo} date to {value}"))?;

    Ok(())
}

#[tauri::command]
pub async fn set_photo_group(
    state: State<'_, PhotoState>,
    photo: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Setting photo {photo} group to {value}");
    state
        .app
        .lock()
        .await
        .set_photo_group(&photo, &value)
        .await
        .with_context(|| format!("Could not set photo {photo} group to {value}"))?;

    Ok(())
}

#[tauri::command]
pub async fn set_photo_rating(
    state: State<'_, PhotoState>,
    photo: String,
    rating: i32,
) -> Result<(), ApiError> {
    debug!("Setting photo {photo} rating to {rating}");
    state
        .app
        .lock()
        .await
        .set_photo_rating(&photo, rating)
        .await
        .with_context(|| format!("Could not set photo {photo} rating to {rating}"))?;

    Ok(())
}

#[tauri::command]
pub async fn set_photo_is_duplicate(
    state: State<'_, PhotoState>,
    photo: String,
    value: bool,
) -> Result<(), ApiError> {
    debug!("Setting photo {photo} duplicate to {value}");
    state
        .app
        .lock()
        .await
        .set_photo_is_duplicate(&photo, value)
        .await
        .with_context(|| format!("Could not set photo {photo} duplicate to {value}"))?;

    Ok(())
}

#[tauri::command]
pub async fn set_photo_hide_thumbnail(
    state: State<'_, PhotoState>,
    photo: String,
    value: bool,
) -> Result<(), ApiError> {
    debug!("Setting photo {photo} hide thumbnail to {value}");
    state
        .app
        .lock()
        .await
        .set_photo_hide_thumbnail(&photo, value)
        .await
        .with_context(|| format!("Could not set photo {photo} hide thumbnail to {value}"))?;

    Ok(())
}

#[tauri::command]
pub async fn refresh<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, PhotoState>,
    path: String,
) -> Result<Vec<String>, ApiError> {
    debug!("Refreshing photos from {path}");

    Ok(state
        .app
        .lock()
        .await
        .refresh(&path, &app.path().app_data_dir()?)
        .await
        .with_context(|| format!("Failed to refresh photos from {path}"))?)
}
