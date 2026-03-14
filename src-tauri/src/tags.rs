use std::collections::HashMap;

use serde::Serialize;
use sqlite::{Connection, Row};
use tauri::State;
use uuid::Uuid;

use crate::{
    esc,
    photos::{get_photo_targets, Photo, PhotoState},
    row_to_vec, ApiError,
};

#[derive(Serialize, Clone)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: String,
    pub prereqs: Vec<String>,
    pub coreqs: Vec<String>,
    pub incompatible: Vec<String>,
    pub count: i64,
    pub has_entry: bool,
}

impl Tag {
    pub fn new(name: &String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.clone(),
            color: String::new(),
            prereqs: Vec::<String>::new(),
            coreqs: Vec::<String>::new(),
            incompatible: Vec::<String>::new(),
            count: 0,
            has_entry: false,
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

pub fn row_to_tag(row: &Row) -> Tag {
    Tag {
        id: row.read::<&str, _>("Id").to_string(),
        name: row.read::<&str, _>("name").to_string(),
        color: row.read::<&str, _>("color").to_string(),
        prereqs: row_to_vec(row, "prereqs"),
        coreqs: row_to_vec(row, "coreqs"),
        incompatible: row_to_vec(row, "incompatible"),
        count: 0,
        has_entry: true,
    }
}

fn ensure_tag(tag: &mut Tag, conn: &Connection) -> Result<(), sqlite::Error> {
    if !tag.has_entry {
        conn.execute(format!(
            "INSERT INTO Tag VALUES ('{0}', '{1}', '{2}', '{3}', '{4}', '{5}')",
            esc(&tag.id),
            esc(&tag.name),
            esc(&tag.color),
            esc(&tag.prereqs.join(",")),
            esc(&tag.coreqs.join(",")),
            esc(&tag.incompatible.join(",")),
        ))?;
        tag.has_entry = true;
    }

    Ok(())
}

#[tauri::command]
pub fn set_tag_color(
    state: State<'_, PhotoState>,
    tag: String,
    value: String,
) -> Result<(), ApiError> {
    let conn = state.db.lock().unwrap();
    let mut state_tags = state.tags.lock().unwrap();

    if state_tags.contains_key(&tag) {
        let mut state_tag = state_tags.get_mut(&tag).unwrap();
        ensure_tag(&mut state_tag, &conn)?;

        conn.execute(format!(
            "UPDATE Tag SET color='{0}' WHERE name='{1}'",
            esc(&value),
            esc(&tag)
        ))?;

        state_tag.color = value.clone();
    } else {
        return Err(ApiError::NotFoundError(format!("Tag '{}' not found", tag)));
    }

    Ok(())
}

fn modify_tag_relationships(
    connection: &Connection,
    category: &str,
    tag: &String,
    value: &Vec<String>,
    state_photos: &mut HashMap<String, Photo>,
    state_tags: &HashMap<String, Tag>,
) -> Result<(), ApiError> {
    connection.execute(format!(
        "UPDATE Tag SET {category}='{0}' WHERE name='{1}'",
        esc(&value.join(",")),
        esc(tag)
    ))?;
    // Gets photos from the database with the tag as a substring in their tags field
    let maybe_has_tag = connection
        .prepare(format!(
            "SELECT * FROM Photo WHERE tags LIKE '%{0}%'",
            esc(tag)
        ))?
        .into_iter()
        .map(|row| row.unwrap());

    for row in maybe_has_tag {
        let mut tags = Vec::<String>::new();
        let tags_row = row.read::<&str, _>("tags").to_string();
        if tags_row.len() > 0 {
            tags = tags_row.split(",").map(str::to_string).collect();
        }
        if tags.contains(&tag) {
            let target = state_photos
                .get_mut(&row.read::<&str, _>("Id").to_string())
                .unwrap();
            let validation = validate_tags(&state_tags, &tags);
            target.valid_tags = validation.is_valid;
            target.validation_msg = validation.message;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn set_tag_prereqs(
    state: State<'_, PhotoState>,
    tag: String,
    value: Vec<String>,
) -> Result<(), ApiError> {
    let connection = state.db.lock().unwrap();
    let mut state_tags = state.tags.lock().unwrap();
    let mut state_photos = state.photos.lock().unwrap();

    if state_tags.contains_key(&tag) {
        let mut state_tag = state_tags.get_mut(&tag).unwrap();
        ensure_tag(&mut state_tag, &connection)?;

        state_tag.prereqs = value.clone();
        modify_tag_relationships(
            &connection,
            "prereqs",
            &tag,
            &value,
            &mut state_photos,
            &state_tags,
        )?;
    } else {
        return Err(ApiError::NotFoundError(format!("Tag '{}' not found", tag)));
    }

    Ok(())
}

#[tauri::command]
pub fn set_tag_coreqs(
    state: State<'_, PhotoState>,
    tag: String,
    value: Vec<String>,
) -> Result<(), ApiError> {
    let connection = state.db.lock().unwrap();
    let mut state_tags = state.tags.lock().unwrap();
    let mut state_photos = state.photos.lock().unwrap();

    if state_tags.contains_key(&tag) {
        let mut state_tag = state_tags.get_mut(&tag).unwrap();
        ensure_tag(&mut state_tag, &connection)?;

        state_tag.coreqs = value.clone();
        modify_tag_relationships(
            &connection,
            "coreqs",
            &tag,
            &value,
            &mut state_photos,
            &state_tags,
        )?;
    } else {
        return Err(ApiError::NotFoundError(format!("Tag '{}' not found", tag)));
    }

    Ok(())
}

#[tauri::command]
pub fn set_tag_incompatible(
    state: State<'_, PhotoState>,
    tag: String,
    value: Vec<String>,
) -> Result<(), ApiError> {
    let connection = state.db.lock().unwrap();
    let mut state_tags = state.tags.lock().unwrap();
    let mut state_photos = state.photos.lock().unwrap();

    if state_tags.contains_key(&tag) {
        let mut state_tag = state_tags.get_mut(&tag).unwrap();
        ensure_tag(&mut state_tag, &connection)?;

        state_tag.incompatible = value.clone();
        modify_tag_relationships(
            &connection,
            "incompatible",
            &tag,
            &value,
            &mut state_photos,
            &state_tags,
        )?;
    } else {
        return Err(ApiError::NotFoundError(format!("Tag '{}' not found", tag)));
    }

    Ok(())
}

#[tauri::command]
pub fn get_tags(state: State<'_, PhotoState>) -> Result<HashMap<String, Tag>, String> {
    Ok(state.tags.lock().unwrap().clone())
}

#[tauri::command]
pub fn validate_photo(
    state: State<'_, PhotoState>,
    photo: String,
) -> Result<ValidationResult, ApiError> {
    let mut state_photos = state.photos.lock().unwrap();

    if state_photos.contains_key(&photo) {
        let validation = validate_tags(
            &state.tags.lock().unwrap(),
            &state_photos.get(&photo).unwrap().tags,
        );

        for target in get_photo_targets(&photo, &state.db.lock().unwrap())? {
            if state_photos.contains_key(&target.id) {
                let p = state_photos.get_mut(&target.id).unwrap();
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
