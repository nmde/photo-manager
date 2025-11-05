use crate::database;
use crate::photos;
use std::collections::HashMap;

#[derive(serde::Serialize, Clone)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: String,
    pub prereqs: Vec<String>,
    pub coreqs: Vec<String>,
    pub incompatible: Vec<String>,
    pub count: i64,
}

#[derive(serde::Serialize)]
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
    state: tauri::State<'_, photos::PhotoState>,
    tag: String,
    value: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "UPDATE Tag SET color='{0}' WHERE name='{1}'",
            database::esc(&value),
            database::esc(&tag)
        ))
        .unwrap();

    state.tags.lock().unwrap().get_mut(&tag).unwrap().color = value;

    Ok(())
}

#[tauri::command]
pub async fn set_tag_prereqs(
    state: tauri::State<'_, photos::PhotoState>,
    tag: String,
    value: Vec<String>,
) -> Result<(), String> {
    let connection = state.db.lock().unwrap();
    let mut state_tags = state.tags.lock().unwrap();
    connection
        .execute(format!(
            "UPDATE Tag SET prereqs='{0}' WHERE name='{1}'",
            database::esc(&value.join(",")),
            database::esc(&tag)
        ))
        .unwrap();

    state_tags.get_mut(&tag).unwrap().prereqs = value;

    // Gets photos from the database with the tag as a substring in their tags field
    let maybe_has_tag = connection
        .prepare(format!(
            "SELECT * FROM Photo WHERE tags LIKE '%{0}%'",
            database::esc(&tag)
        ))
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    let mut state_photos = state.photos.lock().unwrap();
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
pub async fn set_tag_coreqs(
    state: tauri::State<'_, photos::PhotoState>,
    tag: String,
    value: Vec<String>,
) -> Result<(), String> {
    let connection = state.db.lock().unwrap();
    let mut state_tags = state.tags.lock().unwrap();
    connection
        .execute(format!(
            "UPDATE Tag SET coreqs='{0}' WHERE name='{1}'",
            database::esc(&value.join(",")),
            database::esc(&tag)
        ))
        .unwrap();

    state_tags.get_mut(&tag).unwrap().coreqs = value;

    // Gets photos from the database with the tag as a substring in their tags field
    let maybe_has_tag = connection
        .prepare(format!(
            "SELECT * FROM Photo WHERE tags LIKE '%{0}%'",
            database::esc(&tag)
        ))
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    let mut state_photos = state.photos.lock().unwrap();
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
pub async fn set_tag_incompatible(
    state: tauri::State<'_, photos::PhotoState>,
    tag: String,
    value: Vec<String>,
) -> Result<(), String> {
    let connection = state.db.lock().unwrap();
    let mut state_tags = state.tags.lock().unwrap();
    connection
        .execute(format!(
            "UPDATE Tag SET incompatible='{0}' WHERE name='{1}'",
            database::esc(&value.join(",")),
            database::esc(&tag)
        ))
        .unwrap();

    state_tags.get_mut(&tag).unwrap().incompatible = value;

    // Gets photos from the database with the tag as a substring in their tags field
    let maybe_has_tag = connection
        .prepare(format!(
            "SELECT * FROM Photo WHERE tags LIKE '%{0}%'",
            database::esc(&tag)
        ))
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap());

    let mut state_photos = state.photos.lock().unwrap();
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
pub async fn get_tags(
    state: tauri::State<'_, photos::PhotoState>,
) -> Result<HashMap<String, Tag>, String> {
    Ok(state.tags.lock().unwrap().clone())
}

#[derive(serde::Serialize)]
pub struct TagStats {
    pub avg_count: i64,
    pub avg_rating: i64,
}

#[tauri::command]
pub async fn get_tag_stats(
    state: tauri::State<'_, photos::PhotoState>,
) -> Result<TagStats, String> {
    let mut photo_count = 0i64;
    let mut total_count = 0i64;
    let mut total_rating = 0;
    for (_id, photo) in state.photos.lock().unwrap().iter() {
        photo_count += 1;
        total_count += photo.tags.len() as i64;
        total_rating += photo.rating;
    }

    Ok(TagStats {
        avg_count: total_count / photo_count,
        avg_rating: total_rating / photo_count,
    })
}
