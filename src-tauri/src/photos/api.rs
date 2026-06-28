use anyhow::Context;
use log::debug;

use crate::{
    app::ApiError,
    photos::{get_group as _get_group, PhotoDto, PHOTOS},
    tags::ValidationResult,
};

#[tauri::command]
pub async fn set_photo_title(photo: String, value: Option<String>) -> Result<(), ApiError> {
    debug!(
        "Setting photo {photo} title to {}",
        value.as_ref().unwrap_or(&"NULL".to_string())
    );

    let mut photos = PHOTOS.lock().await;
    let target = photos.get_mut(&photo);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Photo {photo} not found")));
    }

    target
        .unwrap()
        .set_photo_title(&photo, &value)
        .await
        .with_context(|| format!("Could not set photo {photo} title"))?;

    Ok(())
}

#[tauri::command]
pub async fn set_photo_desc(photo: String, value: Option<String>) -> Result<(), ApiError> {
    debug!(
        "Setting photo {photo} description to {}",
        value.as_ref().unwrap_or(&"NULL".to_string())
    );

    let mut photos = PHOTOS.lock().await;
    let target = photos.get_mut(&photo);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Photo {photo} not found")));
    }

    target
        .unwrap()
        .set_photo_desc(&photo, &value)
        .await
        .with_context(|| format!("Could not set photo {photo} description"))?;

    Ok(())
}

#[tauri::command]
pub async fn set_photographer(photo: String, value: Option<String>) -> Result<(), ApiError> {
    debug!(
        "Setting photo {photo} photographer to {}",
        value.as_ref().unwrap_or(&"NULL".to_string())
    );

    let mut photos = PHOTOS.lock().await;
    let target = photos.get_mut(&photo);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Photo {photo} not found")));
    }

    target
        .unwrap()
        .set_photographer(&photo, &value)
        .await
        .with_context(|| format!("Could not set photo {photo} photographer"))?;

    Ok(())
}

#[tauri::command]
pub async fn set_photo_people(photo: String, value: Vec<String>) -> Result<(), ApiError> {
    debug!("Setting photo {photo} people to {}", value.join(","));

    let mut photos = PHOTOS.lock().await;
    let target = photos.get_mut(&photo);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Photo {photo} not found")));
    }

    target
        .unwrap()
        .set_photo_people(&photo, &value)
        .await
        .with_context(|| format!("Could not set photo {photo} people to {}", value.join(",")))?;

    Ok(())
}

#[tauri::command]
pub async fn set_photo_location(photo: String, value: Option<String>) -> Result<(), ApiError> {
    debug!(
        "Setting photo {photo} location to {}",
        value.as_ref().unwrap_or(&"NULL".to_string())
    );

    let mut photos = PHOTOS.lock().await;
    let target = photos.get_mut(&photo);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Photo {photo} not found")));
    }

    target
        .unwrap()
        .set_photo_location(&photo, &value)
        .await
        .with_context(|| {
            format!(
                "Could not set photo {photo} location to {}",
                value.unwrap_or("NULL".to_string())
            )
        })?;

    Ok(())
}

#[tauri::command]
pub async fn set_photo_date(photo: String, value: Option<String>) -> Result<(), ApiError> {
    debug!(
        "Setting photo {photo} date to {}",
        value.as_ref().unwrap_or(&"NULL".to_string())
    );

    let mut photos = PHOTOS.lock().await;
    let target = photos.get_mut(&photo);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Photo {photo} not found")));
    }

    target
        .unwrap()
        .set_photo_date(&photo, &value)
        .await
        .with_context(|| {
            format!(
                "Could not set photo {photo} date to {}",
                value.unwrap_or("NULL".to_string())
            )
        })?;

    Ok(())
}

#[tauri::command]
pub async fn set_photo_group(photo: String, value: Option<String>) -> Result<(), ApiError> {
    debug!(
        "Setting photo {photo} group to {}",
        value.as_ref().unwrap_or(&"NULL".to_string())
    );

    let mut photos = PHOTOS.lock().await;
    let target = photos.get_mut(&photo);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Photo {photo} not found")));
    }

    target
        .unwrap()
        .set_photo_group(&photo, &value)
        .await
        .with_context(|| {
            format!(
                "Could not set photo {photo} group to {}",
                value.unwrap_or("NULL".to_string())
            )
        })?;

    Ok(())
}

#[tauri::command]
pub async fn set_photo_rating(photo: String, rating: Option<i32>) -> Result<(), ApiError> {
    debug!(
        "Setting photo {photo} rating to {}",
        rating.as_ref().unwrap_or(&-1)
    );

    let mut photos = PHOTOS.lock().await;
    let target = photos.get_mut(&photo);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Photo {photo} not found")));
    }

    target
        .unwrap()
        .set_photo_rating(&photo, rating)
        .await
        .with_context(|| {
            format!(
                "Could not set photo {photo} rating to {}",
                rating.unwrap_or(-1)
            )
        })?;

    Ok(())
}

#[tauri::command]
pub async fn set_photo_is_duplicate(photo: String, value: bool) -> Result<(), ApiError> {
    debug!("Setting photo {photo} duplicate to {value}");

    let mut photos = PHOTOS.lock().await;
    let target = photos.get_mut(&photo);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Photo {photo} not found")));
    }

    target
        .unwrap()
        .set_photo_is_duplicate(&photo, value)
        .await
        .with_context(|| format!("Could not set photo {photo} duplicate to {value}"))?;

    Ok(())
}

#[tauri::command]
pub async fn set_photo_hide_thumbnail(photo: String, value: bool) -> Result<(), ApiError> {
    debug!("Setting photo {photo} hide thumbnail to {value}");

    let mut photos = PHOTOS.lock().await;
    let target = photos.get_mut(&photo);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Photo {photo} not found")));
    }

    target
        .unwrap()
        .set_photo_hide_thumbnail(&photo, value)
        .await
        .with_context(|| format!("Could not set photo {photo} hide thumbnail to {value}"))?;

    Ok(())
}

#[tauri::command]
pub async fn set_photo_tags(
    photo: String,
    value: Vec<String>,
) -> Result<ValidationResult, ApiError> {
    debug!("Setting photo {photo} tags to {}", value.join(","));

    let mut photos = PHOTOS.lock().await;
    let target = photos.get_mut(&photo);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Photo {photo} not found")));
    }

    Ok(target
        .unwrap()
        .set_photo_tags(&photo, &value)
        .await
        .with_context(|| format!("Could not set photo {photo} tags to {}", value.join(",")))?)
}

/// Gets the thumbnail of a grouped raw for the "show raw" button
#[tauri::command]
pub async fn get_grouped_raw(photo: String) -> Option<String> {
    debug!("Getting grouped raw thumbnail from base photo {photo}");

    let photos = PHOTOS.lock().await;
    photos.get(&photo).and_then(|p| p.thumbnail.clone())
}

/// Gets photos in the given group
#[tauri::command]
pub async fn get_group(group: String) -> Result<Vec<PhotoDto>, ApiError> {
    Ok(_get_group(&group)
        .await?
        .iter()
        .map(PhotoDto::from)
        .collect())
}
