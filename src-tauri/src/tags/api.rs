use anyhow::Context;
use log::debug;
use serde::Serialize;
use tauri::State;

use crate::{
    photos::PhotoState,
    tags::{TagRelationship, ValidationResult},
    ApiError,
};

#[derive(Serialize)]
pub struct TagDto {
    pub name: String,
    pub color: String,
    pub prereqs: Vec<String>,
    pub coreqs: Vec<String>,
    pub incompatible: Vec<String>,
    pub count: usize,
}

#[tauri::command]
pub async fn set_tag_color(
    state: State<'_, PhotoState>,
    tag: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Setting tag {tag} color to {value}");
    state
        .app
        .lock()
        .await
        .set_tag_color(&tag, &value)
        .await
        .with_context(|| format!("Could not set tag {tag} color to {value}"))?;

    Ok(())
}

#[tauri::command]
pub async fn set_tag_prereqs(
    state: State<'_, PhotoState>,
    tag: String,
    value: Vec<String>,
) -> Result<(), ApiError> {
    debug!("Setting tag {tag} prereqs to {}", value.join(","));
    state
        .app
        .lock()
        .await
        .modify_tag_relationships(TagRelationship::Prereqs, &tag, &value)
        .await
        .with_context(|| format!("Could not set tag {tag} prereqs to {}", value.join(",")))?;

    Ok(())
}

#[tauri::command]
pub async fn set_tag_coreqs(
    state: State<'_, PhotoState>,
    tag: String,
    value: Vec<String>,
) -> Result<(), ApiError> {
    debug!("Setting tag {tag} coreqs to {}", value.join(","));
    state
        .app
        .lock()
        .await
        .modify_tag_relationships(TagRelationship::Coreqs, &tag, &value)
        .await
        .with_context(|| format!("Could not set tag {tag} coreqs to {}", value.join(",")))?;

    Ok(())
}

#[tauri::command]
pub async fn set_tag_incompatible(
    state: State<'_, PhotoState>,
    tag: String,
    value: Vec<String>,
) -> Result<(), ApiError> {
    debug!("Setting tag {tag} incompatible to {}", value.join(","));
    state
        .app
        .lock()
        .await
        .modify_tag_relationships(TagRelationship::Incompatible, &tag, &value)
        .await
        .with_context(|| {
            format!(
                "Could not set tag {tag} incompatible to {}",
                value.join(",")
            )
        })?;

    Ok(())
}

#[tauri::command]
pub async fn get_tags(state: State<'_, PhotoState>) -> Result<Vec<TagDto>, ApiError> {
    let mut app = state.app.lock().await;

    let mut results = vec![];
    for tag in app
        .get_tags()
        .await
        .with_context(|| "Failed to get tags".to_string())?
    {
        results.push(TagDto {
            name: tag.name.clone(),
            color: tag.color.clone(),
            prereqs: tag.prereqs(),
            coreqs: tag.coreqs(),
            incompatible: tag.incompatible(),
            count: app.count_photos_by_tag(&tag.name).await.ok().unwrap_or(0),
        });
    }

    Ok(results)
}

#[tauri::command]
pub async fn validate_photo(
    state: State<'_, PhotoState>,
    photo: String,
) -> Result<ValidationResult, ApiError> {
    Ok(state
        .app
        .lock()
        .await
        .validate_photo(&photo)
        .await
        .with_context(|| format!("Failed to validate photo {photo}"))?)
}
