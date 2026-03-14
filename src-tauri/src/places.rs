use std::collections::HashMap;

use serde::Serialize;
use sqlite::Row;
use tauri::State;

use crate::{esc, photos::PhotoState, ApiError};

#[derive(Serialize, Clone)]
pub struct Layer {
    pub id: String,
    pub name: String,
    pub color: String,
    pub count: i64,
}

#[derive(Serialize, Clone)]
pub struct Place {
    pub id: String,
    pub name: String,
    pub lat: f64,
    pub lng: f64,
    pub layer: String,
    pub category: String,
    pub shape: String,
    pub count: i64,
}

#[derive(Serialize)]
pub struct Shape {
    pub id: String,
    pub shape_type: String,
    pub points: String,
    pub layer: String,
    pub name: String,
}

pub fn row_to_layer(row: &Row) -> Layer {
    Layer {
        id: row.read::<&str, _>("Id").to_string(),
        name: row.read::<&str, _>("name").to_string(),
        color: row.read::<&str, _>("color").to_string(),
        count: 0,
    }
}

pub fn row_to_place(row: &Row) -> Place {
    Place {
        id: row.read::<&str, _>("Id").to_string(),
        name: row.read::<&str, _>("name").to_string(),
        lat: row.read::<f64, _>("lat"),
        lng: row.read::<f64, _>("lng"),
        layer: row.read::<&str, _>("layer").to_string(),
        category: row.read::<&str, _>("category").to_string(),
        shape: row.read::<&str, _>("shape").to_string(),
        count: 0,
    }
}

fn row_to_shape(row: &Row) -> Shape {
    Shape {
        id: row.read::<&str, _>("Id").to_string(),
        shape_type: row.read::<&str, _>("type").to_string(),
        points: row.read::<&str, _>("points").to_string(),
        layer: row.read::<&str, _>("layer").to_string(),
        name: row.read::<&str, _>("name").to_string(),
    }
}

#[tauri::command]
pub fn get_layers(state: State<'_, PhotoState>) -> Result<HashMap<String, Layer>, ApiError> {
    Ok(state.layers.lock().unwrap().clone())
}

#[tauri::command]
pub fn get_shapes(state: State<'_, PhotoState>) -> Result<Vec<Shape>, ApiError> {
    Ok(state
        .db
        .lock()
        .unwrap()
        .prepare("SELECT * FROM Shape")?
        .into_iter()
        .map(|row| row_to_shape(&row.unwrap()))
        .collect())
}

#[tauri::command]
pub fn get_places(state: State<'_, PhotoState>) -> Result<HashMap<String, Place>, String> {
    Ok(state.places.lock().unwrap().clone())
}

#[tauri::command]
pub fn create_layer(
    state: State<'_, PhotoState>,
    id: String,
    name: String,
    color: String,
) -> Result<(), ApiError> {
    state.db.lock().unwrap().execute(format!(
        "INSERT INTO Layer VALUES ('{0}', '{1}', '{2}')",
        esc(&id),
        esc(&name),
        esc(&color)
    ))?;
    Ok(())
}

#[tauri::command]
pub fn set_layer_str(
    state: State<'_, PhotoState>,
    layer: String,
    property: String,
    value: String,
) -> Result<(), ApiError> {
    println!(
        "UPDATE Layer SET {property}='{0}' WHERE Id='{1}'",
        esc(&value),
        esc(&layer)
    );
    state.db.lock().unwrap().execute(format!(
        "UPDATE Layer SET {property}='{0}' WHERE Id='{1}'",
        esc(&value),
        esc(&layer)
    ))?;
    Ok(())
}

#[tauri::command]
pub fn delete_layer(
    state: State<'_, PhotoState>,
    layer: String,
    recursive: bool,
    new_layer: Option<String>,
) -> Result<(), ApiError> {
    let conn = state.db.lock().unwrap();

    if recursive {
        conn.execute(format!("DELETE FROM Shape WHERE layer='{0}'", esc(&layer)))?;

        let mut state_photos = state.photos.lock().unwrap();
        for place in conn
            .prepare(format!(
                "SELECT * FROM Place WHERE layer='{0}'",
                esc(&layer)
            ))?
            .into_iter()
            .map(|row| row.unwrap())
        {
            for photo in conn
                .prepare(&format!(
                    "SELECT * FROM Photo WHERE location='{0}'",
                    esc(&place.read::<&str, _>("Id").to_string())
                ))?
                .into_iter()
                .map(|row| row.unwrap())
                .collect::<Vec<Row>>()
            {
                let id = photo.read::<&str, _>("Id").to_string();
                if state_photos.contains_key(&id) {
                    state_photos.get_mut(&id).unwrap().location = String::new();
                    conn.execute(&format!(
                        "UPDATE Photo SET location='' WHERE Id='{0}'",
                        esc(&id)
                    ))?;
                } else {
                    return Err(ApiError::NotFoundError(format!(
                        "Photo with Id '{0}' not found",
                        id
                    )));
                }
            }
        }

        conn.execute(format!("DELETE FROM Place WHERE layer='{0}'", esc(&layer)))?;
    } else if new_layer.is_some() {
        let new_layer_esc = esc(&new_layer.unwrap());
        conn.execute(&format!(
            "UPDATE Shape SET layer='{new_layer_esc}' WHERE layer='{0}'",
            esc(&layer)
        ))?;
        conn.execute(&format!(
            "UPDATE Place SET layer='{new_layer_esc}' WHERE layer='{0}'",
            esc(&layer)
        ))?;
    }

    conn.execute(format!("DELETE FROM Layer WHERE Id='{0}'", esc(&layer)))?;

    Ok(())
}

#[tauri::command]
pub fn create_place(
    state: State<'_, PhotoState>,
    id: String,
    name: String,
    lat: f64,
    lng: f64,
    layer: String,
    category: String,
) -> Result<(), ApiError> {
    state.db.lock().unwrap().execute(format!(
        "INSERT INTO Place VALUES ('{0}', '{1}', {lat}, {lng}, '{2}', '{3}', '', '', '')",
        esc(&id),
        esc(&name),
        esc(&layer),
        esc(&category)
    ))?;

    let mut state_layers = state.layers.lock().unwrap();
    if state_layers.contains_key(&layer) {
        state_layers.get_mut(&layer).unwrap().count += 1;
    }

    state.places.lock().unwrap().insert(
        id.clone(),
        Place {
            id: id.clone(),
            name,
            lat,
            lng,
            layer,
            category,
            shape: String::new(),
            count: 0,
        },
    );

    Ok(())
}

#[tauri::command]
pub fn set_place_str(
    state: State<'_, PhotoState>,
    place: String,
    property: String,
    value: String,
) -> Result<(), ApiError> {
    state.db.lock().unwrap().execute(format!(
        "UPDATE Place SET {property}='{0}' WHERE Id='{1}'",
        esc(&value),
        esc(&place)
    ))?;
    Ok(())
}

#[tauri::command]
pub fn set_place_layer(
    state: State<'_, PhotoState>,
    place: String,
    layer: String,
) -> Result<(), ApiError> {
    let mut state_places = state.places.lock().unwrap();
    if !state_places.contains_key(&place) {
        return Err(ApiError::NotFoundError(format!(
            "Place with Id '{0}' not found",
            place
        )));
    }
    let state_place = state_places.get_mut(&place).unwrap();

    state.db.lock().unwrap().execute(format!(
        "UPDATE Place SET layer='{0}' WHERE Id='{1}'",
        esc(&layer),
        esc(&place)
    ))?;

    let existing_layer = &state_place.layer;
    let mut state_layers = state.layers.lock().unwrap();
    if state_layers.contains_key(existing_layer) {
        state_layers.get_mut(existing_layer).unwrap().count -= 1;
    }
    if state_layers.contains_key(&layer) {
        state_layers.get_mut(&layer).unwrap().count += 1;
    }
    state_place.layer = layer;

    Ok(())
}

#[tauri::command]
pub fn set_place_position(
    state: State<'_, PhotoState>,
    place: String,
    lat: f32,
    lng: f32,
) -> Result<(), ApiError> {
    state.db.lock().unwrap().execute(format!(
        "UPDATE Place SET (lat, lng)=({lat}, {lng}) WHERE Id='{0}'",
        esc(&place)
    ))?;
    Ok(())
}

#[tauri::command]
pub fn delete_place(state: State<'_, PhotoState>, place: String) -> Result<(), ApiError> {
    let conn = state.db.lock().unwrap();

    let mut state_places = state.places.lock().unwrap();

    if !state_places.contains_key(&place) {
        return Err(ApiError::NotFoundError(format!(
            "Place with Id '{0}' not found",
            place
        )));
    }

    let mut state_photos = state.photos.lock().unwrap();
    for photo in conn
        .prepare(&format!(
            "SELECT * FROM Photo WHERE location='{0}'",
            esc(&place)
        ))?
        .into_iter()
        .map(|row| row.unwrap())
        .collect::<Vec<Row>>()
    {
        let id = photo.read::<&str, _>("Id").to_string();
        if state_photos.contains_key(&id) {
            state_photos.get_mut(&id).unwrap().location = String::new();
            conn.execute(&format!(
                "UPDATE Photo SET location='' WHERE Id='{0}'",
                esc(&id)
            ))?;
        }
    }

    let state_place = state_places.get(&place).unwrap();
    let mut state_layers = state.layers.lock().unwrap();
    if state_layers.contains_key(&state_place.layer) {
        state_layers.get_mut(&state_place.layer).unwrap().count -= 1;
    }

    conn.execute(format!("DELETE FROM Place WHERE Id='{0}'", esc(&place)))?;
    state_places.remove(&place);
    Ok(())
}

#[tauri::command]
pub fn create_shape(
    state: State<'_, PhotoState>,
    id: String,
    shape_type: String,
    points: String,
    layer: String,
    name: String,
) -> Result<(), ApiError> {
    state.db.lock().unwrap().execute(format!(
        "INSERT INTO Shape VALUES ('{0}', '{1}', '{2}', '{3}', '{4}')",
        esc(&id),
        esc(&shape_type),
        esc(&points),
        esc(&layer),
        esc(&name)
    ))?;
    Ok(())
}

#[tauri::command]
pub fn set_shape_str(
    state: State<'_, PhotoState>,
    shape: String,
    property: String,
    value: String,
) -> Result<(), ApiError> {
    state.db.lock().unwrap().execute(format!(
        "UPDATE Shape SET {property}='{0}' WHERE Id='{1}'",
        esc(&value),
        esc(&shape)
    ))?;
    Ok(())
}

#[tauri::command]
pub fn delete_shape(state: State<'_, PhotoState>, shape: String) -> Result<(), ApiError> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!("DELETE FROM Shape WHERE Id='{0}'", esc(&shape)))?;
    Ok(())
}
