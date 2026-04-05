use std::collections::HashMap;

use diesel::{
    delete,
    dsl::{insert_into, update},
    query_builder::AsChangeset,
    ExpressionMethods, QueryDsl,
};
use diesel_async::RunQueryDsl;
use log::debug;
use serde::Serialize;
use tauri::State;

use crate::{
    models::{Layer, Photo, Place, Shape},
    photos::PhotoState,
    schema::{layers, photos, places, shapes},
    ApiError,
};

#[derive(Serialize, Clone)]
pub struct LayerDto {
    pub layer: Layer,
    pub count: i64,
}

impl From<Layer> for LayerDto {
    fn from(layer: Layer) -> Self {
        LayerDto { layer, count: 0 }
    }
}

#[derive(Serialize, Clone)]
pub struct PlaceDto {
    pub place: Place,
    pub count: i64,
}

impl From<Place> for PlaceDto {
    fn from(place: Place) -> Self {
        PlaceDto { place, count: 0 }
    }
}

#[tauri::command]
pub async fn get_layers(
    state: State<'_, PhotoState>,
) -> Result<HashMap<String, LayerDto>, ApiError> {
    Ok(state.layers.lock().await.clone())
}

#[tauri::command]
pub async fn get_shapes(state: State<'_, PhotoState>) -> Result<Vec<Shape>, ApiError> {
    let mut conn = state.db.lock().await;
    let conn = conn.as_mut().unwrap();

    Ok(shapes::table.load::<Shape>(conn).await?)
}

#[tauri::command]
pub async fn get_places(state: State<'_, PhotoState>) -> Result<HashMap<String, PlaceDto>, String> {
    Ok(state.places.lock().await.clone())
}

#[tauri::command]
pub async fn create_layer(
    state: State<'_, PhotoState>,
    id: String,
    name: String,
    color: String,
) -> Result<(), ApiError> {
    debug!("Creating layer with id {id}, name {name} and color {color}");
    let mut conn = state.db.lock().await;
    let conn = conn.as_mut().unwrap();

    insert_into(layers::table)
        .values(Layer { id, name, color })
        .execute(conn)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn set_layer_color(
    state: State<'_, PhotoState>,
    layer: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Setting layer {layer} color to {value}");
    let mut conn = state.db.lock().await;
    let conn = conn.as_mut().unwrap();

    update(layers::table.filter(layers::id.eq(layer)))
        .set(layers::color.eq(value))
        .execute(conn)
        .await?;

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
    let mut conn = state.db.lock().await;
    let conn = conn.as_mut().unwrap();

    if recursive {
        delete(shapes::table.filter(shapes::layer.eq(layer.clone())))
            .execute(conn)
            .await?;

        let mut state_photos = state.photos.lock().await;
        for place in places::table
            .filter(places::layer.eq(layer.clone()))
            .load::<Place>(conn)
            .await?
        {
            for photo in photos::table
                .filter(photos::location.eq(place.id))
                .load::<Photo>(conn)
                .await?
            {
                if state_photos.contains_key(&photo.name) {
                    state_photos.get_mut(&photo.name).unwrap().photo.location = None;
                    update(photos::table.filter(photos::name.eq(photo.name.clone())))
                        .set(photos::location.eq::<Option<String>>(None))
                        .execute(conn)
                        .await?;
                } else {
                    return Err(ApiError::NotFoundError(format!(
                        "Photo '{0}' not found",
                        photo.name
                    )));
                }
            }
        }

        delete(places::table.filter(places::layer.eq(layer.clone())))
            .execute(conn)
            .await?;
    } else if new_layer.is_some() {
        let new_layer = new_layer.unwrap();
        update(shapes::table.filter(shapes::layer.eq(layer.clone())))
            .set(shapes::layer.eq(new_layer.clone()))
            .execute(conn)
            .await?;
        update(places::table.filter(places::layer.eq(layer.clone())))
            .set(places::layer.eq(new_layer))
            .execute(conn)
            .await?;
    }

    delete(layers::table.filter(layers::id.eq(layer)))
        .execute(conn)
        .await?;

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
    let mut conn = state.db.lock().await;
    let conn = conn.as_mut().unwrap();

    let new_place = Place {
        id: id.clone(),
        name,
        lat,
        lng,
        layer: layer.clone(),
        category,
        shape: None,
    };

    insert_into(places::table)
        .values(new_place.clone())
        .execute(conn)
        .await?;

    let mut state_layers = state.layers.lock().await;
    if state_layers.contains_key(&layer) {
        state_layers.get_mut(&layer).unwrap().count += 1;
    }

    state
        .places
        .lock()
        .await
        .insert(id.clone(), PlaceDto::from(new_place));

    Ok(())
}

#[tauri::command]
pub async fn set_place_name(
    state: State<'_, PhotoState>,
    place: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Set place {place} name to {value}");
    let mut conn = state.db.lock().await;
    let conn = conn.as_mut().unwrap();

    update(places::table.filter(places::id.eq(place)))
        .set(places::name.eq(value))
        .execute(conn)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn set_place_category(
    state: State<'_, PhotoState>,
    place: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Set place {place} category to {value}");
    let mut conn = state.db.lock().await;
    let conn = conn.as_mut().unwrap();

    update(places::table.filter(places::id.eq(place)))
        .set(places::name.eq(value))
        .execute(conn)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn set_place_shape(
    state: State<'_, PhotoState>,
    place: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Set place {place} shape to {value}");
    let mut conn = state.db.lock().await;
    let conn = conn.as_mut().unwrap();

    update(places::table.filter(places::id.eq(place)))
        .set(places::shape.eq(value))
        .execute(conn)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn set_place_layer(
    state: State<'_, PhotoState>,
    place: String,
    layer: String,
) -> Result<(), ApiError> {
    debug!("Set place {place} layer to {layer}");
    let mut conn = state.db.lock().await;
    let conn = conn.as_mut().unwrap();

    let mut state_places = state.places.lock().await;
    if !state_places.contains_key(&place) {
        return Err(ApiError::NotFoundError(format!(
            "Place with Id '{0}' not found",
            place
        )));
    }
    let state_place = state_places.get_mut(&place).unwrap();

    update(places::table.filter(places::id.eq(place)))
        .set(places::layer.eq(layer.clone()))
        .execute(conn)
        .await?;

    let existing_layer = &state_place.place.layer;
    let mut state_layers = state.layers.lock().await;
    if state_layers.contains_key(existing_layer) {
        state_layers.get_mut(existing_layer).unwrap().count -= 1;
    }
    if state_layers.contains_key(&layer) {
        state_layers.get_mut(&layer).unwrap().count += 1;
    }
    state_place.place.layer = layer;

    Ok(())
}

#[derive(AsChangeset)]
#[diesel(table_name = places)]
struct PositionUpdate {
    lat: f32,
    lng: f32,
}

#[tauri::command]
pub async fn set_place_position(
    state: State<'_, PhotoState>,
    place: String,
    lat: f32,
    lng: f32,
) -> Result<(), ApiError> {
    debug!("Set place {place} position to {lat},{lng}");
    let mut conn = state.db.lock().await;
    let conn = conn.as_mut().unwrap();

    update(places::table.filter(places::id.eq(place)))
        .set(PositionUpdate { lat, lng })
        .execute(conn)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn delete_place(state: State<'_, PhotoState>, place: String) -> Result<(), ApiError> {
    debug!("Deleting place {place}");
    let mut conn = state.db.lock().await;
    let conn = conn.as_mut().unwrap();

    let mut state_places = state.places.lock().await;

    if !state_places.contains_key(&place) {
        return Err(ApiError::NotFoundError(format!(
            "Place with Id '{0}' not found",
            place
        )));
    }

    let mut state_photos = state.photos.lock().await;
    for photo in photos::table
        .filter(photos::location.eq(place.clone()))
        .load::<Photo>(conn)
        .await?
    {
        if state_photos.contains_key(&photo.name) {
            state_photos.get_mut(&photo.name).unwrap().photo.location = None;
            update(photos::table.filter(photos::name.eq(photo.name)))
                .set(photos::location.eq::<Option<String>>(None))
                .execute(conn)
                .await?;
        }
    }

    let state_place = state_places.get(&place).unwrap();
    let mut state_layers = state.layers.lock().await;
    if state_layers.contains_key(&state_place.place.layer) {
        state_layers
            .get_mut(&state_place.place.layer)
            .unwrap()
            .count -= 1;
    }

    delete(places::table.filter(places::id.eq(place.clone())))
        .execute(conn)
        .await?;
    state_places.remove(&place);
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
    let mut conn = state.db.lock().await;
    let conn = conn.as_mut().unwrap();

    insert_into(shapes::table)
        .values(Shape {
            id,
            shape_type,
            points,
            layer,
            name,
        })
        .execute(conn)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn set_shape_points(
    state: State<'_, PhotoState>,
    shape: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Setting shape {shape} points");
    let mut conn = state.db.lock().await;
    let conn = conn.as_mut().unwrap();

    update(shapes::table.filter(shapes::id.eq(shape)))
        .set(shapes::points.eq(value))
        .execute(conn)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn set_shape_layer(
    state: State<'_, PhotoState>,
    shape: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Setting shape {shape} layer to {value}");
    let mut conn = state.db.lock().await;
    let conn = conn.as_mut().unwrap();

    update(shapes::table.filter(shapes::id.eq(shape)))
        .set(shapes::layer.eq(value))
        .execute(conn)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn set_shape_name(
    state: State<'_, PhotoState>,
    shape: String,
    value: String,
) -> Result<(), ApiError> {
    debug!("Setting shape {shape} name to {value}");
    let mut conn = state.db.lock().await;
    let conn = conn.as_mut().unwrap();

    update(shapes::table.filter(shapes::id.eq(shape)))
        .set(shapes::name.eq(value))
        .execute(conn)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn delete_shape(state: State<'_, PhotoState>, shape: String) -> Result<(), ApiError> {
    debug!("Deleting shape {shape}");
    let mut conn = state.db.lock().await;
    let conn = conn.as_mut().unwrap();

    delete(shapes::table.filter(shapes::id.eq(shape)))
        .execute(conn)
        .await?;

    Ok(())
}
