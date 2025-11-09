use crate::database;
use crate::photos;
use std::collections::HashMap;

#[derive(serde::Serialize)]
pub struct Layer {
    pub id: String,
    pub name: String,
    pub color: String,
}

#[derive(serde::Serialize, Clone)]
pub struct Place {
    pub id: String,
    pub name: String,
    pub lat: f64,
    pub lng: f64,
    pub layer: String,
    pub category: String,
    pub shape: String,
    pub tags: Vec<String>,
    pub notes: String,
    pub count: i64,
}

#[derive(serde::Serialize)]
pub struct Shape {
    pub id: String,
    pub shape_type: String,
    pub points: String,
    pub layer: String,
    pub name: String,
}

#[tauri::command]
pub async fn get_layers(state: tauri::State<'_, photos::PhotoState>) -> Result<Vec<Layer>, String> {
    let mut layers = Vec::<Layer>::new();
    for row in state
        .db
        .lock()
        .unwrap()
        .prepare("SELECT * FROM Layer")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap())
    {
        layers.push(Layer {
            id: row.read::<&str, _>("Id").to_string(),
            name: row.read::<&str, _>("name").to_string(),
            color: row.read::<&str, _>("color").to_string(),
        });
    }

    Ok(layers)
}

#[tauri::command]
pub async fn get_shapes(state: tauri::State<'_, photos::PhotoState>) -> Result<Vec<Shape>, String> {
    let mut shapes = Vec::<Shape>::new();
    for row in state
        .db
        .lock()
        .unwrap()
        .prepare("SELECT * FROM Shape")
        .unwrap()
        .into_iter()
        .map(|row| row.unwrap())
    {
        shapes.push(Shape {
            id: row.read::<&str, _>("Id").to_string(),
            shape_type: row.read::<&str, _>("type").to_string(),
            points: row.read::<&str, _>("points").to_string(),
            layer: row.read::<&str, _>("layer").to_string(),
            name: row.read::<&str, _>("name").to_string(),
        });
    }

    Ok(shapes)
}

#[tauri::command]
pub async fn get_places(
    state: tauri::State<'_, photos::PhotoState>,
) -> Result<HashMap<String, Place>, String> {
    Ok(state.places.lock().unwrap().clone())
}

#[tauri::command]
pub async fn create_layer(
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
            "INSERT INTO Layer VALUES ('{0}', '{1}', '{2}')",
            database::esc(&id),
            database::esc(&name),
            database::esc(&color)
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn set_layer_color(
    state: tauri::State<'_, photos::PhotoState>,
    layer: String,
    color: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "UPDATE Layer SET color='{0}' WHERE Id='{1}'",
            database::esc(&color),
            database::esc(&layer)
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn create_place(
    state: tauri::State<'_, photos::PhotoState>,
    id: String,
    name: String,
    lat: f64,
    lng: f64,
    layer: String,
    category: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "INSERT INTO Place VALUES ('{0}', '{1}', {lat}, {lng}, '{2}', '{3}', '', '', '')",
            database::esc(&id),
            database::esc(&name),
            database::esc(&layer),
            database::esc(&category)
        ))
        .unwrap();

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
            tags: Vec::<String>::new(),
            notes: String::new(),
            count: 0,
        },
    );

    *state.newest_place.lock().unwrap() = id;

    Ok(())
}

#[tauri::command]
pub async fn set_place_str(
    state: tauri::State<'_, photos::PhotoState>,
    place: String,
    property: String,
    value: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "UPDATE Place SET {property}='{0}' WHERE Id='{1}'",
            database::esc(&value),
            database::esc(&place)
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn set_place_position(
    state: tauri::State<'_, photos::PhotoState>,
    place: String,
    lat: f32,
    lng: f32,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "UPDATE Place SET (lat, lng)=({lat}, {lng}) WHERE Id='{0}'",
            database::esc(&place)
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn delete_place(
    state: tauri::State<'_, photos::PhotoState>,
    place: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "DELETE FROM Place WHERE Id='{0}'",
            database::esc(&place)
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn create_shape(
    state: tauri::State<'_, photos::PhotoState>,
    id: String,
    shape_type: String,
    points: String,
    layer: String,
    name: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "INSERT INTO Shape VALUES ('{0}', '{1}', '{2}', '{3}', '{4}')",
            database::esc(&id),
            database::esc(&shape_type),
            database::esc(&points),
            database::esc(&layer),
            database::esc(&name)
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn set_shape_str(
    state: tauri::State<'_, photos::PhotoState>,
    shape: String,
    property: String,
    value: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "UPDATE Shape SET {property}='{0}' WHERE Id='{1}'",
            database::esc(&value),
            database::esc(&shape)
        ))
        .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn delete_shape(
    state: tauri::State<'_, photos::PhotoState>,
    shape: String,
) -> Result<(), String> {
    state
        .db
        .lock()
        .unwrap()
        .execute(format!(
            "DELETE FROM Shape WHERE Id='{0}'",
            database::esc(&shape)
        ))
        .unwrap();
    Ok(())
}
