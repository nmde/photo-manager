use anyhow::Context;
use log::debug;

use crate::{
    app::ApiError,
    models::Tag,
    tags::{
        get_tags as _get_tags, validate_photo as _validate_photo, TagRelationship,
        ValidationResult, TAGS,
    },
};

#[tauri::command]
pub async fn set_tag_color(tag: String, value: String) -> Result<(), ApiError> {
    debug!("Setting tag {tag} color to {value}");

    let tags = TAGS.lock().await;
    let target = tags.get(&tag);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Tag {tag} not found")));
    }

    target
        .unwrap()
        .set_tag_color(&tag, &value)
        .await
        .with_context(|| format!("Could not set tag {tag} color to {value}"))?;

    Ok(())
}

#[tauri::command]
pub async fn set_tag_prereqs(tag: String, value: Vec<String>) -> Result<(), ApiError> {
    debug!("Setting tag {tag} prereqs to {}", value.join(","));

    let tags = TAGS.lock().await;
    let target = tags.get(&tag);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Tag {tag} not found")));
    }

    target
        .unwrap()
        .modify_tag_relationships(TagRelationship::Prereqs, &tag, &value)
        .await
        .with_context(|| format!("Could not set tag {tag} prereqs to {}", value.join(",")))?;

    Ok(())
}

#[tauri::command]
pub async fn set_tag_coreqs(tag: String, value: Vec<String>) -> Result<(), ApiError> {
    debug!("Setting tag {tag} coreqs to {}", value.join(","));

    let tags = TAGS.lock().await;
    let target = tags.get(&tag);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Tag {tag} not found")));
    }

    target
        .unwrap()
        .modify_tag_relationships(TagRelationship::Coreqs, &tag, &value)
        .await
        .with_context(|| format!("Could not set tag {tag} coreqs to {}", value.join(",")))?;

    Ok(())
}

#[tauri::command]
pub async fn set_tag_incompatible(tag: String, value: Vec<String>) -> Result<(), ApiError> {
    debug!("Setting tag {tag} incompatible to {}", value.join(","));

    let tags = TAGS.lock().await;
    let target = tags.get(&tag);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Tag {tag} not found")));
    }

    target
        .unwrap()
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
pub async fn get_tags() -> Result<Vec<Tag>, ApiError> {
    Ok(_get_tags()
        .await
        .with_context(|| "Failed to get tags".to_string())?)
}

#[tauri::command]
pub async fn validate_photo(photo: String) -> Result<ValidationResult, ApiError> {
    Ok(_validate_photo(&photo)
        .await
        .with_context(|| format!("Failed to validate photo {photo}"))?)
}
