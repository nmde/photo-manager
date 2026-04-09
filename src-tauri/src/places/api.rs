use std::collections::HashMap;

use anyhow::Context;
use log::debug;

use crate::{
    app::ApiError,
    models::{Layer, Place, Shape},
    places::{
        create_layer as _create_layer, create_place as _create_place,
        create_shape as _create_shape, delete_layer as _delete_layer,
        delete_place as _delete_place, delete_shape as _delete_shape, get_layers as _get_layers,
        get_places as _get_places, get_shapes as _get_shapes, LAYERS, PLACES, SHAPES,
    },
};

#[tauri::command]
pub async fn get_layers() -> Result<HashMap<String, Layer>, ApiError> {
    Ok(_get_layers()
        .await
        .with_context(|| "Could not get layers".to_string())?)
}

#[tauri::command]
pub async fn get_shapes() -> Result<HashMap<String, Shape>, ApiError> {
    Ok(_get_shapes()
        .await
        .with_context(|| "Could not get shapes".to_string())?)
}

#[tauri::command]
pub async fn get_places() -> Result<HashMap<String, Place>, ApiError> {
    Ok(_get_places()
        .await
        .with_context(|| "Could not get places".to_string())?)
}

#[tauri::command]
pub async fn create_layer(id: String, name: String, color: String) -> Result<(), ApiError> {
    debug!("Creating layer with id {id}, name {name} and color {color}");
    _create_layer(&id, &name, &color)
        .await
        .with_context(|| "Could not create layer".to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn set_layer_color(layer: String, value: String) -> Result<(), ApiError> {
    debug!("Setting layer {layer} color to {value}");

    let mut layers = LAYERS.lock().await;
    let target = layers.get_mut(&layer);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Layer {layer} not found")));
    }

    target
        .unwrap()
        .set_layer_color(&layer, &value)
        .await
        .with_context(|| format!("Could not set layer {0} color to {1}", layer, value))?;

    Ok(())
}

#[tauri::command]
pub async fn set_layer_name(layer: String, value: String) -> Result<(), ApiError> {
    debug!("Setting layer {layer} name to {value}");

    let mut layers = LAYERS.lock().await;
    let target = layers.get_mut(&layer);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Layer {layer} not found")));
    }

    target
        .unwrap()
        .set_layer_name(&layer, &value)
        .await
        .with_context(|| format!("Could not set layer {0} name to {1}", layer, value))?;

    Ok(())
}

#[tauri::command]
pub async fn delete_layer(
    layer: String,
    recursive: bool,
    new_layer: Option<String>,
) -> Result<(), ApiError> {
    debug!("Deleting layer {layer}");
    _delete_layer(&layer, recursive, &new_layer)
        .await
        .with_context(|| format!("Could not delete layer {0}", layer))?;

    Ok(())
}

#[tauri::command]
pub async fn create_place(
    id: String,
    name: String,
    lat: f32,
    lng: f32,
    layer: String,
    category: String,
) -> Result<(), ApiError> {
    debug!("Creating place {name} at {lat},{lng}");
    _create_place(id, &name, lat, lng, &layer, &category)
        .await
        .with_context(|| format!("Could not create place {0}", name))?;

    Ok(())
}

#[tauri::command]
pub async fn set_place_name(place: String, value: String) -> Result<(), ApiError> {
    debug!("Set place {place} name to {value}");

    let mut places = PLACES.lock().await;
    let target = places.get_mut(&place);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Place {place} not found")));
    }

    target
        .unwrap()
        .set_place_name(&place, &value)
        .await
        .with_context(|| format!("Could not set place {0} name to {1}", place, value))?;

    Ok(())
}

#[tauri::command]
pub async fn set_place_category(place: String, value: String) -> Result<(), ApiError> {
    debug!("Set place {place} category to {value}");

    let mut places = PLACES.lock().await;
    let target = places.get_mut(&place);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Place {place} not found")));
    }

    target
        .unwrap()
        .set_place_category(&place, &value)
        .await
        .with_context(|| format!("Could not set place {0} category to {1}", place, value))?;

    Ok(())
}

#[tauri::command]
pub async fn set_place_shape(place: String, value: Option<String>) -> Result<(), ApiError> {
    debug!(
        "Set place {place} shape to {}",
        value.as_ref().unwrap_or(&"NULL".to_string())
    );

    let mut places = PLACES.lock().await;
    let target = places.get_mut(&place);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Place {place} not found")));
    }

    target
        .unwrap()
        .set_place_shape(&place, &value)
        .await
        .with_context(|| {
            format!(
                "Could not set place {0} shape to {1}",
                place,
                value.unwrap_or("NULL".to_string())
            )
        })?;

    Ok(())
}

#[tauri::command]
pub async fn set_place_layer(place: String, layer: String) -> Result<(), ApiError> {
    debug!("Set place {place} layer to {layer}");

    let mut places = PLACES.lock().await;
    let target = places.get_mut(&place);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Place {place} not found")));
    }

    target
        .unwrap()
        .set_place_layer(&place, &layer)
        .await
        .with_context(|| format!("Could not set place {0} layer to {1}", place, layer))?;

    Ok(())
}

#[tauri::command]
pub async fn set_place_position(place: String, lat: f32, lng: f32) -> Result<(), ApiError> {
    debug!("Set place {place} position to {lat},{lng}");

    let mut places = PLACES.lock().await;
    let target = places.get_mut(&place);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Place {place} not found")));
    }

    target
        .unwrap()
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
pub async fn delete_place(place: String) -> Result<(), ApiError> {
    debug!("Deleting place {place}");
    _delete_place(&place)
        .await
        .with_context(|| format!("Could not delete place {0}", place))?;

    Ok(())
}

#[tauri::command]
pub async fn create_shape(
    id: String,
    shape_type: String,
    points: String,
    layer: String,
    name: String,
) -> Result<(), ApiError> {
    debug!("Creating shape {id}");
    _create_shape(&id, &shape_type, &points, &layer, &name)
        .await
        .with_context(|| "Could not create shape".to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn set_shape_points(shape: String, value: String) -> Result<(), ApiError> {
    debug!("Setting shape {shape} points");

    let mut shapes = SHAPES.lock().await;
    let target = shapes.get_mut(&shape);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Shape {shape} not found")));
    }

    target
        .unwrap()
        .set_shape_points(&shape, &value)
        .await
        .with_context(|| format!("Could not set shape {0} points", shape))?;

    Ok(())
}

#[tauri::command]
pub async fn set_shape_layer(shape: String, value: String) -> Result<(), ApiError> {
    debug!("Setting shape {shape} layer to {value}");

    let mut shapes = SHAPES.lock().await;
    let target = shapes.get_mut(&shape);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Shape {shape} not found")));
    }

    target
        .unwrap()
        .set_shape_layer(&shape, &value)
        .await
        .with_context(|| format!("Could not set shape {0} layer to {1}", shape, value))?;

    Ok(())
}

#[tauri::command]
pub async fn set_shape_name(shape: String, value: String) -> Result<(), ApiError> {
    debug!("Setting shape {shape} name to {value}");

    let mut shapes = SHAPES.lock().await;
    let target = shapes.get_mut(&shape);
    if target.is_none() {
        return Err(ApiError::NotFound(format!("Shape {shape} not found")));
    }

    target
        .unwrap()
        .set_shape_name(&shape, &value)
        .await
        .with_context(|| format!("Could not set shape {0} name to {1}", shape, value))?;

    Ok(())
}

#[tauri::command]
pub async fn delete_shape(shape: String) -> Result<(), ApiError> {
    debug!("Deleting shape {shape}");
    _delete_shape(&shape)
        .await
        .with_context(|| format!("Could not delete shape {0}", shape))?;

    Ok(())
}
