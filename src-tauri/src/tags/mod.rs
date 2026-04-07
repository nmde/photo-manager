use std::collections::HashMap;

use diesel::{dsl::update, ExpressionMethods, QueryDsl, SqliteConnection, TextExpressionMethods};
use diesel_async::{sync_connection_wrapper::SyncConnectionWrapper, RunQueryDsl};
use log::debug;
use serde::Serialize;
use tauri::State;

use crate::{
    models::{Photo, Tag},
    photos::{get_photo_targets, PhotoState},
    schema::{photos, tags},
    ApiError,
};

impl Tag {
    pub fn new(name: &String) -> Self {
        Self {
            name: name.clone(),
            color: String::new(),
            prereqs: None,
            coreqs: None,
            incompatible: None,
        }
    }
}

#[derive(Serialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub message: String,
}

pub fn validate_tags(state_tags: &HashMap<String, Tag>, tags: &Vec<String>) -> ValidationResult {
    if tags.len() == 0 {
        ValidationResult {
            is_valid: true,
            message: String::new(),
        }
    } else {
        let mut valid = true;
        let mut message = String::new();
        let mut missing_prereqs = String::new();
        let mut missing_coreqs = String::new();
        let mut incompatibles = String::new();
        for tag in tags {
            if state_tags.contains_key(tag) {
                let tag_data = state_tags.get(tag).unwrap();
                for prereq in &tag_data.prereqs {
                    if !tags.contains(prereq) {
                        valid = false;
                        missing_prereqs.push_str(&prereq);
                        missing_prereqs.push_str(", ");
                    }
                }
                for coreq in &tag_data.coreqs {
                    if !tags.contains(coreq) {
                        valid = false;
                        missing_coreqs.push_str(&coreq);
                        missing_coreqs.push_str(", ");
                    }
                }
                for incompatible in &tag_data.incompatible {
                    if tags.contains(incompatible) {
                        valid = false;
                        incompatibles.push_str(&incompatible);
                        incompatibles.push_str(", ");
                    }
                }
            }
        }

        if missing_prereqs.len() > 0 {
            message.push_str("Missing prerequisite tag(s): ");
            message.push_str(&missing_prereqs);
        }
        if missing_coreqs.len() > 0 {
            message.push_str("Missing corequisite tag(s): ");
            message.push_str(&missing_coreqs);
        }
        if incompatibles.len() > 0 {
            message.push_str("Incompatible tag(s) present: ");
            message.push_str(&incompatibles);
        }

        ValidationResult {
            is_valid: valid,
            message,
        }
    }
}

#[tauri::command]
pub async fn set_tag_color(
    state: State<'_, PhotoState>,
    tag: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Setting tag {tag} color to {value}");
    let mut conn = state.db.lock().await;
    let conn = conn.as_mut().unwrap();
    let mut state_tags = state.tags.lock().await;

    if state_tags.contains_key(&tag) {
        let state_tag = state_tags.get_mut(&tag).unwrap();
        update(tags::table.filter(tags::name.eq(tag)))
            .set(tags::color.eq(value.clone()))
            .execute(conn)
            .await?;

        state_tag.color = value.clone();
    } else {
        return Err(ApiError::NotFoundError(format!("Tag '{}' not found", tag)));
    }

    Ok(())
}

enum TagRelationship {
    Prereqs,
    Coreqs,
    Incompatible,
}

async fn modify_tag_relationships(
    connection: &mut SyncConnectionWrapper<SqliteConnection>,
    category: TagRelationship,
    tag: &String,
    value: &Vec<String>,
    state_photos: &mut HashMap<String, Photo>,
    state_tags: &HashMap<String, Tag>,
) -> Result<(), ApiError> {
    let joined = value.join(",");
    match category {
        TagRelationship::Prereqs => {
            update(tags::table.filter(tags::name.eq(tag)))
                .set(tags::prereqs.eq(joined))
                .execute(connection)
                .await?
        }
        TagRelationship::Coreqs => {
            update(tags::table.filter(tags::name.eq(tag)))
                .set(tags::coreqs.eq(joined))
                .execute(connection)
                .await?
        }
        TagRelationship::Incompatible => {
            update(tags::table.filter(tags::name.eq(tag)))
                .set(tags::incompatible.eq(joined))
                .execute(connection)
                .await?
        }
    };

    // Gets photos from the database with the tag as a substring in their tags field
    let maybe_has_tag = photos::table
        .filter(photos::tags.like(tag))
        .load::<Photo>(connection)
        .await?;

    for photo in maybe_has_tag {
        if photo.tags.contains(&tag) {
            let target = state_photos.get_mut(&photo.photo.name).unwrap();
            let validation = validate_tags(&state_tags, &photo.tags);
            target.valid_tags = validation.is_valid;
            target.validation_msg = validation.message;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn set_tag_prereqs(
    state: State<'_, PhotoState>,
    tag: String,
    value: Vec<String>,
) -> Result<(), ApiError> {
    debug!("Setting tag {tag} prereqs to {}", value.join(","));
    let mut conn = state.db.lock().await;
    let conn = conn.as_mut().unwrap();
    let mut state_tags = state.tags.lock().await;
    let mut state_photos = state.photos.lock().await;

    if state_tags.contains_key(&tag) {
        let state_tag = state_tags.get_mut(&tag).unwrap();
        state_tag.prereqs = value.clone();
        modify_tag_relationships(
            conn,
            TagRelationship::Prereqs,
            &tag,
            &value,
            &mut state_photos,
            &state_tags,
        )
        .await?;
    } else {
        return Err(ApiError::NotFoundError(format!("Tag '{}' not found", tag)));
    }

    Ok(())
}

#[tauri::command]
pub async fn set_tag_coreqs(
    state: State<'_, PhotoState>,
    tag: String,
    value: Vec<String>,
) -> Result<(), ApiError> {
    debug!("Setting tag {tag} coreqs to {}", value.join(","));
    let mut conn = state.db.lock().await;
    let conn = conn.as_mut().unwrap();
    let mut state_tags = state.tags.lock().await;
    let mut state_photos = state.photos.lock().await;

    if state_tags.contains_key(&tag) {
        let state_tag = state_tags.get_mut(&tag).unwrap();
        state_tag.coreqs = value.clone();
        modify_tag_relationships(
            conn,
            TagRelationship::Coreqs,
            &tag,
            &value,
            &mut state_photos,
            &state_tags,
        )
        .await?;
    } else {
        return Err(ApiError::NotFoundError(format!("Tag '{}' not found", tag)));
    }

    Ok(())
}

#[tauri::command]
pub async fn set_tag_incompatible(
    state: State<'_, PhotoState>,
    tag: String,
    value: Vec<String>,
) -> Result<(), ApiError> {
    debug!("Setting tag {tag} incompatible to {}", value.join(","));
    let mut conn = state.db.lock().await;
    let conn = conn.as_mut().unwrap();
    let mut state_tags = state.tags.lock().await;
    let mut state_photos = state.photos.lock().await;

    if state_tags.contains_key(&tag) {
        let state_tag = state_tags.get_mut(&tag).unwrap();
        state_tag.incompatible = value.clone();
        modify_tag_relationships(
            conn,
            TagRelationship::Incompatible,
            &tag,
            &value,
            &mut state_photos,
            &state_tags,
        )
        .await?;
    } else {
        return Err(ApiError::NotFoundError(format!("Tag '{}' not found", tag)));
    }

    Ok(())
}

#[tauri::command]
pub async fn get_tags(state: State<'_, PhotoState>) -> Result<HashMap<String, Tag>, String> {
    Ok(state.tags.lock().await.clone())
}

#[tauri::command]
pub async fn validate_photo(
    state: State<'_, PhotoState>,
    photo: String,
) -> Result<ValidationResult, ApiError> {
    let mut state_photos = state.photos.lock().await;

    if state_photos.contains_key(&photo) {
        let state_tags = state.tags.lock().await;
        let validation = validate_tags(&state_tags, &state_photos.get(&photo).unwrap().tags);

        let mut conn = state.db.lock().await;
        let conn = conn.as_mut().unwrap();
        for target in get_photo_targets(&photo, conn).await? {
            if state_photos.contains_key(&target.name) {
                let p = state_photos.get_mut(&target.name).unwrap();
                p.valid_tags = validation.is_valid;
                p.validation_msg = validation.message.clone();
            }
        }

        return Ok(validation);
    }
    Err(ApiError::NotFoundError(format!(
        "Photo {photo} could not be found"
    )))
}
