use std::collections::HashMap;

use anyhow::Context;
use log::debug;

use crate::{
    app::ApiError,
    models::{Person, PersonCategory},
    people::{
        create_person as _create_person, create_person_category as _create_person_category,
        get_people as _get_people, get_people_categories as _get_people_categories, PEOPLE,
    },
};

#[tauri::command]
pub async fn create_person(id: String, name: String, category: String) -> Result<(), ApiError> {
    debug!("Creating new person with id {id}, name {name}, and category {category}");
    _create_person(&id, &name, &category)
        .await
        .with_context(|| format!("Could not create person {id}"))?;

    Ok(())
}

#[tauri::command]
pub async fn create_person_category(
    id: String,
    name: String,
    color: String,
) -> Result<(), ApiError> {
    debug!("Creating new people category with id {id}, name {name}, and color {color}");
    _create_person_category(&id, &name, &color)
        .await
        .with_context(|| format!("Could not create person category {id}"))?;

    Ok(())
}

#[tauri::command]
pub async fn set_person_name(person: String, value: String) -> Result<(), ApiError> {
    debug!("Setting person {person} name to {value}");

    let mut people = PEOPLE.lock().await;
    let target = people.get_mut(&person);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Person {person} not found!")));
    }

    target
        .unwrap()
        .set_person_name(&person, &value)
        .await
        .with_context(|| format!("Could not set person {person} name to {value}"))?;

    Ok(())
}

#[tauri::command]
pub async fn set_person_category(person: String, value: String) -> Result<(), ApiError> {
    debug!("Setting person {person} category to {value}");

    let mut people = PEOPLE.lock().await;
    let target = people.get_mut(&person);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Person {person} not found!")));
    }

    target
        .unwrap()
        .set_person_category(&person, &value)
        .await
        .with_context(|| format!("Could not set person {person} category to {value}"))?;

    Ok(())
}

#[tauri::command]
pub async fn set_person_photo(person: String, value: Option<String>) -> Result<(), ApiError> {
    debug!(
        "Setting person {person} photo to {}",
        value.as_ref().unwrap_or(&"NULL".to_string())
    );

    let mut people = PEOPLE.lock().await;
    let target = people.get_mut(&person);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Person {person} not found!")));
    }

    target
        .unwrap()
        .set_person_photo(&person, &value)
        .await
        .with_context(|| {
            format!(
                "Could not set person {person} photo to {}",
                value.unwrap_or("NULL".to_string())
            )
        })?;

    Ok(())
}

#[tauri::command]
pub async fn get_people() -> Result<HashMap<String, Person>, ApiError> {
    Ok(_get_people()
        .await
        .with_context(|| "Could not get people".to_string())?)
}

#[tauri::command]
pub async fn get_people_categories() -> Result<Vec<PersonCategory>, ApiError> {
    Ok(_get_people_categories()
        .await
        .with_context(|| "Could not get people categories".to_string())?)
}
