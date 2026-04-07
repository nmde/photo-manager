use anyhow::Context;
use log::debug;
use serde::Serialize;
use tauri::State;

use crate::{models::Shape, photos::PhotoState, ApiError};

#[derive(Serialize)]
pub struct LayerDto {
    pub id: String,
    pub name: String,
    pub color: String,
    pub count: usize,
}

#[derive(Serialize)]
pub struct PlaceDto {
    pub id: String,
    pub name: String,
    pub lat: f32,
    pub lng: f32,
    pub layer: String,
    pub category: String,
    pub shape: Option<String>,
    pub count: usize,
}

#[tauri::command]
pub async fn get_layers(state: State<'_, PhotoState>) -> Result<Vec<LayerDto>, ApiError> {
    let mut app = state.app.lock().await;

    let mut results = vec![];
    for layer in app
        .get_layers()
        .await
        .with_context(|| "Could not get layers".to_string())?
    {
        results.push(LayerDto {
            id: layer.id.clone(),
            name: layer.name.clone(),
            color: layer.color.clone(),
            count: app.count_places_in_layer(&layer.id).await.ok().unwrap_or(0),
        });
    }

    Ok(results)
}

#[tauri::command]
pub async fn get_shapes(state: State<'_, PhotoState>) -> Result<Vec<Shape>, ApiError> {
    Ok(state
        .app
        .lock()
        .await
        .get_shapes()
        .await
        .with_context(|| "Could not get shapes".to_string())?)
}

#[tauri::command]
pub async fn get_places(state: State<'_, PhotoState>) -> Result<Vec<PlaceDto>, ApiError> {
    let mut app = state.app.lock().await;

    let mut results = vec![];
    for place in app
        .get_places()
        .await
        .with_context(|| "Could not get places".to_string())?
    {
        results.push(PlaceDto {
            id: place.id.clone(),
            name: place.name.clone(),
            lat: place.lat,
            lng: place.lng,
            layer: place.layer.clone(),
            category: place.category.clone(),
            shape: place.shape.clone(),
            count: app.count_photos_in_place(&place.id).await.ok().unwrap_or(0),
        });
    }

    Ok(results)
}

#[tauri::command]
pub async fn create_layer(
    state: State<'_, PhotoState>,
    id: String,
    name: String,
    color: String,
) -> Result<(), ApiError> {
    debug!("Creating layer with id {id}, name {name} and color {color}");
    state
        .app
        .lock()
        .await
        .create_layer(&id, &name, &color)
        .await
        .with_context(|| "Could not create layer".to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn set_layer_color(
    state: State<'_, PhotoState>,
    layer: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Setting layer {layer} color to {value}");
    state
        .app
        .lock()
        .await
        .set_layer_color(&layer, &value)
        .await
        .with_context(|| format!("Could not set layer {0} color to {1}", layer, value))?;

    Ok(())
}

#[tauri::command]
pub async fn delete_layer(
    state: State<'_, PhotoState>,
    layer: String,
    recursive: bool,
    new_layer: Option<String>,
) -> Result<(), ApiError> {
    debug!("Deleting layer {layer}");
    state
        .app
        .lock()
        .await
        .delete_layer(&layer, recursive, &new_layer)
        .await
        .with_context(|| format!("Could not delete layer {0}", layer))?;

    Ok(())
}

#[tauri::command]
pub async fn create_place(
    state: State<'_, PhotoState>,
    id: String,
    name: String,
    lat: f32,
    lng: f32,
    layer: String,
    category: String,
) -> Result<(), ApiError> {
    debug!("Creating place {name} at {lat},{lng}");
    state
        .app
        .lock()
        .await
        .create_place(id, &name, lat, lng, &layer, &category)
        .await
        .with_context(|| format!("Could not create place {0}", name))?;

    Ok(())
}

#[tauri::command]
pub async fn set_place_name(
    state: State<'_, PhotoState>,
    place: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Set place {place} name to {value}");
    state
        .app
        .lock()
        .await
        .set_place_name(&place, &value)
        .await
        .with_context(|| format!("Could not set place {0} name to {1}", place, value))?;

    Ok(())
}

#[tauri::command]
pub async fn set_place_category(
    state: State<'_, PhotoState>,
    place: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Set place {place} category to {value}");
    state
        .app
        .lock()
        .await
        .set_place_category(&place, &value)
        .await
        .with_context(|| format!("Could not set place {0} category to {1}", place, value))?;

    Ok(())
}

#[tauri::command]
pub async fn set_place_shape(
    state: State<'_, PhotoState>,
    place: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Set place {place} shape to {value}");
    state
        .app
        .lock()
        .await
        .set_place_shape(&place, &value)
        .await
        .with_context(|| {
            format!(
                "Could not set place {0} shape to {1}",
                place,
                if value.is_empty() {
                    "None".to_string()
                } else {
                    value
                }
            )
        })?;

    Ok(())
}

#[tauri::command]
pub async fn set_place_layer(
    state: State<'_, PhotoState>,
    place: String,
    layer: String,
) -> Result<(), ApiError> {
    debug!("Set place {place} layer to {layer}");
    state
        .app
        .lock()
        .await
        .set_place_layer(&place, &layer)
        .await
        .with_context(|| format!("Could not set place {0} layer to {1}", place, layer))?;

    Ok(())
}

#[tauri::command]
pub async fn set_place_position(
    state: State<'_, PhotoState>,
    place: String,
    lat: f32,
    lng: f32,
) -> Result<(), ApiError> {
    debug!("Set place {place} position to {lat},{lng}");
    state
        .app
        .lock()
        .await
        .set_place_position(&place, lat, lng)
        .await
        .with_context(|| {
            format!(
                "Could not set place {0} position to {1},{2}",
                place, lat, lng
            )
        })?;

    Ok(())
}

#[tauri::command]
pub async fn delete_place(state: State<'_, PhotoState>, place: String) -> Result<(), ApiError> {
    debug!("Deleting place {place}");
    state
        .app
        .lock()
        .await
        .delete_place(&place)
        .await
        .with_context(|| format!("Could not delete place {0}", place))?;

    Ok(())
}

#[tauri::command]
pub async fn create_shape(
    state: State<'_, PhotoState>,
    id: String,
    shape_type: String,
    points: String,
    layer: String,
    name: String,
) -> Result<(), ApiError> {
    debug!("Creating shape {id}");
    state
        .app
        .lock()
        .await
        .create_shape(&id, &shape_type, &points, &layer, &name)
        .await
        .with_context(|| "Could not create shape".to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn set_shape_points(
    state: State<'_, PhotoState>,
    shape: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Setting shape {shape} points");
    state
        .app
        .lock()
        .await
        .set_shape_points(&shape, &value)
        .await
        .with_context(|| format!("Could not set shape {0} points", shape))?;

    Ok(())
}

#[tauri::command]
pub async fn set_shape_layer(
    state: State<'_, PhotoState>,
    shape: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Setting shape {shape} layer to {value}");
    state
        .app
        .lock()
        .await
        .set_shape_layer(&shape, &value)
        .await
        .with_context(|| format!("Could not set shape {0} layer to {1}", shape, value))?;

    Ok(())
}

#[tauri::command]
pub async fn set_shape_name(
    state: State<'_, PhotoState>,
    shape: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Setting shape {shape} name to {value}");
    state
        .app
        .lock()
        .await
        .set_shape_name(&shape, &value)
        .await
        .with_context(|| format!("Could not set shape {0} name to {1}", shape, value))?;

    Ok(())
}

#[tauri::command]
pub async fn delete_shape(state: State<'_, PhotoState>, shape: String) -> Result<(), ApiError> {
    debug!("Deleting shape {shape}");
    state
        .app
        .lock()
        .await
        .delete_shape(&shape)
        .await
        .with_context(|| format!("Could not delete shape {0}", shape))?;

    Ok(())
}
