use crate::database;
use crate::photos;
use std::collections::HashMap;

#[derive(serde::Serialize, Clone)]
pub struct Person {
    pub id: String,
    pub name: String,
    pub photo: String,
    pub notes: String,
    pub category: String,
    pub photographer_count: i64,
    pub photo_count: i64,
}

#[derive(serde::Serialize, Clone)]
pub struct PersonCategory {
    pub id: String,
    pub name: String,
    pub color: String,
}

#[tauri::command]
pub async fn create_person(
    state: tauri::State<'_, photos::PhotoState>,
    id: String,
    name: String,
    photo: String,
    notes: String,
    category: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "INSERT INTO Person VALUES ('{0}', '{1}', '{2}', '{3}', '{4}')",
            database::esc(&id),
            database::esc(&name),
            database::esc(&photo),
            database::esc(&notes),
            database::esc(&category)
        ))
        .unwrap();

    state.people.lock().unwrap().insert(
        id.clone(),
        Person {
            id,
            name,
            photo,
            notes,
            category,
            photographer_count: 0,
            photo_count: 0,
        },
    );

    Ok(())
}

#[tauri::command]
pub async fn set_person_str(
    state: tauri::State<'_, photos::PhotoState>,
    person: String,
    property: String,
    value: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "UPDATE Person SET {property}='{0}' WHERE Id='{1}'",
            database::esc(&value),
            database::esc(&person)
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn create_person_category(
    state: tauri::State<'_, photos::PhotoState>,
    id: String,
    name: String,
    color: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "INSERT INTO PersonCategory VALUES ('{0}', '{1}', '{2}')",
            database::esc(&id),
            database::esc(&name),
            database::esc(&color)
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn get_people(
    state: tauri::State<'_, photos::PhotoState>,
) -> Result<HashMap<String, Person>, String> {
    Ok(state.people.lock().unwrap().clone())
}

#[tauri::command]
pub async fn get_people_categories(
    state: tauri::State<'_, photos::PhotoState>,
) -> Result<Vec::<PersonCategory>, String> {
    Ok(state.people_categories.lock().unwrap().clone())
}
