use anyhow::Context;
use log::debug;
use serde::Serialize;
use tauri::State;

use crate::{models::PersonCategory, photos::PhotoState, ApiError};

#[derive(Serialize)]
pub struct PersonDto {
    pub id: String,
    pub name: String,
    pub photo: Option<String>,
    pub category: String,
    pub photographer_count: usize,
    pub photo_count: usize,
}

#[tauri::command]
pub async fn create_person(
    state: State<'_, PhotoState>,
    id: String,
    name: String,
    category: String,
) -> Result<(), ApiError> {
    debug!("Creating new person with id {id}, name {name}, and category {category}");
    state
        .app
        .lock()
        .await
        .create_person(&id, &name, &category)
        .await
        .with_context(|| format!("Could not create person {id}"))?;

    Ok(())
}

#[tauri::command]
pub async fn create_person_category(
    state: State<'_, PhotoState>,
    id: String,
    name: String,
    color: String,
) -> Result<(), ApiError> {
    debug!("Creating new people category with id {id}, name {name}, and color {color}");
    state
        .app
        .lock()
        .await
        .create_person_category(&id, &name, &color)
        .await
        .with_context(|| format!("Could not create person category {id}"))?;

    Ok(())
}

#[tauri::command]
pub async fn set_person_name(
    state: State<'_, PhotoState>,
    person: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Setting person {person} name to {value}");
    state
        .app
        .lock()
        .await
        .set_person_name(&person, &value)
        .await
        .with_context(|| format!("Could not set person {person} name to {value}"))?;

    Ok(())
}

#[tauri::command]
pub async fn set_person_category(
    state: State<'_, PhotoState>,
    person: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Setting person {person} category to {value}");
    state
        .app
        .lock()
        .await
        .set_person_category(&person, &value)
        .await
        .with_context(|| format!("Could not set person {person} category to {value}"))?;

    Ok(())
}

#[tauri::command]
pub async fn set_person_photo(
    state: State<'_, PhotoState>,
    person: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Setting person {person} photo to {value}");
    state
        .app
        .lock()
        .await
        .set_person_photo(&person, &value)
        .await
        .with_context(|| format!("Could not set person {person} photo to {value}"))?;

    Ok(())
}

#[tauri::command]
pub async fn get_people(state: State<'_, PhotoState>) -> Result<Vec<PersonDto>, ApiError> {
    let mut app = state.app.lock().await;

    let mut results = vec![];
    for person in app
        .get_people()
        .await
        .with_context(|| "Could not get people".to_string())?
    {
        results.push(PersonDto {
            id: person.id.clone(),
            name: person.name,
            photo: person.photo,
            category: person.category,
            photographer_count: app
                .get_photographer_count(&person.id)
                .await
                .ok()
                .unwrap_or(0),
            photo_count: app.get_photo_count(&person.id).await.ok().unwrap_or(0),
        });
    }

    Ok(results)
}

#[tauri::command]
pub async fn get_people_categories(
    state: State<'_, PhotoState>,
) -> Result<Vec<PersonCategory>, ApiError> {
    Ok(state
        .app
        .lock()
        .await
        .get_people_categories()
        .await
        .with_context(|| "Could not get people categories".to_string())?)
}
