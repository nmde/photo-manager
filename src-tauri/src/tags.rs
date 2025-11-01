use crate::database;
use crate::photos;
use std::collections::HashMap;
use unique_id::string::StringGenerator;
use unique_id::Generator;

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

        for tag in tags {
            if state_tags.contains_key(tag) {
                let tag_data = state_tags.get(tag).unwrap();
                let mut prereqs_met = true;
                for prereq in &tag_data.prereqs {
                    if !tags.contains(prereq) {
                        valid = false;
                        if prereqs_met {
                            message.push_str("Missing prerequisite tag(s): ");
                            prereqs_met = false;
                        }
                        message.push_str(prereq);
                    }
                }
                let mut coreqs_met = true;
                for coreq in &tag_data.coreqs {
                    if !tags.contains(coreq) {
                        valid = false;
                        if coreqs_met {
                            message.push_str("Missing corequisite tag(s): ");
                            coreqs_met = false;
                        }
                        message.push_str(coreq);
                    }
                }
                let mut inc_met = true;
                for incompatible in &tag_data.incompatible {
                    if tags.contains(incompatible) {
                        valid = false;
                        if inc_met {
                            message.push_str("Incompatible tag(s) present: ");
                            inc_met = false;
                        }
                        message.push_str(incompatible);
                    }
                }
            }
        }

        ValidationResult {
            is_valid: valid,
            message,
        }
    }
}

pub fn create_new_tags(state: &tauri::State<'_, photos::PhotoState>, tags: &Vec<String>) {
    let connection = state.db.lock().unwrap();
    let mut state_tags = state.tags.lock().unwrap();
    let id_generator = StringGenerator::default();

    for tag in tags {
        if !state_tags.contains_key(tag) {
            let id = id_generator.next_id();
            connection
                .execute(format!(
                    "INSERT INTO Tag VALUES ('{0}', '{1}', '', '', '', '')",
                    database::esc(&id),
                    database::esc(&tag)
                ))
                .unwrap();
            state_tags.insert(
                tag.to_string(),
                Tag {
                    id,
                    name: tag.to_string(),
                    color: String::new(),
                    prereqs: Vec::<String>::new(),
                    coreqs: Vec::<String>::new(),
                    incompatible: Vec::<String>::new(),
                    count: 0,
                },
            );
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
        .map(|row| photos::row_to_photo(&state, row.unwrap()));

    let mut state_photos = state.photos.lock().unwrap();
    for photo in maybe_has_tag {
        if photo.tags.contains(&tag) {
            match state_photos.binary_search_by_key(&photo.id, |p| p.id.clone()) {
                Ok(pos) => {
                    let validation = validate_tags(&state_tags, &photo.tags);
                    state_photos[pos].valid_tags = validation.is_valid;
                    state_photos[pos].validation_msg = validation.message;
                }
                Err(_pos) => {}
            }
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
