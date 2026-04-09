use anyhow::Context;
use log::debug;

use crate::{app::ApiError, photos::PHOTOS, tags::ValidationResult};

#[tauri::command]
pub async fn set_photo_title(photo: String, value: String) -> Result<(), ApiError> {
    debug!("Setting photo {photo} title to {value}");

    let photos = PHOTOS.lock().await;
    let target = photos.get(&photo);
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
pub async fn set_photo_desc(photo: String, value: String) -> Result<(), ApiError> {
    debug!("Setting photo {photo} description to {value}");

    let photos = PHOTOS.lock().await;
    let target = photos.get(&photo);
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
pub async fn set_photographer(photo: String, value: String) -> Result<(), ApiError> {
    debug!("Setting photo {photo} photographer to {value}");

    let photos = PHOTOS.lock().await;
    let target = photos.get(&photo);
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

    let photos = PHOTOS.lock().await;
    let target = photos.get(&photo);
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
pub async fn set_photo_location(photo: String, value: String) -> Result<(), ApiError> {
    debug!("Setting photo {photo} location to {value}");

    let photos = PHOTOS.lock().await;
    let target = photos.get(&photo);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Photo {photo} not found")));
    }

    target
        .unwrap()
        .set_photo_location(&photo, &value)
        .await
        .with_context(|| format!("Could not set photo {photo} location to {value}"))?;

    Ok(())
}

#[tauri::command]
pub async fn set_photo_date(photo: String, value: String) -> Result<(), ApiError> {
    debug!("Setting photo {photo} date to {value}");

    let photos = PHOTOS.lock().await;
    let target = photos.get(&photo);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Photo {photo} not found")));
    }

    target
        .unwrap()
        .set_photo_date(&photo, &value)
        .await
        .with_context(|| format!("Could not set photo {photo} date to {value}"))?;

    Ok(())
}

#[tauri::command]
pub async fn set_photo_group(photo: String, value: String) -> Result<(), ApiError> {
    debug!("Setting photo {photo} group to {value}");

    let photos = PHOTOS.lock().await;
    let target = photos.get(&photo);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Photo {photo} not found")));
    }

    target
        .unwrap()
        .set_photo_group(&photo, &value)
        .await
        .with_context(|| format!("Could not set photo {photo} group to {value}"))?;

    Ok(())
}

#[tauri::command]
pub async fn set_photo_rating(photo: String, rating: i32) -> Result<(), ApiError> {
    debug!("Setting photo {photo} rating to {rating}");

    let photos = PHOTOS.lock().await;
    let target = photos.get(&photo);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Photo {photo} not found")));
    }

    target
        .unwrap()
        .set_photo_rating(&photo, rating)
        .await
        .with_context(|| format!("Could not set photo {photo} rating to {rating}"))?;

    Ok(())
}

#[tauri::command]
pub async fn set_photo_is_duplicate(photo: String, value: bool) -> Result<(), ApiError> {
    debug!("Setting photo {photo} duplicate to {value}");

    let photos = PHOTOS.lock().await;
    let target = photos.get(&photo);
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

    let photos = PHOTOS.lock().await;
    let target = photos.get(&photo);
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

    let photos = PHOTOS.lock().await;
    let target = photos.get(&photo);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Photo {photo} not found")));
    }

    Ok(target
        .unwrap()
        .set_photo_tags(&photo, &value)
        .await
        .with_context(|| format!("Could not set photo {photo} tags to {}", value.join(",")))?)
}
