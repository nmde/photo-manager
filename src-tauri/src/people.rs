use std::collections::HashMap;

use serde::Serialize;
use sqlite::Row;
use tauri::State;

use crate::{esc, photos::PhotoState, ApiError};

#[derive(Serialize, Clone)]
pub struct Person {
    pub id: String,
    pub name: String,
    pub photo: String,
    pub notes: String,
    pub category: String,
    pub photographer_count: i64,
    pub photo_count: i64,
}

#[derive(Serialize, Clone)]
pub struct PersonCategory {
    pub id: String,
    pub name: String,
    pub color: String,
}

pub fn row_to_person(row: &Row) -> Person {
    Person {
        id: row.read::<&str, _>("Id").to_string(),
        name: row.read::<&str, _>("name").to_string(),
        photo: row.read::<&str, _>("photo").to_string(),
        notes: row.read::<&str, _>("notes").to_string(),
        category: row.read::<&str, _>("category").to_string(),
        photo_count: 0,
        photographer_count: 0,
    }
}

fn row_to_person_category(row: &Row) -> PersonCategory {
    PersonCategory {
        id: row.read::<&str, _>("Id").to_string(),
        name: row.read::<&str, _>("name").to_string(),
        color: row.read::<&str, _>("color").to_string(),
    }
}

#[tauri::command]
pub fn create_person(
    state: State<'_, PhotoState>,
    id: String,
    name: String,
    notes: String,
    category: String,
) -> Result<(), ApiError> {
    state.db.lock().unwrap().execute(format!(
        "INSERT INTO Person VALUES ('{0}', '{1}', '', '{2}', '{3}')",
        esc(&id),
        esc(&name),
        esc(&notes),
        esc(&category)
    ))?;

    state.people.lock().unwrap().insert(
        id.clone(),
        Person {
            id,
            name,
            photo: String::new(),
            notes,
            category,
            photographer_count: 0,
            photo_count: 0,
        },
    );

    Ok(())
}

#[tauri::command]
pub fn create_person_category(
    state: State<'_, PhotoState>,
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
            esc(&id),
            esc(&name),
            esc(&color)
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub fn set_person_name(
    state: State<'_, PhotoState>,
    person: String,
    value: String,
) -> Result<(), ApiError> {
    state.db.lock().unwrap().execute(format!(
        "UPDATE Person SET name='{0}' WHERE Id='{1}'",
        esc(&value),
        esc(&person)
    ))?;

    let mut state_people = state.people.lock().unwrap();
    if state_people.contains_key(&person) {
        state_people.get_mut(&person).unwrap().name = value;
    }

    Ok(())
}

#[tauri::command]
pub fn set_person_category(
    state: State<'_, PhotoState>,
    person: String,
    value: String,
) -> Result<(), ApiError> {
    state.db.lock().unwrap().execute(format!(
        "UPDATE Person SET category='{0}' WHERE Id='{1}'",
        esc(&value),
        esc(&person)
    ))?;

    let mut state_people = state.people.lock().unwrap();
    if state_people.contains_key(&person) {
        state_people.get_mut(&person).unwrap().category = value;
    }

    Ok(())
}

#[tauri::command]
pub fn set_person_notes(
    state: State<'_, PhotoState>,
    person: String,
    value: String,
) -> Result<(), ApiError> {
    state.db.lock().unwrap().execute(format!(
        "UPDATE Person SET notes='{0}' WHERE Id='{1}'",
        esc(&value),
        esc(&person)
    ))?;

    let mut state_people = state.people.lock().unwrap();
    if state_people.contains_key(&person) {
        state_people.get_mut(&person).unwrap().notes = value;
    }

    Ok(())
}

#[tauri::command]
pub fn set_person_photo(
    state: State<'_, PhotoState>,
    person: String,
    value: String,
) -> Result<(), ApiError> {
    state.db.lock().unwrap().execute(format!(
        "UPDATE Person SET photo='{0}' WHERE Id='{1}'",
        esc(&value),
        esc(&person)
    ))?;

    let mut state_people = state.people.lock().unwrap();
    if state_people.contains_key(&person) {
        state_people.get_mut(&person).unwrap().photo = value;
    }

    Ok(())
}

#[tauri::command]
pub fn get_people(state: State<'_, PhotoState>) -> Result<HashMap<String, Person>, String> {
    Ok(state.people.lock().unwrap().clone())
}

#[tauri::command]
pub fn get_people_categories(
    state: State<'_, PhotoState>,
) -> Result<Vec<PersonCategory>, ApiError> {
    let mut results = Vec::<PersonCategory>::new();
    for row in state
        .db
        .lock()
        .unwrap()
        .prepare("SELECT * FROM PersonCategory")?
    {
        results.push(row_to_person_category(&row?));
    }
    Ok(results)
}
