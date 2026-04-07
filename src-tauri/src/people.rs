use std::collections::HashMap;

use diesel::{
    dsl::{insert_into, update},
    ExpressionMethods, QueryDsl,
};
use diesel_async::RunQueryDsl;
use log::debug;
use tauri::State;

use crate::{
    models::{Person, PersonCategory},
    photos::PhotoState,
    schema::{people, people_categories},
    ApiError,
};

#[tauri::command]
pub async fn create_person(
    state: State<'_, PhotoState>,
    id: String,
    name: String,
    category: String,
) -> Result<(), ApiError> {
    debug!("Creating new person with id {id}, name {name}, and category {category}");
    let mut conn = state.db.lock().await;
    let conn = conn.as_mut().unwrap();

    let new_person = Person {
        id: id.clone(),
        name,
        photo: None,
        category,
    };

    insert_into(people::table)
        .values(new_person.clone())
        .execute(conn)
        .await?;

    state.people.lock().await.insert(id.clone(), new_person);

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
    let mut conn = state.db.lock().await;
    let conn = conn.as_mut().unwrap();

    insert_into(people_categories::table)
        .values(PersonCategory { id, name, color })
        .execute(conn)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn set_person_name(
    state: State<'_, PhotoState>,
    person: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Setting person {person} name to {value}");
    let mut conn = state.db.lock().await;
    let conn = conn.as_mut().unwrap();

    update(people::table.filter(people::id.eq(person.clone())))
        .set(people::name.eq(value.clone()))
        .execute(conn)
        .await?;

    let mut state_people = state.people.lock().await;
    if state_people.contains_key(&person) {
        state_people.get_mut(&person).unwrap().name = value;
    }

    Ok(())
}

#[tauri::command]
pub async fn set_person_category(
    state: State<'_, PhotoState>,
    person: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Setting person {person} category to {value}");
    let mut conn = state.db.lock().await;
    let conn = conn.as_mut().unwrap();

    update(people::table.filter(people::id.eq(person.clone())))
        .set(people::category.eq(value.clone()))
        .execute(conn)
        .await?;

    let mut state_people = state.people.lock().await;
    if state_people.contains_key(&person) {
        state_people.get_mut(&person).unwrap().category = value;
    }

    Ok(())
}

#[tauri::command]
pub async fn set_person_photo(
    state: State<'_, PhotoState>,
    person: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Setting person {person} photo to {value}");
    let mut conn = state.db.lock().await;
    let conn = conn.as_mut().unwrap();

    update(people::table.filter(people::id.eq(person.clone())))
        .set(people::photo.eq(value.clone()))
        .execute(conn)
        .await?;

    let mut state_people = state.people.lock().await;
    if state_people.contains_key(&person) {
        state_people.get_mut(&person).unwrap().photo = Some(value);
    }

    Ok(())
}

#[tauri::command]
pub async fn get_people(state: State<'_, PhotoState>) -> Result<HashMap<String, Person>, String> {
    Ok(state.people.lock().await.clone())
}

#[tauri::command]
pub async fn get_people_categories(
    state: State<'_, PhotoState>,
) -> Result<Vec<PersonCategory>, ApiError> {
    let mut conn = state.db.lock().await;
    let conn = conn.as_mut().unwrap();

    Ok(people_categories::table
        .load::<PersonCategory>(conn)
        .await?)
}
